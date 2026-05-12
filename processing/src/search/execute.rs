use super::QueryNode;
use crate::error::ProcessingError;
use sqlx::SqlitePool;
use std::future::Future;
use std::pin::Pin;

/// Execute a parsed query and return matching book IDs.
pub async fn execute_query(pool: &SqlitePool, query: &str) -> Result<Vec<String>, ProcessingError> {
    let node = super::parse_query(query)?;
    let ids = collect_ids(pool, &node).await?;
    Ok(ids)
}

fn collect_ids<'a>(
    pool: &'a SqlitePool,
    node: &'a QueryNode,
) -> Pin<Box<dyn Future<Output = Result<Vec<String>, ProcessingError>> + Send + 'a>> {
    Box::pin(async move {
        match node {
            QueryNode::FtsTerm(term) => {
                fts_search(pool, term).await
            }
            QueryNode::Field { field, value, negate } => {
                let ids = field_search(pool, field, value).await?;
                if *negate {
                    let all = all_ids(pool).await?;
                    Ok(all.into_iter().filter(|id| !ids.contains(id)).collect())
                } else {
                    Ok(ids)
                }
            }
            QueryNode::And(left, right) => {
                let l = collect_ids(pool, left).await?;
                let r = collect_ids(pool, right).await?;
                Ok(l.into_iter().filter(|id| r.contains(id)).collect())
            }
            QueryNode::Or(left, right) => {
                let mut l = collect_ids(pool, left).await?;
                let r = collect_ids(pool, right).await?;
                for id in r { if !l.contains(&id) { l.push(id); } }
                Ok(l)
            }
            QueryNode::Not(inner) => {
                let excluded = collect_ids(pool, inner).await?;
                let all = all_ids(pool).await?;
                Ok(all.into_iter().filter(|id| !excluded.contains(id)).collect())
            }
        }
    })
}

async fn fts_search(pool: &SqlitePool, term: &str) -> Result<Vec<String>, ProcessingError> {
    // Use FTS5 books_fts table (created in migration 0005_fts.sql)
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT b.id FROM local_books b
         JOIN books_fts f ON f.rowid = b.rowid
         WHERE books_fts MATCH ?
         ORDER BY rank",
    )
    .bind(term)
    .fetch_all(pool).await.map_err(ProcessingError::DbError)?;
    Ok(rows.into_iter().map(|(id,)| id).collect())
}

async fn field_search(pool: &SqlitePool, field: &str, value: &str) -> Result<Vec<String>, ProcessingError> {
    let like = format!("%{}%", value.to_lowercase());
    let sql = match field {
        "title"     => "SELECT id FROM local_books WHERE lower(title) LIKE ?",
        "author"    => "SELECT id FROM local_books WHERE lower(authors_json) LIKE ?",
        "tag"       => "SELECT DISTINCT bt.book_id FROM book_tags bt JOIN tags t ON t.id = bt.tag_id WHERE lower(t.name) LIKE ?",
        "series"    => "SELECT id FROM local_books WHERE lower(COALESCE(series_name,'')) LIKE ?",
        "format"    => "SELECT id FROM local_books WHERE lower(format) = lower(?)",
        "publisher" => "SELECT id FROM local_books WHERE lower(COALESCE(publisher,'')) LIKE ?",
        "language"  => "SELECT id FROM local_books WHERE 1=0",  // no language column available
        _           => return Ok(vec![]),
    };
    // format:VALUE uses exact match not LIKE
    let param = if field == "format" { value.to_string() } else { like };
    let rows: Vec<(String,)> = sqlx::query_as(sql)
        .bind(&param)
        .fetch_all(pool).await.map_err(ProcessingError::DbError)?;
    Ok(rows.into_iter().map(|(id,)| id).collect())
}

async fn all_ids(pool: &SqlitePool) -> Result<Vec<String>, ProcessingError> {
    let rows: Vec<(String,)> = sqlx::query_as("SELECT id FROM local_books")
        .fetch_all(pool).await.map_err(ProcessingError::DbError)?;
    Ok(rows.into_iter().map(|(id,)| id).collect())
}
