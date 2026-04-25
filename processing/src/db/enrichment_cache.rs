use crate::error::ProcessingError;
use sqlx::SqlitePool;

pub async fn get_cache(
    pool: &SqlitePool,
    isbn: &str,
) -> Result<Option<(String, String)>, ProcessingError> {
    let row: Option<(String, String)> = sqlx::query_as(
        "SELECT ol_json, gb_json FROM enrichment_cache
         WHERE isbn = ? AND fetched_at >= datetime('now', '-30 days')",
    )
    .bind(isbn)
    .fetch_optional(pool)
    .await
    .map_err(ProcessingError::DbError)?;
    Ok(row)
}

pub async fn set_cache(
    pool: &SqlitePool,
    isbn: &str,
    ol_json: &str,
    gb_json: &str,
) -> Result<(), ProcessingError> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT OR REPLACE INTO enrichment_cache (isbn, ol_json, gb_json, fetched_at)
         VALUES (?, ?, ?, ?)",
    )
    .bind(isbn)
    .bind(ol_json)
    .bind(gb_json)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(ProcessingError::DbError)?;
    Ok(())
}