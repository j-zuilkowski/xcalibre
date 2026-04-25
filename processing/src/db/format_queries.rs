use crate::error::ProcessingError;
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct BookFormat {
    pub id:          String,
    pub book_id:     String,
    pub format:      String,
    pub file_path:   String,
    pub file_sha256: String,
    pub file_size:   i64,
    pub added_at:    String,
}

pub async fn insert_book_format(
    pool: &SqlitePool,
    book_id: &str,
    format: &str,
    file_path: &str,
    file_sha256: &str,
    file_size: i64,
) -> Result<String, ProcessingError> {
    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO book_formats
         (id, book_id, format, file_path, file_sha256, file_size)
         VALUES (?, ?, ?, ?, ?, ?)
         ON CONFLICT(file_path) DO UPDATE SET
             book_id     = excluded.book_id,
             format      = excluded.format,
             file_sha256 = excluded.file_sha256,
             file_size   = excluded.file_size,
             added_at    = datetime('now')",
    )
    .bind(&id)
    .bind(book_id)
    .bind(format)
    .bind(file_path)
    .bind(file_sha256)
    .bind(file_size)
    .execute(pool)
    .await
    .map_err(ProcessingError::DbError)?;
    Ok(id)
}

pub async fn get_formats_for_book(
    pool: &SqlitePool,
    book_id: &str,
) -> Result<Vec<BookFormat>, ProcessingError> {
    let rows = sqlx::query_as::<_, (String, String, String, String, String, i64, String)>(
        "SELECT id, book_id, format, file_path, file_sha256, file_size, added_at
         FROM book_formats WHERE book_id = ? ORDER BY added_at ASC",
    )
    .bind(book_id)
    .fetch_all(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    Ok(rows
        .into_iter()
        .map(|(id, book_id, format, file_path, file_sha256, file_size, added_at)| BookFormat {
            id,
            book_id,
            format,
            file_path,
            file_sha256,
            file_size,
            added_at,
        })
        .collect())
}
