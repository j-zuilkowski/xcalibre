use crate::error::ProcessingError;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use std::io::Write;
use std::path::Path;
use zip::write::{FileOptions, ZipWriter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupOptions {
    pub include_files: bool,
    pub compress:      bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupManifest {
    pub xcalibre_version: String,
    pub created_at:       String,
    pub book_count:       u32,
    pub include_files:    bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
struct BookRecord {
    id:           String,
    title:        String,
    authors_json: String,
    format:       String,
    local_path:   Option<String>,
    cover_path:   Option<String>,
    series_name:  Option<String>,
    series_index: Option<f64>,
    description:  Option<String>,
    publisher:    Option<String>,
    pubdate:      Option<String>,
    library_id:   Option<String>,
    created_at:   String,
    updated_at:   String,
}

pub async fn export_library_backup(
    pool: &SqlitePool,
    library_root: &Path,
    out_path: &Path,
    opts: &BackupOptions,
) -> Result<(), ProcessingError> {
    let compression = if opts.compress {
        zip::CompressionMethod::Deflated
    } else {
        zip::CompressionMethod::Stored
    };
    let file_opts = FileOptions::default().compression_method(compression);

    let file = std::fs::File::create(out_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    let mut zip = ZipWriter::new(file);

    // Export books as JSON
    let books = sqlx::query_as::<_, BookRecord>(
        "SELECT id, title, authors_json, format, local_path, cover_path,
                series_name, series_index, description, publisher, pubdate,
                library_id, created_at, updated_at
         FROM local_books"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    let book_count = books.len() as u32;
    let books_json = serde_json::to_string_pretty(&books)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    zip.start_file("library.db", file_opts)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    zip.write_all(books_json.as_bytes())
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    // Manifest JSON
    let manifest = BackupManifest {
        xcalibre_version: env!("CARGO_PKG_VERSION").to_string(),
        created_at:       chrono::Utc::now().to_rfc3339(),
        book_count,
        include_files:    opts.include_files,
    };
    let manifest_json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    zip.start_file("manifest.json", file_opts)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    zip.write_all(manifest_json.as_bytes())
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    // Optionally include book files
    if opts.include_files && library_root.exists() {
        for entry in walkdir::WalkDir::new(library_root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let rel = entry.path().strip_prefix(library_root).unwrap();
            let arc_name = format!("files/{}", rel.display());
            if let Ok(bytes) = std::fs::read(entry.path()) {
                zip.start_file(&arc_name, file_opts)
                    .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
                zip.write_all(&bytes)
                    .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
            }
        }
    }

    zip.finish()
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    Ok(())
}

pub async fn restore_library_backup(
    backup_path: &Path,
    restore_dir: &Path,
    pool: &SqlitePool,
) -> Result<u32, ProcessingError> {
    let file = std::fs::File::open(backup_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    // Read books JSON from library.db entry
    let books: Vec<BookRecord> = {
        let mut entry = archive.by_name("library.db")
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
        let mut json = String::new();
        std::io::Read::read_to_string(&mut entry, &mut json)?;
        serde_json::from_str(&json)
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?
    };

    let count = books.len() as u32;
    for book in &books {
        sqlx::query(
            "INSERT OR IGNORE INTO local_books
             (id, title, authors_json, format, local_path, cover_path, series_name,
              series_index, description, publisher, pubdate, library_id, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&book.id)
        .bind(&book.title)
        .bind(&book.authors_json)
        .bind(&book.format)
        .bind(&book.local_path)
        .bind(&book.cover_path)
        .bind(&book.series_name)
        .bind(book.series_index)
        .bind(&book.description)
        .bind(&book.publisher)
        .bind(&book.pubdate)
        .bind(&book.library_id)
        .bind(&book.created_at)
        .bind(&book.updated_at)
        .execute(pool)
        .await
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    }

    // Extract files if present
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
        let name = entry.name().to_string();
        if name.starts_with("files/") {
            let rel = name.trim_start_matches("files/");
            let dest = restore_dir.join(rel);
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            if let Ok(mut f) = std::fs::File::create(&dest) {
                std::io::copy(&mut entry, &mut f).ok();
            }
        }
    }

    Ok(count)
}
