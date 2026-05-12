use crate::error::ProcessingError;
use crate::pipeline::ingest::IngestResult;
use crate::plugins::DetectedFormat;
use sqlx::SqlitePool;
use std::path::Path;
use tracing::info;

pub async fn run_text(
    pool: &SqlitePool,
    result: &IngestResult,
    path: &Path,
) -> Result<(), ProcessingError> {
    let extracted = match result.format {
        DetectedFormat::Epub => crate::text::epub::extract(path)?,
        DetectedFormat::Pdf => crate::text::pdf::extract(path)?,
        DetectedFormat::Mobi | DetectedFormat::Azw3 => crate::text::mobi::extract(path)?,
        DetectedFormat::Azw4 => crate::text::azw4::extract(path)?,
        DetectedFormat::Fb2 => crate::text::fb2::extract(path)?,
        DetectedFormat::Html | DetectedFormat::Htmlz => crate::text::html::extract(path)?,
        DetectedFormat::Rtf => crate::text::rtf::extract(path)?,
        DetectedFormat::Docx => crate::text::docx::extract(path)?,
        DetectedFormat::Odt => crate::text::odt::extract(path)?,
        #[cfg(not(target_os = "windows"))]
        DetectedFormat::Chm => crate::text::chm::extract(path)?,
        #[cfg(target_os = "windows")]
        DetectedFormat::Chm => crate::text::ExtractedText { full_text: String::new(), word_count: 0 },
        DetectedFormat::Lrf | DetectedFormat::Lrx => crate::text::lrf::extract(path)?,
        DetectedFormat::Pdb | DetectedFormat::Pml | DetectedFormat::Rb => crate::text::pdb::extract(path)?,
        DetectedFormat::Snb => crate::text::snb::extract(path)?,
        DetectedFormat::Tcr => crate::text::tcr::extract(path)?,
        DetectedFormat::Djvu => crate::text::djvu::extract(path)?,
        DetectedFormat::Lit => crate::text::lit::extract(path)?,
        DetectedFormat::Kfx => crate::text::kfx::extract(path)?,
        DetectedFormat::Txt => crate::text::txt::extract(path)?,
        DetectedFormat::Cbz | DetectedFormat::Cbr => return Ok(()),
    };

    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO job_text (job_id, full_text, word_count, user_edited, updated_at)
         VALUES (?, ?, ?, 0, ?)
         ON CONFLICT(job_id) DO UPDATE
         SET full_text   = excluded.full_text,
             word_count  = excluded.word_count,
             updated_at  = excluded.updated_at",
    )
    .bind(&result.job_id)
    .bind(&extracted.full_text)
    .bind(extracted.word_count as i64)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    // Run RAG chunking after text extraction (non-fatal)
    if let Err(e) = crate::pipeline::rag::run_rag_chunk(pool, &result.job_id, &extracted.full_text).await {
        tracing::warn!("RAG chunk failed (non-fatal): {}", e);
    }

    info!(job_id = %result.job_id, words = extracted.word_count, "text extracted");
    Ok(())
}
