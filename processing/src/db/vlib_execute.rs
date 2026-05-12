use crate::db::vlib_queries::VirtualLibrary;
use crate::search::execute::execute_query;
use sqlx::sqlite::SqlitePool;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BookRow {
    pub id:      String,
    pub title:   String,
    pub authors: String,
}

pub async fn execute_virtual_library(
    pool: &SqlitePool,
    vlib: &VirtualLibrary,
) -> Result<Vec<BookRow>, crate::error::ProcessingError> {
    let ids = execute_query(pool, &vlib.search_expr).await?;

    if ids.is_empty() {
        return Ok(vec![]);
    }

    let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let order = if vlib.sort_asc { "ASC" } else { "DESC" };
    let sort_col = sanitize_sort_field(&vlib.sort_field);
    let sql = format!(
        "SELECT id, title, authors_json as authors FROM local_books WHERE id IN ({placeholders}) ORDER BY {sort_col} {order}"
    );

    let mut q = sqlx::query_as::<_, BookRow>(&sql);
    for id in &ids { q = q.bind(id); }
    let rows = q.fetch_all(pool).await
        .map_err(|e| crate::error::ProcessingError::DbError(e))?;
    Ok(rows)
}

fn sanitize_sort_field(field: &str) -> &str {
    match field {
        "title" | "authors" | "added_at" | "last_opened_at" | "word_count" => field,
        _ => "title",
    }
}
