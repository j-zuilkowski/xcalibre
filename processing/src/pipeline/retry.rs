use crate::error::ProcessingError;
use sqlx::SqlitePool;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{info, warn};
use xcalibre_api::client::ApiClient;

fn next_retry_delay_minutes(retry_count: i64) -> i64 {
    2_i64.pow(retry_count.min(6) as u32).min(60)
}

pub async fn start_retry_worker(pool: Arc<SqlitePool>, client: Option<Arc<ApiClient>>) {
    let mut ticker = interval(Duration::from_secs(60));
    loop {
        ticker.tick().await;
        if let Err(e) = run_retries(&pool, client.as_deref()).await {
            warn!("retry worker error: {}", e);
        }
    }
}

pub async fn run_retries(
    pool: &SqlitePool,
    client: Option<&ApiClient>,
) -> Result<(), ProcessingError> {
    let Some(client) = client else {
        return Ok(());
    };

    let jobs: Vec<(String, String, String, i64)> = sqlx::query_as(
        "SELECT id, format, file_sha256, retry_count FROM jobs
         WHERE status = 'RETRYING'
           AND (next_retry_at IS NULL OR next_retry_at <= datetime('now'))",
    )
    .fetch_all(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    for (id, _format, _sha256, retry_count) in jobs {
        let job = sqlx::query_as::<_, (String, String, String, String, i64, Option<String>, Option<String>, i64, Option<String>, String, String, String)>(
            "SELECT id, file_path, file_sha256, format, status, retry_count,
                    next_retry_at, xs_book_id, push_step, error_message,
                    created_at, updated_at
             FROM jobs WHERE id = ?",
        )
        .bind(&id)
        .fetch_optional(pool)
        .await
        .map_err(ProcessingError::DbError)?
        .ok_or_else(|| ProcessingError::DbError(sqlx::Error::RowNotFound))?;
        let (_, file_path, _, _, _, _, _, _, _, _, _, _) = job;
        let file_path = PathBuf::from(file_path);
        let file_name = file_path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| ProcessingError::MetadataError("job file path has no file name".into()))?
            .to_string();

        let meta: serde_json::Value = sqlx::query_as::<_, (String,)>(
            "SELECT metadata_json FROM job_metadata WHERE job_id = ?",
        )
        .bind(&id)
        .fetch_optional(pool)
        .await
        .map_err(ProcessingError::DbError)?
        .and_then(|(s,)| serde_json::from_str(&s).ok())
        .unwrap_or(serde_json::Value::Null);

        let payload = xcalibre_api::push::PushPayload {
            file_path,
            file_name,
            metadata: meta,
        };

        match xcalibre_api::push::push_book(client, &payload).await {
            Ok(resp) => {
                sqlx::query(
                    "UPDATE jobs SET status='COMPLETED', xs_book_id=?, updated_at=datetime() WHERE id=?",
                )
                .bind(&resp.id)
                .bind(&id)
                .execute(pool)
                .await
                .map_err(ProcessingError::DbError)?;
                info!(job_id = %id, book_id = %resp.id, "retry push complete");
            }
            Err(e) => {
                warn!(job_id = %id, error = %e, "retry push failed");
                let new_count = retry_count + 1;
                if new_count >= 5 {
                    sqlx::query(
                        "UPDATE jobs SET status='FAILED', updated_at=datetime() WHERE id=?",
                    )
                    .bind(&id)
                    .execute(pool)
                    .await
                    .map_err(ProcessingError::DbError)?;
                } else {
                    let delay = next_retry_delay_minutes(retry_count);
                    let next = chrono::Utc::now() + chrono::Duration::minutes(delay);
                    sqlx::query(
                        "UPDATE jobs SET retry_count=?, next_retry_at=?, updated_at=datetime() WHERE id=?",
                    )
                    .bind(new_count)
                    .bind(next.to_rfc3339())
                    .bind(&id)
                    .execute(pool)
                    .await
                    .map_err(ProcessingError::DbError)?;
                }
            }
        }
    }
    Ok(())
}
