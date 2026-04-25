use crate::db::enrichment_cache;
use crate::error::ProcessingError;
use crate::metadata::{self, enrichment as enrich, BookMetadata};
use crate::pipeline::ingest::IngestResult;
use crate::plugins::DetectedFormat;
use sqlx::SqlitePool;
use std::path::Path;
use xcalibre_api::enrichment::{google_books, open_library};

pub async fn run_enrichment(
    pool: &SqlitePool,
    result: &IngestResult,
    path: &Path,
    meta: &BookMetadata,
) -> Result<Vec<enrich::EnrichmentSuggestion>, ProcessingError> {
    let isbn = match result.format {
        DetectedFormat::Epub => metadata::isbn::from_epub(path),
        DetectedFormat::Pdf  => metadata::isbn::from_pdf(path),
        _                    => None,
    };
    let isbn = match isbn {
        Some(i) => i,
        None    => return Ok(vec![]),
    };

    let (ol_json, gb_json) = match enrichment_cache::get_cache(pool, &isbn).await? {
        Some(cached) => cached,
        None => {
            let (ol, gb) = tokio::join!(
                open_library::lookup_by_isbn(&isbn),
                google_books::lookup_by_isbn(&isbn),
            );
            let ol_json = serde_json::to_string(&ol)
                .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;
            let gb_json = serde_json::to_string(&gb)
                .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;
            enrichment_cache::set_cache(pool, &isbn, &ol_json, &gb_json).await?;
            (ol_json, gb_json)
        }
    };

    let ol: open_library::OLBook = serde_json::from_str(&ol_json).unwrap_or_default();
    let gb: google_books::GBBook = serde_json::from_str(&gb_json).unwrap_or_default();
    Ok(enrich::build_suggestions(meta, &ol, &gb))
}