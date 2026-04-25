use crate::error::ProcessingError;
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Collection {
    pub id:         String,
    pub name:       String,
    pub created_at: String,
}

pub async fn create_collection(
    pool: &SqlitePool,
    name: &str,
) -> Result<String, ProcessingError> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("INSERT INTO collections (id, name, created_at) VALUES (?, ?, ?)")
        .bind(&id)
        .bind(name)
        .bind(&now)
        .execute(pool)
        .await
        .map_err(ProcessingError::DbError)?;
    Ok(id)
}

pub async fn list_collections(pool: &SqlitePool) -> Result<Vec<Collection>, ProcessingError> {
    let rows = sqlx::query_as::<_, (String, String, String)>(
        "SELECT id, name, created_at FROM collections ORDER BY name",
    )
    .fetch_all(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    Ok(rows
        .into_iter()
        .map(|(id, name, created_at)| Collection { id, name, created_at })
        .collect())
}

pub async fn delete_collection(pool: &SqlitePool, id: &str) -> Result<(), ProcessingError> {
    sqlx::query("DELETE FROM collection_books WHERE collection_id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(ProcessingError::DbError)?;
    sqlx::query("DELETE FROM collections WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(ProcessingError::DbError)?;
    Ok(())
}

pub async fn add_book_to_collection(
    pool: &SqlitePool,
    book_id: &str,
    collection_id: &str,
) -> Result<(), ProcessingError> {
    sqlx::query(
        "INSERT OR IGNORE INTO collection_books (collection_id, book_id, added_at)
         VALUES (?, ?, ?)",
    )
    .bind(collection_id)
    .bind(book_id)
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(pool)
    .await
    .map_err(ProcessingError::DbError)?;
    Ok(())
}

pub async fn remove_book_from_collection(
    pool: &SqlitePool,
    book_id: &str,
    collection_id: &str,
) -> Result<(), ProcessingError> {
    sqlx::query("DELETE FROM collection_books WHERE collection_id = ? AND book_id = ?")
        .bind(collection_id)
        .bind(book_id)
        .execute(pool)
        .await
        .map_err(ProcessingError::DbError)?;
    Ok(())
}

pub async fn get_books_in_collection(
    pool: &SqlitePool,
    collection_id: &str,
) -> Result<Vec<String>, ProcessingError> {
    let rows = sqlx::query_as::<_, (String,)>(
        "SELECT book_id FROM collection_books WHERE collection_id = ? ORDER BY added_at ASC",
    )
    .bind(collection_id)
    .fetch_all(pool)
    .await
    .map_err(ProcessingError::DbError)?;
    Ok(rows.into_iter().map(|(id,)| id).collect())
}
