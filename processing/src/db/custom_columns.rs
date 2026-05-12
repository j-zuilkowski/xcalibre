use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CustomColumn {
    pub id:              String,
    pub library_id:      Option<String>,
    pub name:            String,
    pub label:           String,
    pub col_type:        String,
    pub is_multiple:     bool,
    pub display_in_grid: bool,
    pub created_at:      String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewCustomColumn {
    pub library_id:      Option<String>,
    pub name:            String,
    pub label:           String,
    pub col_type:        String,
    pub is_multiple:     bool,
    pub display_in_grid: bool,
}

pub async fn create_custom_column(
    pool: &SqlitePool,
    new: &NewCustomColumn,
) -> Result<CustomColumn, sqlx::Error> {
    sqlx::query_as::<_, CustomColumn>(
        "INSERT INTO custom_columns (library_id, name, label, col_type, is_multiple, display_in_grid)
         VALUES (?, ?, ?, ?, ?, ?)
         RETURNING *"
    )
    .bind(&new.library_id)
    .bind(&new.name)
    .bind(&new.label)
    .bind(&new.col_type)
    .bind(new.is_multiple)
    .bind(new.display_in_grid)
    .fetch_one(pool)
    .await
}

pub async fn list_custom_columns(
    pool: &SqlitePool,
    library_id: Option<&str>,
) -> Result<Vec<CustomColumn>, sqlx::Error> {
    match library_id {
        Some(lid) => {
            sqlx::query_as::<_, CustomColumn>(
                "SELECT * FROM custom_columns WHERE library_id=? ORDER BY created_at"
            )
            .bind(lid)
            .fetch_all(pool)
            .await
        }
        None => {
            sqlx::query_as::<_, CustomColumn>(
                "SELECT * FROM custom_columns ORDER BY created_at"
            )
            .fetch_all(pool)
            .await
        }
    }
}

pub async fn update_custom_column(
    pool: &SqlitePool,
    id: &str,
    label: &str,
    display_in_grid: bool,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE custom_columns SET label=?, display_in_grid=? WHERE id=?"
    )
    .bind(label)
    .bind(display_in_grid)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_custom_column(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM custom_columns WHERE id=?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn set_book_custom_value(
    pool: &SqlitePool,
    book_id: &str,
    column_id: &str,
    value: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO book_custom_values (book_id, column_id, value)
         VALUES (?, ?, ?)
         ON CONFLICT(book_id, column_id) DO UPDATE SET value=excluded.value"
    )
    .bind(book_id)
    .bind(column_id)
    .bind(value)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_book_custom_value(
    pool: &SqlitePool,
    book_id: &str,
    column_id: &str,
) -> Result<Option<String>, sqlx::Error> {
    let row: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT value FROM book_custom_values WHERE book_id=? AND column_id=?"
    )
    .bind(book_id)
    .bind(column_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.and_then(|(v,)| v))
}

pub async fn get_all_book_custom_values(
    pool: &SqlitePool,
    book_id: &str,
) -> Result<HashMap<String, Option<String>>, sqlx::Error> {
    let rows: Vec<(String, Option<String>)> = sqlx::query_as(
        "SELECT column_id, value FROM book_custom_values WHERE book_id=?"
    )
    .bind(book_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().collect())
}
