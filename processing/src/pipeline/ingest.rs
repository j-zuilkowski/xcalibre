use crate::error::ProcessingError;
use crate::plugins::DetectedFormat;
use crate::utils::hash::sha256_file;
use sqlx::SqlitePool;
use std::fs::File;
use std::io::{BufReader, Read, Seek};
use std::path::Path;

#[derive(Debug)]
pub struct IngestResult {
    pub job_id: String,
    pub format: DetectedFormat,
    pub sha256: String,
    pub file_size: u64,
}

pub fn detect_format(path: &Path) -> Result<DetectedFormat, ProcessingError> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let mut file = File::open(path).map_err(ProcessingError::IoError)?;
    let mut header = [0u8; 4096];
    let n = file.read(&mut header).map_err(ProcessingError::IoError)?;
    let header = &header[..n];

    if header.starts_with(b"PK\x03\x04") {
        if is_epub(path) {
            return Ok(DetectedFormat::Epub);
        }
        if is_docx(path) {
            return Ok(DetectedFormat::Docx);
        }
        if is_odt(path) {
            return Ok(DetectedFormat::Odt);
        }
        if is_htmlz(path) {
            return Ok(DetectedFormat::Htmlz);
        }
        return Ok(DetectedFormat::Cbz);
    }

    if header.starts_with(b"%PDF-") {
        return Ok(DetectedFormat::Pdf);
    }

    if header.len() >= 68 && &header[60..68] == b"BOOKMOBI" {
        return match ext.as_str() {
            "azw3" => Ok(DetectedFormat::Azw3),
            "azw4" => Ok(DetectedFormat::Azw4),
            _ => Ok(DetectedFormat::Mobi),
        };
    }

    if header.starts_with(b"{\\rtf") {
        return Ok(DetectedFormat::Rtf);
    }

    if looks_like_fb2(header) {
        return Ok(DetectedFormat::Fb2);
    }

    if looks_like_html(header) {
        return Ok(DetectedFormat::Html);
    }

    if header.starts_with(b"AT&TFORM") {
        return Ok(DetectedFormat::Djvu);
    }

    if header.starts_with(b"ITSF") {
        return Ok(DetectedFormat::Chm);
    }

    if header.starts_with(b"ITOLITLS") {
        return Ok(DetectedFormat::Lit);
    }

    if header.starts_with(b"SNBP") {
        return Ok(DetectedFormat::Snb);
    }

    if header.starts_with(b"L\0R\0F\0") {
        return Ok(DetectedFormat::Lrf);
    }

    if header.starts_with(b"Rar!\x1a\x07\x00") || header.starts_with(b"Rar!\x1a\x07\x01\x00") {
        return Ok(DetectedFormat::Cbr);
    }

    match ext.as_str() {
        "docx" => return Ok(DetectedFormat::Docx),
        "odt" => return Ok(DetectedFormat::Odt),
        "chm" => return Ok(DetectedFormat::Chm),
        "lrf" => return Ok(DetectedFormat::Lrf),
        "lrx" => return Ok(DetectedFormat::Lrx),
        "pdb" => return Ok(DetectedFormat::Pdb),
        "pml" => return Ok(DetectedFormat::Pml),
        "rb" => return Ok(DetectedFormat::Rb),
        "snb" => return Ok(DetectedFormat::Snb),
        "tcr" => return Ok(DetectedFormat::Tcr),
        "azw3" => return Ok(DetectedFormat::Azw3),
        "azw4" => return Ok(DetectedFormat::Azw4),
        "djvu" => return Ok(DetectedFormat::Djvu),
        "lit" => return Ok(DetectedFormat::Lit),
        "cbz" => return Ok(DetectedFormat::Cbz),
        "cbr" => return Ok(DetectedFormat::Cbr),
        _ => {}
    }

    if looks_like_text(header) {
        return Ok(DetectedFormat::Txt);
    }

    Err(ProcessingError::UnsupportedFormat(format!(
        "unsupported file format for {:?}",
        path
    )))
}

fn is_epub(path: &Path) -> bool {
    // Try to open as ZIP and check for mimetype entry
    match std::fs::File::open(path) {
        Ok(file) => {
            let mut archive = match zip::ZipArchive::new(BufReader::new(file)) {
                Ok(archive) => archive,
                Err(_) => return false,
            };
            
            // Look for the mimetype entry
            let result = match archive.by_name("mimetype") {
                Ok(mut file) => {
                    let mut mimetype = String::new();
                    if file.read_to_string(&mut mimetype).is_ok() {
                        mimetype.trim().starts_with("application/epub")
                    } else {
                        false
                    }
                },
                Err(_) => false,
            };
            result
        },
        Err(_) => false,
    }
}

