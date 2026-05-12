use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct VirtualLibrary {
    pub id:          String,
    pub library_id:  Option<String>,
    pub name:        String,
    pub search_expr: String,
    pub sort_field:  String,
    pub sort_asc:    bool,
    pub created_at:  String,
    pub updated_at:  String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewVirtualLibrary {
    pub library_id:  Option<String>,
    pub name:        String,
    pub search_expr: String,
    pub sort_field:  String,
    pub sort_asc:    bool,
}

pub async fn create_virtual_library(
    pool: &SqlitePool,
    new: &NewVirtualLibrary,
) -> Result<VirtualLibrary, sqlx::Error> {
    let row = sqlx::query_as::<_, VirtualLibrary>(
        "INSERT INTO virtual_libraries (library_id, name, search_expr, sort_field, sort_asc)
         VALUES (?, ?, ?, ?, ?)
         RETURNING *"
    )
    .bind(&new.library_id)
    .bind(&new.name)
    .bind(&new.search_expr)
    .bind(&new.sort_field)
    .bind(new.sort_asc)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn list_virtual_libraries(
    pool: &SqlitePool,
    library_id: Option<&str>,
) -> Result<Vec<VirtualLibrary>, sqlx::Error> {
    match library_id {
        Some(lid) => {
            sqlx::query_as::<_, VirtualLibrary>(
                "SELECT * FROM virtual_libraries WHERE library_id = ? ORDER BY name"
            )
            .bind(lid)
            .fetch_all(pool)
            .await
        }
        None => {
            sqlx::query_as::<_, VirtualLibrary>(
                "SELECT * FROM virtual_libraries ORDER BY name"
            )
            .fetch_all(pool)
            .await
        }
    }
}

pub async fn update_virtual_library(
    pool: &SqlitePool,
    id: &str,
    name: &str,
    search_expr: &str,
    sort_field: &str,
    sort_asc: bool,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE virtual_libraries
         SET name=?, search_expr=?, sort_field=?, sort_asc=?,
             updated_at=datetime('now')
         WHERE id=?"
    )
    .bind(name)
    .bind(search_expr)
    .bind(sort_field)
    .bind(sort_asc)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_virtual_library(
    pool: &SqlitePool,
    id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM virtual_libraries WHERE id=?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_virtual_library(
    pool: &SqlitePool,
    id: &str,
) -> Result<Option<VirtualLibrary>, sqlx::Error> {
    sqlx::query_as::<_, VirtualLibrary>(
        "SELECT * FROM virtual_libraries WHERE id=?"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}
