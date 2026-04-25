use crate::error::ProcessingError;
use sqlx::SqlitePool;

/// Insert or replace a book's FTS entry.
/// Deletes any existing rows for this book_id first.
pub async fn upsert_fts(
    pool: &SqlitePool,
    book_id: &str,
    title: &str,
    authors: &str,
    description: &str,
    full_text: &str,
) -> Result<(), ProcessingError> {
    sqlx::query(
        "DELETE FROM books_fts WHERE rowid IN
         (SELECT rowid FROM books_fts WHERE book_id = ?)",
    )
    .bind(book_id)
    .execute(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    sqlx::query(
        "INSERT INTO books_fts (book_id, title, authors, description, full_text) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(book_id)
    .bind(title)
    .bind(authors)
    .bind(description)
    .bind(full_text)
    .execute(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    Ok(())
}

/// Search the FTS index. Returns matching book_ids.
pub async fn search(pool: &SqlitePool, query: &str) -> Result<Vec<String>, ProcessingError> {
    let rows = sqlx::query_as::<_, (String,)>(
        "SELECT DISTINCT book_id FROM books_fts WHERE books_fts MATCH ? ORDER BY rank",
    )
    .bind(query)
    .fetch_all(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    Ok(rows.into_iter().map(|(id,)| id).collect())
}

pub async fn refresh_book_index(
    pool: &SqlitePool,
    book_id: &str,
) -> Result<(), ProcessingError> {
    let row = sqlx::query_as::<_, (String, String, Option<String>, Option<String>)>(
        "SELECT lb.title, lb.authors_json, lb.description, jt.full_text
         FROM local_books lb
         LEFT JOIN job_text jt ON jt.job_id = lb.id
         WHERE lb.id = ?",
    )
    .bind(book_id)
    .fetch_optional(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    let Some((title, authors_json, description, full_text)) = row else {
        return Ok(());
    };

    upsert_fts(
        pool,
        book_id,
        &title,
        &authors_json,
        description.as_deref().unwrap_or(""),
        full_text.as_deref().unwrap_or(""),
    )
    .await
}
