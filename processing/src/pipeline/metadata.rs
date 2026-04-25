use crate::error::ProcessingError;
use crate::metadata::{self, BookMetadata};
use crate::pipeline::ingest::IngestResult;
use crate::plugins::DetectedFormat;
use sqlx::SqlitePool;
use std::path::Path;
use tracing::info;

pub async fn run_metadata(
    pool: &SqlitePool,
    result: &IngestResult,
    path: &Path,
) -> Result<BookMetadata, ProcessingError> {
    let meta = match result.format {
        DetectedFormat::Epub => metadata::epub::extract(path)?,
        DetectedFormat::Pdf => metadata::pdf::extract(path)?,
        DetectedFormat::Mobi | DetectedFormat::Azw3 => metadata::mobi::extract(path)?,
        DetectedFormat::Azw4 => metadata::azw4::extract(path)?,
        DetectedFormat::Fb2 => metadata::fb2::extract(path)?,
        DetectedFormat::Html | DetectedFormat::Htmlz => metadata::html::extract(path)?,
        DetectedFormat::Rtf => metadata::rtf::extract(path)?,
        DetectedFormat::Docx => metadata::docx::extract(path)?,
        DetectedFormat::Odt => metadata::odt::extract(path)?,
        DetectedFormat::Chm => metadata::chm::extract(path)?,
        DetectedFormat::Lrf | DetectedFormat::Lrx => metadata::lrf::extract(path)?,
        DetectedFormat::Pdb | DetectedFormat::Pml | DetectedFormat::Rb => metadata::pdb::extract(path)?,
        DetectedFormat::Snb => metadata::snb::extract(path)?,
        DetectedFormat::Tcr => metadata::tcr::extract(path)?,
        DetectedFormat::Djvu => metadata::djvu::extract(path)?,
        DetectedFormat::Lit => metadata::lit::extract(path)?,
        DetectedFormat::Cbz | DetectedFormat::Cbr => metadata::cbz::extract(path)?,
        DetectedFormat::Txt => BookMetadata::default(),
    };

    let json = serde_json::to_string(&meta)
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO job_metadata (job_id, metadata_json, extracted_at)
         VALUES (?, ?, ?)
         ON CONFLICT(job_id) DO UPDATE
         SET metadata_json = excluded.metadata_json,
             extracted_at  = excluded.extracted_at",
    )
    .bind(&result.job_id)
    .bind(&json)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    info!(job_id = %result.job_id, "metadata extracted");
    Ok(meta)
}
