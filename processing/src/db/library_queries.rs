use crate::error::ProcessingError;
use sqlx::SqlitePool;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NewLibrary {
    pub name:      String,
    pub db_path:   String,
    pub cover_dir: String,
    pub layout:    String,
    pub xs_url:    Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct LibraryRow {
    pub id:         String,
    pub name:       String,
    pub db_path:    String,
    pub cover_dir:  String,
    pub layout:     String,
    pub xs_url:     Option<String>,
    pub is_active:  bool,
    pub created_at: String,
    pub updated_at: String,
}

pub async fn create_library(pool: &SqlitePool, lib: &NewLibrary) -> Result<String, ProcessingError> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO libraries (id, name, db_path, cover_dir, layout, xs_url, created_at, updated_at)
         VALUES (?,?,?,?,?,?,?,?)",
    )
    .bind(&id).bind(&lib.name).bind(&lib.db_path).bind(&lib.cover_dir)
    .bind(&lib.layout).bind(&lib.xs_url).bind(&now).bind(&now)
    .execute(pool).await.map_err(ProcessingError::DbError)?;
    Ok(id)
}

pub async fn list_libraries(pool: &SqlitePool) -> Result<Vec<LibraryRow>, ProcessingError> {
    sqlx::query_as::<_, LibraryRow>(
        "SELECT id, name, db_path, cover_dir, layout, xs_url,
                CAST(is_active AS BOOLEAN) as is_active, created_at, updated_at
         FROM libraries ORDER BY created_at ASC",
    )
    .fetch_all(pool).await.map_err(ProcessingError::DbError)
}

pub async fn get_active_library(pool: &SqlitePool) -> Result<Option<LibraryRow>, ProcessingError> {
    sqlx::query_as::<_, LibraryRow>(
        "SELECT id, name, db_path, cover_dir, layout, xs_url,
                CAST(is_active AS BOOLEAN) as is_active, created_at, updated_at
         FROM libraries WHERE is_active = 1 LIMIT 1",
    )
    .fetch_optional(pool).await.map_err(ProcessingError::DbError)
}

pub async fn set_active_library(pool: &SqlitePool, id: &str) -> Result<(), ProcessingError> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("UPDATE libraries SET is_active = 0, updated_at = ? WHERE is_active = 1")
        .bind(&now).execute(pool).await.map_err(ProcessingError::DbError)?;
    sqlx::query("UPDATE libraries SET is_active = 1, updated_at = ? WHERE id = ?")
        .bind(&now).bind(id).execute(pool).await.map_err(ProcessingError::DbError)?;
    Ok(())
}

pub async fn update_library(
    pool: &SqlitePool, id: &str, name: &str, xs_url: Option<&str>,
) -> Result<(), ProcessingError> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("UPDATE libraries SET name = ?, xs_url = ?, updated_at = ? WHERE id = ?")
        .bind(name).bind(xs_url).bind(&now).bind(id)
        .execute(pool).await.map_err(ProcessingError::DbError)?;
    Ok(())
}

pub async fn delete_library(pool: &SqlitePool, id: &str) -> Result<(), ProcessingError> {
    sqlx::query("DELETE FROM libraries WHERE id = ?")
        .bind(id).execute(pool).await.map_err(ProcessingError::DbError)?;
    Ok(())
}
