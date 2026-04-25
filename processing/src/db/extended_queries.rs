use crate::error::ProcessingError;
use chrono::Utc;
use sqlx::SqlitePool;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookIdentifier {
    pub id_type: String,
    pub value:   String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookDetails {
    pub id:            String,
    pub title:         String,
    pub authors:       Vec<String>,
    pub format:        String,
    pub local_path:    Option<String>,
    pub cover_path:    Option<String>,
    pub title_sort:    Option<String>,
    pub author_sort:   Option<String>,
    pub pubdate:       Option<String>,
    pub description:   Option<String>,
    pub publisher:     Option<String>,
    pub series_name:   Option<String>,
    pub series_index:  Option<f64>,
    pub rating:        i64,
    pub tags:          Vec<String>,
    pub identifiers:   Vec<BookIdentifier>,
}

// ── Tags ──────────────────────────────────────────────────────────────────────

pub async fn upsert_tag(
    pool: &SqlitePool,
    book_id: &str,
    tag: &str,
) -> Result<(), ProcessingError> {
    sqlx::query("INSERT OR IGNORE INTO tags (name) VALUES (?)")
        .bind(tag)
        .execute(pool)
        .await
        .map_err(ProcessingError::DbError)?;

    sqlx::query(
        "INSERT OR IGNORE INTO book_tags (book_id, tag_id)
         SELECT ?, id FROM tags WHERE name = ?",
    )
    .bind(book_id)
    .bind(tag)
    .execute(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    Ok(())
}

pub async fn get_tags(pool: &SqlitePool, book_id: &str) -> Result<Vec<String>, ProcessingError> {
    let rows = sqlx::query_as::<_, (String,)>(
        "SELECT t.name FROM tags t
         JOIN book_tags bt ON bt.tag_id = t.id
         WHERE bt.book_id = ?
         ORDER BY t.name",
    )
    .bind(book_id)
    .fetch_all(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    Ok(rows.into_iter().map(|(name,)| name).collect())
}

pub async fn remove_tag(
    pool: &SqlitePool,
    book_id: &str,
    tag: &str,
) -> Result<(), ProcessingError> {
    sqlx::query(
        "DELETE FROM book_tags WHERE book_id = ?
         AND tag_id = (SELECT id FROM tags WHERE name = ?)",
    )
    .bind(book_id)
    .bind(tag)
    .execute(pool)
    .await
    .map_err(ProcessingError::DbError)?;
    Ok(())
}

// ── Identifiers ───────────────────────────────────────────────────────────────

pub async fn upsert_identifier(
    pool: &SqlitePool,
    book_id: &str,
    id_type: &str,
    value: &str,
) -> Result<(), ProcessingError> {
    sqlx::query(
        "INSERT INTO identifiers (book_id, type, value) VALUES (?, ?, ?)
         ON CONFLICT(book_id, type) DO UPDATE SET value = excluded.value",
    )
    .bind(book_id)
    .bind(id_type)
    .bind(value)
    .execute(pool)
    .await
    .map_err(ProcessingError::DbError)?;
    Ok(())
}

pub async fn get_identifiers(
    pool: &SqlitePool,
    book_id: &str,
) -> Result<Vec<(String, String)>, ProcessingError> {
    let rows = sqlx::query_as::<_, (String, String)>(
        "SELECT type, value FROM identifiers WHERE book_id = ? ORDER BY type",
    )
    .bind(book_id)
    .fetch_all(pool)
    .await
    .map_err(ProcessingError::DbError)?;
    Ok(rows)
}

// ── Extended metadata update ──────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
pub async fn update_extended_metadata(
    pool: &SqlitePool,
    book_id: &str,
    title_sort: Option<&str>,
    author_sort: Option<&str>,
    pubdate: Option<&str>,
    description: Option<&str>,
    publisher: Option<&str>,
    series_name: Option<&str>,
    series_index: Option<f64>,
    rating: Option<i64>,
) -> Result<(), ProcessingError> {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "UPDATE local_books SET
             title_sort   = COALESCE(?, title_sort),
             author_sort  = COALESCE(?, author_sort),
             pubdate      = COALESCE(?, pubdate),
             description  = COALESCE(?, description),
             publisher    = COALESCE(?, publisher),
             series_name  = COALESCE(?, series_name),
             series_index = COALESCE(?, series_index),
             rating       = COALESCE(?, rating),
             updated_at   = ?
         WHERE id = ?",
    )
    .bind(title_sort)
    .bind(author_sort)
    .bind(pubdate)
    .bind(description)
    .bind(publisher)
    .bind(series_name)
    .bind(series_index)
    .bind(rating)
    .bind(&now)
    .bind(book_id)
    .execute(pool)
    .await
    .map_err(ProcessingError::DbError)?;
    Ok(())
}

pub async fn get_book_details(
    pool: &SqlitePool,
    book_id: &str,
) -> Result<Option<BookDetails>, ProcessingError> {
    let row = sqlx::query_as::<_, (
        String,
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<f64>,
        i64,
    )>(
        "SELECT id, title, authors_json, format, local_path, cover_path,
                title_sort, author_sort, pubdate, description, publisher,
                series_name, series_index, rating
         FROM local_books WHERE id = ?",
    )
    .bind(book_id)
    .fetch_optional(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    let Some((
        id,
        title,
        authors_json,
        format,
        local_path,
        cover_path,
        title_sort,
        author_sort,
        pubdate,
        description,
        publisher,
        series_name,
        series_index,
        rating,
    )) = row else {
        return Ok(None);
    };

    let authors: Vec<String> = serde_json::from_str(&authors_json)
        .map_err(|e| ProcessingError::DbError(sqlx::Error::Protocol(e.to_string())))?;

    let tag_rows = sqlx::query_as::<_, (String,)>(
        "SELECT t.name FROM tags t
         JOIN book_tags bt ON bt.tag_id = t.id
         WHERE bt.book_id = ?
         ORDER BY t.name",
    )
    .bind(book_id)
    .fetch_all(pool)
    .await
    .map_err(ProcessingError::DbError)?;
    let tags = tag_rows.into_iter().map(|(tag,)| tag).collect();

    let identifier_rows = sqlx::query_as::<_, (String, String)>(
        "SELECT type, value FROM identifiers WHERE book_id = ? ORDER BY type",
    )
    .bind(book_id)
    .fetch_all(pool)
    .await
    .map_err(ProcessingError::DbError)?;
    let identifiers = identifier_rows
        .into_iter()
        .map(|(id_type, value)| BookIdentifier { id_type, value })
        .collect();

    Ok(Some(BookDetails {
        id,
        title,
        authors,
        format,
        local_path,
        cover_path,
        title_sort,
        author_sort,
        pubdate,
        description,
        publisher,
        series_name,
        series_index,
        rating,
        tags,
        identifiers,
    }))
}

#[allow(clippy::too_many_arguments)]
pub async fn replace_book_details(
    pool: &SqlitePool,
    book_id: &str,
    title: &str,
    authors: &[String],
    title_sort: &str,
    author_sort: &str,
    pubdate: Option<&str>,
    description: Option<&str>,
    publisher: Option<&str>,
    series_name: Option<&str>,
    series_index: Option<f64>,
    rating: i64,
) -> Result<(), ProcessingError> {
    let now = Utc::now().to_rfc3339();
    let authors_json = serde_json::to_string(authors)
        .map_err(|e| ProcessingError::DbError(sqlx::Error::Protocol(e.to_string())))?;

    sqlx::query(
        "UPDATE local_books SET
             title       = ?,
             authors_json = ?,
             title_sort  = ?,
             author_sort = ?,
             pubdate     = ?,
             description = ?,
             publisher   = ?,
             series_name = ?,
             series_index = ?,
             rating      = ?,
             updated_at  = ?
         WHERE id = ?",
    )
    .bind(title)
    .bind(&authors_json)
    .bind(title_sort)
    .bind(author_sort)
    .bind(pubdate)
    .bind(description)
    .bind(publisher)
    .bind(series_name)
    .bind(series_index)
    .bind(rating)
    .bind(&now)
    .bind(book_id)
    .execute(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    Ok(())
}

pub async fn replace_tags(
    pool: &SqlitePool,
    book_id: &str,
    tags: &[String],
) -> Result<(), ProcessingError> {
    sqlx::query("DELETE FROM book_tags WHERE book_id = ?")
        .bind(book_id)
        .execute(pool)
        .await
        .map_err(ProcessingError::DbError)?;

    for tag in tags {
        upsert_tag(pool, book_id, tag).await?;
    }

    Ok(())
}

pub async fn replace_identifiers(
    pool: &SqlitePool,
    book_id: &str,
    identifiers: &[BookIdentifier],
) -> Result<(), ProcessingError> {
    sqlx::query("DELETE FROM identifiers WHERE book_id = ?")
        .bind(book_id)
        .execute(pool)
        .await
        .map_err(ProcessingError::DbError)?;

    for identifier in identifiers {
        if identifier.id_type.trim().is_empty() || identifier.value.trim().is_empty() {
            continue;
        }
        upsert_identifier(pool, book_id, &identifier.id_type, &identifier.value).await?;
    }

    Ok(())
}
