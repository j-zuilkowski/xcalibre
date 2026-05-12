use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyBookOptions {
    pub move_file: bool,
}

pub async fn copy_book_to_library(
    pool: &SqlitePool,
    book_id: &str,
    target_library_id: &str,
    _opts: &CopyBookOptions,
) -> Result<String, sqlx::Error> {
    let book = sqlx::query_as::<_, BookRow>(
        "SELECT id, title, authors_json, format, local_path, cover_path,
                series_name, series_index, description, publisher, pubdate,
                created_at, updated_at
         FROM local_books WHERE id=?"
    )
    .bind(book_id)
    .fetch_one(pool)
    .await?;

    let new_id = unique_id();

    sqlx::query(
        "INSERT INTO local_books
         (id, title, authors_json, format, local_path, cover_path, series_name,
          series_index, description, publisher, pubdate, library_id, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&new_id)
    .bind(&book.title)
    .bind(&book.authors_json)
    .bind(&book.format)
    .bind(&book.local_path)
    .bind(&book.cover_path)
    .bind(&book.series_name)
    .bind(book.series_index)
    .bind(&book.description)
    .bind(&book.publisher)
    .bind(&book.pubdate)
    .bind(target_library_id)
    .bind(&book.created_at)
    .bind(&book.updated_at)
    .execute(pool)
    .await?;

    Ok(new_id)
}

#[derive(sqlx::FromRow)]
struct BookRow {
    #[allow(dead_code)]
    id:           String,
    title:        String,
    authors_json: String,
    format:       String,
    local_path:   Option<String>,
    cover_path:   Option<String>,
    series_name:  Option<String>,
    series_index: Option<f64>,
    description:  Option<String>,
    publisher:    Option<String>,
    pubdate:      Option<String>,
    created_at:   String,
    updated_at:   String,
}

fn unique_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    format!("copy-{nanos:08x}")
}
