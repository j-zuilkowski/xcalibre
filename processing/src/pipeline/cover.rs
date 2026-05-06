use crate::cover::{epub, resize};
use crate::error::ProcessingError;
use crate::pipeline::ingest::IngestResult;
use crate::plugins::DetectedFormat;
use sqlx::SqlitePool;
use std::path::Path;
use tracing::info;

pub async fn run_cover(
    pool: &SqlitePool,
    result: &IngestResult,
    path: &Path,
) -> Result<Option<std::path::PathBuf>, ProcessingError> {
    let raw = match result.format {
        DetectedFormat::Epub => epub::extract(path)?,
        DetectedFormat::Pdf => crate::cover::pdf::extract(path)?,
        DetectedFormat::Cbz => crate::cover::cbz::extract(path)?,
        DetectedFormat::Cbr => crate::cover::cbz::extract(path)?,  // CBR: returns Ok(None) internally
        _ => return Ok(None),
    };

    let raw = match raw {
        Some(r) => r,
        None    => return Ok(None),
    };

    let resized    = resize::resize_cover(raw)?;
    let cover_path = path.with_extension("cover.jpg");
    std::fs::write(&cover_path, &resized.data).map_err(ProcessingError::IoError)?;

    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("UPDATE local_books SET cover_path = ?, updated_at = ? WHERE id = ?")
        .bind(cover_path.to_string_lossy().as_ref())
        .bind(&now)
        .bind(&result.job_id)
        .execute(pool)
        .await
        .map_err(ProcessingError::DbError)?;

    info!(job_id = %result.job_id, "cover saved");
    Ok(Some(cover_path))
}
