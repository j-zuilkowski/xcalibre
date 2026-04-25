use crate::error::ProcessingError;
use crate::import::opf;
use crate::pipeline::ingest::detect_format;
use crate::pipeline::local::import_local_book;
use crate::utils::hash::sha256_file;
use sqlx::SqlitePool;
use std::collections::HashSet;
use std::path::Path;
use tracing::{info, warn};

pub struct CalibreImportResult {
    pub found: usize,
    pub imported: usize,
    pub queued: usize,
    pub skipped: usize,
    pub errors: usize,
}

const SUPPORTED_EXTENSIONS: &[&str] = &["epub", "pdf", "mobi", "azw3", "cbz", "cbr", "txt"];

pub async fn import_calibre_library(
    pool: &SqlitePool,
    library_path: &Path,
) -> Result<CalibreImportResult, ProcessingError> {
    let mut found = 0;
    let mut imported = 0;
    let mut queued = 0;
    let mut skipped = 0;
    let mut errors = 0;
    let mut parsed_dirs: HashSet<std::path::PathBuf> = HashSet::new();

    for entry in walkdir::WalkDir::new(library_path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.path();
        let ext = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();
        if !SUPPORTED_EXTENSIONS.contains(&ext.as_str()) {
            continue;
        }
        found += 1;

        if let Some(dir) = path.parent() {
            let dir = dir.to_path_buf();
            if parsed_dirs.insert(dir.clone()) {
                let opf_path = dir.join("metadata.opf");
                if opf_path.exists() {
                    let _ = opf::parse_opf(&opf_path);
                }
            }
        }

        if detect_format(path).is_err() {
            warn!("skipping unsupported/unreadable file: {}", path.display());
            skipped += 1;
            continue;
        }

        let sha = match sha256_file(path) {
            Ok(sha) => sha,
            Err(err) => {
                warn!("hash failed for {}: {}", path.display(), err);
                skipped += 1;
                continue;
            }
        };

        let existing: Option<(String,)> = sqlx::query_as("SELECT id FROM jobs WHERE file_sha256 = ?")
            .bind(&sha)
            .fetch_optional(pool)
            .await
            .map_err(ProcessingError::DbError)?;

        if existing.is_some() {
            skipped += 1;
            continue;
        }

        match import_local_book(pool, path).await {
            Ok(_) => {
                info!("imported: {}", path.display());
                imported += 1;
                queued += 1;
            }
            Err(err) => {
                warn!("ingest error for {}: {}", path.display(), err);
                errors += 1;
            }
        }
    }

    Ok(CalibreImportResult {
        found,
        imported,
        queued,
        skipped,
        errors,
    })
}
