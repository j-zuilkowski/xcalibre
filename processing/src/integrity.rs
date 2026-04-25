use crate::error::ProcessingError;
use sqlx::SqlitePool;

#[derive(Debug, Clone, serde::Serialize)]
pub struct IntegrityIssue {
    pub book_id:   String,
    pub file_path: String,
    pub issue:     String,
}

/// Scan all local_books records and verify files exist on disk.
pub async fn check_integrity(pool: &SqlitePool) -> Result<Vec<IntegrityIssue>, ProcessingError> {
    let rows = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT id, local_path FROM local_books WHERE local_path IS NOT NULL",
    )
    .fetch_all(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    let mut issues = vec![];
    for (id, path) in rows {
        let Some(path) = path else {
            continue;
        };
        let p = std::path::Path::new(&path);
        if !p.exists() {
            issues.push(IntegrityIssue {
                book_id: id,
                file_path: path,
                issue: "file_missing".to_string(),
            });
        }
    }
    Ok(issues)
}
