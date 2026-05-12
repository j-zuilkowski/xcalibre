use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReadingSession {
    pub id:             String,
    pub book_id:        String,
    pub started_at:     String,
    pub ended_at:       Option<String>,
    pub duration_s:     i64,
    pub progress_start: f64,
    pub progress_end:   f64,
}

pub async fn start_reading_session(
    pool: &SqlitePool,
    book_id: &str,
    progress_start: f64,
) -> Result<String, sqlx::Error> {
    let row: (String,) = sqlx::query_as(
        "INSERT INTO reading_sessions (book_id, progress_start)
         VALUES (?, ?)
         RETURNING id"
    )
    .bind(book_id)
    .bind(progress_start)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

pub async fn end_reading_session(
    pool: &SqlitePool,
    session_id: &str,
    duration_s: i64,
    progress_start: f64,
    progress_end: f64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE reading_sessions
         SET ended_at=datetime('now'), duration_s=?, progress_start=?, progress_end=?
         WHERE id=?"
    )
    .bind(duration_s)
    .bind(progress_start)
    .bind(progress_end)
    .bind(session_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_reading_sessions(
    pool: &SqlitePool,
    book_id: &str,
) -> Result<Vec<ReadingSession>, sqlx::Error> {
    sqlx::query_as::<_, ReadingSession>(
        "SELECT * FROM reading_sessions WHERE book_id=? ORDER BY started_at DESC"
    )
    .bind(book_id)
    .fetch_all(pool)
    .await
}

pub async fn total_reading_time_seconds(
    pool: &SqlitePool,
    book_id: &str,
) -> Result<i64, sqlx::Error> {
    let row: (Option<i64>,) = sqlx::query_as(
        "SELECT COALESCE(SUM(duration_s), 0) FROM reading_sessions WHERE book_id=?"
    )
    .bind(book_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0.unwrap_or(0))
}
