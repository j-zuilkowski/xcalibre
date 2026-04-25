use crate::error::ProcessingError;
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConversionJob {
    pub id:            String,
    pub book_id:       String,
    pub source_format: String,
    pub target_format: String,
    pub mode:          String,
    pub output_path:   String,
    pub status:        String,
    pub error_message: Option<String>,
    pub created_at:    String,
    pub updated_at:    String,
}

pub async fn create_job(
    pool: &SqlitePool,
    book_id: &str,
    source_format: &str,
    target_format: &str,
    mode: &str,
    output_path: &str,
) -> Result<String, ProcessingError> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO conversion_jobs
         (id, book_id, source_format, target_format, mode, output_path, status,
          error_message, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, 'PENDING', NULL, ?, ?)",
    )
    .bind(&id)
    .bind(book_id)
    .bind(source_format)
    .bind(target_format)
    .bind(mode)
    .bind(output_path)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(ProcessingError::DbError)?;
    Ok(id)
}

pub async fn update_status(
    pool: &SqlitePool,
    job_id: &str,
    status: &str,
    error_message: Option<&str>,
) -> Result<(), ProcessingError> {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "UPDATE conversion_jobs
         SET status = ?, error_message = ?, updated_at = ?
         WHERE id = ?",
    )
    .bind(status)
    .bind(error_message)
    .bind(&now)
    .bind(job_id)
    .execute(pool)
    .await
    .map_err(ProcessingError::DbError)?;
    Ok(())
}

pub async fn get_job(
    pool: &SqlitePool,
    job_id: &str,
) -> Result<Option<ConversionJob>, ProcessingError> {
    let row = sqlx::query_as::<_, (
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        Option<String>,
        String,
        String,
    )>(
        "SELECT id, book_id, source_format, target_format, mode, output_path,
                status, error_message, created_at, updated_at
         FROM conversion_jobs
         WHERE id = ?",
    )
    .bind(job_id)
    .fetch_optional(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    Ok(row.map(
        |(
            id,
            book_id,
            source_format,
            target_format,
            mode,
            output_path,
            status,
            error_message,
            created_at,
            updated_at,
        )| ConversionJob {
            id,
            book_id,
            source_format,
            target_format,
            mode,
            output_path,
            status,
            error_message,
            created_at,
            updated_at,
        },
    ))
}
