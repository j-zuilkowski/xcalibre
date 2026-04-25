use crate::db::queries;
use crate::error::ProcessingError;
use crate::pipeline::ingest::IngestResult;
use sqlx::SqlitePool;
use std::path::PathBuf;
use tracing::{info, warn};
use xcalibre_api::{client::ApiClient, push};

const MAX_RETRIES: i64 = 5;

pub async fn run_push(
    pool: &SqlitePool,
    result: &IngestResult,
    metadata_json: serde_json::Value,
    client: Option<&ApiClient>,
) -> Result<(), ProcessingError> {
    let client = match client {
        Some(c) => c,
        None    => {
            info!(job_id = %result.job_id, "no API client — skipping push");
            return Ok(());
        }
    };

    let job = queries::get_job(pool, &result.job_id)
        .await?
        .ok_or_else(|| ProcessingError::DbError(sqlx::Error::RowNotFound))?;
    let file_path = PathBuf::from(&job.file_path);
    let file_name = file_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| ProcessingError::MetadataError("job file path has no file name".into()))?
        .to_string();

    let payload = push::PushPayload {
        file_path,
        file_name,
        metadata: metadata_json,
    };

    match push::push_book(client, &payload).await {
        Ok(resp) => {
            queries::update_job_status(pool, &result.job_id, "COMPLETED").await?;
            sqlx::query(
                "UPDATE jobs SET xs_book_id = ?, updated_at = datetime() WHERE id = ?",
            )
            .bind(&resp.id)
            .bind(&result.job_id)
            .execute(pool)
            .await
            .map_err(ProcessingError::DbError)?;
            info!(job_id = %result.job_id, book_id = %resp.id, "push complete");
        }
        Err(e) => {
            warn!(job_id = %result.job_id, error = %e, "push failed");
            let job = queries::get_job(pool, &result.job_id)
                .await?
                .ok_or_else(|| ProcessingError::DbError(sqlx::Error::RowNotFound))?;
            if job.retry_count >= MAX_RETRIES {
                queries::update_job_status(pool, &result.job_id, "FAILED").await?;
            } else {
                queries::update_job_status(pool, &result.job_id, "RETRYING").await?;
                let next = chrono::Utc::now()
                    + chrono::Duration::minutes(5 * (job.retry_count + 1));
                sqlx::query(
                    "UPDATE jobs SET retry_count = retry_count + 1, next_retry_at = ?, updated_at = datetime() WHERE id = ?",
                )
                .bind(next.to_rfc3339())
                .bind(&result.job_id)
                .execute(pool)
                .await
                .map_err(ProcessingError::DbError)?;
            }
        }
    }
    Ok(())
}
