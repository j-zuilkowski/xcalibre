use crate::db::{extended_queries, format_queries, fts_queries};
use crate::error::ProcessingError;
use crate::pipeline::local::write_placeholder_cover;
use crate::utils::sort::{author_sort, title_sort};
use sqlx::SqlitePool;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RepairResult {
    pub book_id:          String,
    pub fixed_fields:     Vec<String>,
    pub remaining_issues: Vec<String>,
}

pub async fn repair_books(
    pool: &SqlitePool,
    book_ids: &[String],
) -> Result<Vec<RepairResult>, ProcessingError> {
    let targets = if book_ids.is_empty() {
        sqlx::query_as::<_, (String,)>("SELECT id FROM local_books ORDER BY title")
            .fetch_all(pool)
            .await
            .map_err(ProcessingError::DbError)?
            .into_iter()
            .map(|(id,)| id)
            .collect::<Vec<_>>()
    } else {
        book_ids.to_vec()
    };

    let mut results = Vec::with_capacity(targets.len());
    for book_id in targets {
        results.push(repair_single_book(pool, &book_id).await?);
    }
    Ok(results)
}

async fn repair_single_book(
    pool: &SqlitePool,
    book_id: &str,
) -> Result<RepairResult, ProcessingError> {
    let Some(details) = extended_queries::get_book_details(pool, book_id).await? else {
        return Err(ProcessingError::MetadataError("book not found".into()));
    };

    let mut fixed_fields = Vec::new();
    let mut remaining_issues = Vec::new();

    let authors = if details.authors.is_empty() {
        fixed_fields.push("authors".to_string());
        vec!["Unknown".to_string()]
    } else {
        details.authors.clone()
    };
    let title = if details.title.trim().is_empty() {
        fixed_fields.push("title".to_string());
        "Untitled".to_string()
    } else {
        details.title.clone()
    };
    let title_sort_value = title_sort(&title);
    let author_sort_value = authors
        .first()
        .map(|author| author_sort(author))
        .unwrap_or_default();

    let source_path = resolve_source_path(pool, book_id, details.local_path.as_deref()).await;
    let mut local_path = details.local_path.clone();
    if local_path
        .as_deref()
        .is_none_or(|path| !Path::new(path).exists())
    {
        if let Ok(candidate) = &source_path {
            local_path = Some(candidate.to_string_lossy().into_owned());
            fixed_fields.push("local_path".to_string());
        } else {
            remaining_issues.push("missing_file".to_string());
        }
    }

    let mut cover_path = details.cover_path.clone();
    let cover_is_missing = cover_path
        .as_deref()
        .is_none_or(|path| !Path::new(path).exists());
    if cover_is_missing {
        let temp_base = std::env::temp_dir().join(format!("{book_id}.epub"));
        let base = source_path
            .as_ref()
            .ok()
            .map(|path| path.as_path())
            .or_else(|| local_path.as_deref().map(Path::new))
            .unwrap_or(temp_base.as_path());
        let generated = write_placeholder_cover(base, &title, &authors, &details.format)?;
        cover_path = Some(generated.to_string_lossy().into_owned());
        fixed_fields.push("cover_path".to_string());
    }

    let rating = details.rating.clamp(0, 5);
    if rating != details.rating {
        fixed_fields.push("rating".to_string());
    }

    extended_queries::replace_book_details(
        pool,
        book_id,
        &title,
        &authors,
        &title_sort_value,
        &author_sort_value,
        details.pubdate.as_deref(),
        details.description.as_deref(),
        details.publisher.as_deref(),
        details.series_name.as_deref(),
        details.series_index,
        rating,
    )
    .await?;

    if let Some(path) = local_path.as_deref() {
        sqlx::query("UPDATE local_books SET local_path = ?, updated_at = ? WHERE id = ?")
            .bind(path)
            .bind(chrono::Utc::now().to_rfc3339())
            .bind(book_id)
            .execute(pool)
            .await
            .map_err(ProcessingError::DbError)?;
    }

    if let Some(path) = cover_path.as_deref() {
        sqlx::query("UPDATE local_books SET cover_path = ?, updated_at = ? WHERE id = ?")
            .bind(path)
            .bind(chrono::Utc::now().to_rfc3339())
            .bind(book_id)
            .execute(pool)
            .await
            .map_err(ProcessingError::DbError)?;
    }

    fts_queries::refresh_book_index(pool, book_id).await?;

    Ok(RepairResult {
        book_id: book_id.to_string(),
        fixed_fields,
        remaining_issues,
    })
}

async fn resolve_source_path(
    pool: &SqlitePool,
    book_id: &str,
    local_path: Option<&str>,
) -> Result<PathBuf, ProcessingError> {
    if let Some(path) = local_path {
        let candidate = PathBuf::from(path);
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    for format in format_queries::get_formats_for_book(pool, book_id).await? {
        let candidate = PathBuf::from(format.file_path);
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(ProcessingError::MetadataError(
        "no readable source path available".to_string(),
    ))
}
