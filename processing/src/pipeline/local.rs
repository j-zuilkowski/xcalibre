use crate::db::{extended_queries, fts_queries, queries};
use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use crate::pipeline::{cover, ingest, metadata, text};
use crate::utils::sort::{author_sort, title_sort};
use sqlx::SqlitePool;
use std::path::Path;
use std::path::PathBuf;
use tracing::warn;

pub async fn import_local_book(
    pool: &SqlitePool,
    path: &Path,
) -> Result<ingest::IngestResult, ProcessingError> {
    let result = ingest::run_ingest(pool, path).await?;

    let meta = match metadata::run_metadata(pool, &result, path).await {
        Ok(meta) => meta,
        Err(err) => {
            warn!(job_id = %result.job_id, error = %err, "metadata extraction failed; continuing with fallback metadata");
            BookMetadata::default()
        }
    };

    let fallback_title = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("Untitled")
        .to_string();
    let title = meta.title.clone().unwrap_or(fallback_title);
    let authors = if meta.authors.is_empty() {
        vec!["Unknown".to_string()]
    } else {
        meta.authors.clone()
    };

    let cover_path = write_placeholder_cover(path, &title, &authors, &result.format.to_string())?;

    queries::upsert_local_book(
        pool,
        &result.job_id,
        &title,
        &authors,
        &result.format.to_string(),
        Some(path.to_string_lossy().as_ref()),
        Some(cover_path.to_string_lossy().as_ref()),
    )
    .await?;

    let title_sort_value = title_sort(&title);
    let author_sort_value = authors
        .first()
        .map(|author| author_sort(author))
        .unwrap_or_default();

    if let Err(err) = extended_queries::update_extended_metadata(
        pool,
        &result.job_id,
        Some(&title_sort_value),
        Some(&author_sort_value),
        meta.published.as_deref(),
        meta.description.as_deref(),
        meta.publisher.as_deref(),
        meta.series.as_deref(),
        meta.series_index.map(f64::from),
        None,
    )
    .await
    {
        warn!(job_id = %result.job_id, error = %err, "extended metadata update failed; continuing");
    }

    if let Some(isbn) = meta.isbn.as_deref() {
        if let Err(err) = extended_queries::upsert_identifier(pool, &result.job_id, "isbn", isbn)
            .await
        {
            warn!(job_id = %result.job_id, error = %err, "identifier update failed; continuing");
        }
    }

    for tag in &meta.tags {
        if let Err(err) = extended_queries::upsert_tag(pool, &result.job_id, tag).await {
            warn!(job_id = %result.job_id, error = %err, tag = %tag, "tag update failed; continuing");
        }
    }

    if let Err(err) = text::run_text(pool, &result, path).await {
        warn!(job_id = %result.job_id, error = %err, "text extraction failed; continuing");
    }
    if let Err(err) = fts_queries::refresh_book_index(pool, &result.job_id).await {
        warn!(job_id = %result.job_id, error = %err, "fts refresh failed; continuing");
    }
    if let Err(err) = cover::run_cover(pool, &result, path).await {
        warn!(job_id = %result.job_id, error = %err, "cover extraction failed; continuing");
    }

    queries::update_job_status(pool, &result.job_id, "READY_TO_PUSH").await?;
    Ok(result)
}

pub(crate) fn write_placeholder_cover(
    book_path: &Path,
    title: &str,
    authors: &[String],
    format: &str,
) -> Result<PathBuf, ProcessingError> {
    let cover_path = book_path.with_extension("placeholder.cover.svg");
    let author_text = if authors.is_empty() {
        "Unknown".to_string()
    } else {
        authors.join(", ")
    };
    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="500" height="750" viewBox="0 0 500 750">
  <defs>
    <linearGradient id="bg" x1="0" x2="1" y1="0" y2="1">
      <stop offset="0%" stop-color="#dbeafe"/>
      <stop offset="100%" stop-color="#bfdbfe"/>
    </linearGradient>
  </defs>
  <rect width="500" height="750" rx="28" fill="url(#bg)"/>
  <rect x="32" y="32" width="436" height="686" rx="20" fill="#ffffff" fill-opacity="0.7"/>
  <text x="64" y="120" font-family="Inter, Arial, sans-serif" font-size="42" font-weight="700" fill="#0f172a">{}</text>
  <text x="64" y="190" font-family="Inter, Arial, sans-serif" font-size="24" fill="#334155">{}</text>
  <text x="64" y="680" font-family="Inter, Arial, sans-serif" font-size="28" font-weight="600" fill="#1d4ed8">{}</text>
</svg>"##,
        escape_svg(title),
        escape_svg(&author_text),
        escape_svg(format),
    );
    std::fs::write(&cover_path, svg).map_err(ProcessingError::IoError)?;
    Ok(cover_path)
}

fn escape_svg(text: &str) -> String {
    text
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