fn is_docx(path: &Path) -> bool {
    zip_has_entries(path, &["docProps/core.xml", "word/document.xml"])
}

fn is_odt(path: &Path) -> bool {
    if zip_has_entries(path, &["content.xml", "meta.xml"]) {
        return true;
    }

    match std::fs::File::open(path) {
        Ok(file) => match zip::ZipArchive::new(BufReader::new(file)) {
            Ok(mut archive) => {
                if let Ok(mut entry) = archive.by_name("mimetype") {
                    let mut mimetype = String::new();
                    entry.read_to_string(&mut mimetype).is_ok()
                        && mimetype.trim() == "application/vnd.oasis.opendocument.text"
                } else {
                    false
                }
            }
            Err(_) => false,
        },
        Err(_) => false,
    }
}

fn is_htmlz(path: &Path) -> bool {
    match std::fs::File::open(path) {
        Ok(file) => {
            let mut archive = match zip::ZipArchive::new(BufReader::new(file)) {
                Ok(archive) => archive,
                Err(_) => return false,
            };
            let has_index = archive.by_name("index.html").is_ok();
            has_index
        }
        Err(_) => false,
    }
}

fn zip_has_entries(path: &Path, entries: &[&str]) -> bool {
    match std::fs::File::open(path) {
        Ok(file) => match zip::ZipArchive::new(BufReader::new(file)) {
            Ok(mut archive) => entries.iter().all(|name| archive.by_name(name).is_ok()),
            Err(_) => false,
        },
        Err(_) => false,
    }
}

fn looks_like_fb2(header: &[u8]) -> bool {
    if !header.starts_with(b"<?xml") && !header.starts_with(b"<Fict") {
        return false;
    }
    let head = match std::str::from_utf8(header) {
        Ok(s) => s,
        Err(_) => return false,
    };
    head.contains("FictionBook")
}

fn looks_like_html(header: &[u8]) -> bool {
    let head = match std::str::from_utf8(header) {
        Ok(s) => s.trim_start(),
        Err(_) => return false,
    };
    head.starts_with("<html")
        || head.starts_with("<HTML")
        || head.starts_with("<!DOCTYPE")
        || head.starts_with("<!doctype")
}

fn looks_like_text(header: &[u8]) -> bool {
    // Check for null bytes
    if header.contains(&0x00) {
        return false;
    }
    
    // Count printable characters
    let printable_count = header.iter()
        .filter(|&&b| {
            b == 0x09 || // tab
            b == 0x0A || // newline
            b == 0x0D || // carriage return
            (0x20..=0x7E).contains(&b) || // printable ASCII
            b >= 0x80 // valid UTF-8 continuation bytes
        })
        .count();
    
    // At least 50% should be printable characters
    printable_count >= header.len() / 2
}

pub fn validate_integrity(path: &Path, format: &DetectedFormat) -> Result<(), ProcessingError> {
    let file_size = std::fs::metadata(path)
        .map_err(ProcessingError::IoError)?
        .len();

    match format {
        DetectedFormat::Epub | DetectedFormat::Cbz | DetectedFormat::Htmlz | DetectedFormat::Docx | DetectedFormat::Odt => {
            // Try to open as ZIP
            let file = std::fs::File::open(path)
                .map_err(ProcessingError::IoError)?;
            let _archive = zip::ZipArchive::new(BufReader::new(file))
                .map_err(|_| ProcessingError::IntegrityError("Invalid ZIP archive".to_string()))?;
        },
        DetectedFormat::Pdf => {
            // Check PDF header and EOF marker
            let mut file = File::open(path)
                .map_err(ProcessingError::IoError)?;

            let mut header = [0u8; 5];
            file.read_exact(&mut header)
                .map_err(ProcessingError::IoError)?;

            if &header != b"%PDF-" {
                return Err(ProcessingError::IntegrityError("Invalid PDF header".to_string()));
            }

            // Check for EOF marker in last 1024 bytes (or whole file if smaller)
            let tail_offset = (file_size as i64).min(1024);
            file.seek(std::io::SeekFrom::End(-tail_offset))
                .map_err(ProcessingError::IoError)?;
            let mut buffer = Vec::with_capacity(tail_offset as usize);
            file.read_to_end(&mut buffer).map_err(ProcessingError::IoError)?;

            if !buffer.windows(5).any(|window| window == b"%%EOF") {
                return Err(ProcessingError::IntegrityError("PDF missing EOF marker".to_string()));
            }
        },
        DetectedFormat::Mobi | DetectedFormat::Azw3 => {
            // Check BOOKMOBI header
            let mut file = File::open(path)
                .map_err(ProcessingError::IoError)?;

            let mut header = [0u8; 68];
            file.read_exact(&mut header)
                .map_err(ProcessingError::IoError)?;
            
            if &header[60..68] != b"BOOKMOBI" {
                return Err(ProcessingError::IntegrityError("Invalid MOBI header".to_string()));
            }
            
            // Check minimum file size
            if file_size < 132 {
                return Err(ProcessingError::IntegrityError("MOBI file too small".to_string()));
            }
        },
        DetectedFormat::Cbr => {
            // Check RAR header
            let mut file = File::open(path)
                .map_err(ProcessingError::IoError)?;

            let mut header = [0u8; 8];
            file.read_exact(&mut header)
                .map_err(ProcessingError::IoError)?;
            
            if !(header.starts_with(b"Rar!\x1a\x07\x00") || header.starts_with(b"Rar!\x1a\x07\x01\x00")) {
                return Err(ProcessingError::IntegrityError("Invalid CBR header".to_string()));
            }
        },
        DetectedFormat::Txt
        | DetectedFormat::Fb2
        | DetectedFormat::Html
        | DetectedFormat::Rtf
        | DetectedFormat::Azw4
        | DetectedFormat::Chm
        | DetectedFormat::Lrf
        | DetectedFormat::Lrx
        | DetectedFormat::Pdb
        | DetectedFormat::Pml
        | DetectedFormat::Rb
        | DetectedFormat::Snb
        | DetectedFormat::Tcr
        | DetectedFormat::Djvu
        | DetectedFormat::Lit
        | DetectedFormat::Kfx => {}
    }
    
    Ok(())
}

pub async fn run_ingest(
    pool: &SqlitePool,
    path: &Path,
) -> Result<IngestResult, ProcessingError> {
    let path = path.to_path_buf();
    let (file_size, format, sha256) = tokio::task::spawn_blocking({
        let path = path.clone();
        move || -> Result<(u64, DetectedFormat, String), ProcessingError> {
            // Check file exists and is a regular file.
            if !path.is_file() {
                return Err(ProcessingError::IoError(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "File not found",
                )));
            }

            // Check file size (500 MB limit).
            let metadata = std::fs::metadata(&path).map_err(ProcessingError::IoError)?;
            let file_size = metadata.len();
            if file_size > 500 * 1024 * 1024 {
                return Err(ProcessingError::UnsupportedFormat(
                    "File too large (max 500 MB)".to_string(),
                ));
            }

            // Detect format, validate integrity, and calculate SHA-256 off the async worker threads.
            let format = detect_format(&path)?;
            validate_integrity(&path, &format)?;
            let sha256 = sha256_file(&path)?;
            Ok((file_size, format, sha256))
        }
    })
    .await
    .map_err(|_| ProcessingError::IoError(std::io::Error::other("blocking ingest task failed")))??;
    
    // Check if this file has already been imported
    let existing_job: Option<(String,)> = sqlx::query_as(
        "SELECT id FROM jobs WHERE file_sha256 = ? AND status = 'COMPLETED'"
    )
    .bind(&sha256)
    .fetch_optional(pool)
    .await
    .map_err(ProcessingError::DbError)?;
    
    if let Some((job_id,)) = existing_job {
        return Err(ProcessingError::UnsupportedFormat(
            format!("already imported: {}", job_id)
        ));
    }
    
    // Create new job record
    let job_id: String = uuid::Uuid::new_v4().to_string();
    
    sqlx::query(
        "INSERT INTO jobs (id, file_path, file_sha256, format, status, created_at, updated_at) 
         VALUES (?, ?, ?, ?, 'PENDING', datetime(), datetime())"
    )
    .bind(&job_id)
    .bind(path.to_string_lossy().as_ref())
    .bind(&sha256)
    .bind(format.to_string())
    .execute(pool)
    .await
    .map_err(ProcessingError::DbError)?;
    
    Ok(IngestResult {
        job_id,
        format,
        sha256,
        file_size,
    })
}
