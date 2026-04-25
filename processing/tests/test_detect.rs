use xcalibre_processing::db::queries::{self, Job};
use xcalibre_processing::error::ProcessingError;
use xcalibre_processing::plugins::DetectedFormat;
use xcalibre_processing::pipeline::ingest::{detect_format, validate_integrity, run_ingest, IngestResult};
use xcalibre_processing::utils::hash::sha256_file;
use sqlx::{Pool, Sqlite, sqlite::SqlitePoolOptions};
use std::path::PathBuf;

async fn setup_db() -> Pool<Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!("src/db/migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

#[tokio::test]
async fn test_detect_epub() {
    let path = PathBuf::from("tests/fixtures/fixture_epub.epub");
    let format = detect_format(&path).expect("Failed to detect format");
    assert_eq!(format, DetectedFormat::Epub);
}

#[tokio::test]
async fn test_detect_pdf() {
    let path = PathBuf::from("tests/fixtures/fixture_pdf.pdf");
    let format = detect_format(&path).expect("Failed to detect format");
    assert_eq!(format, DetectedFormat::Pdf);
}

#[tokio::test]
async fn test_detect_cbz() {
    let path = PathBuf::from("tests/fixtures/fixture_cbz.cbz");
    let format = detect_format(&path).expect("Failed to detect format");
    assert_eq!(format, DetectedFormat::Cbz);
}

#[tokio::test]
async fn test_epub_not_cbz() {
    let path = PathBuf::from("tests/fixtures/fixture_cbz.cbz");
    let format = detect_format(&path).expect("Failed to detect format");
    assert_eq!(format, DetectedFormat::Cbz);
}

#[tokio::test]
async fn test_integrity_epub_valid() {
    let path = PathBuf::from("tests/fixtures/fixture_epub.epub");
    let format = detect_format(&path).expect("Failed to detect format");
    let result = validate_integrity(&path, &format);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_ingest_creates_job() {
    let pool = setup_db().await;
    let path = PathBuf::from("tests/fixtures/fixture_epub.epub");

    let result: IngestResult = run_ingest(&pool, &path).await.expect("Failed to run ingest");

    assert!(!result.job_id.is_empty());
    assert_eq!(result.format, DetectedFormat::Epub);
    let expected_sha256 = sha256_file(&path).expect("Failed to compute SHA-256");
    assert_eq!(result.sha256, expected_sha256);

    let job: Option<Job> = queries::get_job(&pool, &result.job_id)
        .await
        .expect("Failed to get job from database");
    let job = job.expect("Job not found");
    assert_eq!(job.id, result.job_id);
    assert_eq!(job.file_path, path.display().to_string());
    assert_eq!(job.file_sha256, result.sha256);
    assert_eq!(job.format, "EPUB");
    assert_eq!(job.status, "PENDING");
}

#[tokio::test]
async fn test_detect_unknown_format_errors() {
    // A file with no recognisable magic bytes must return UnsupportedFormat.
    // We'll write a small all-zero temp file to guarantee this.
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("zero.bin");
    std::fs::write(&path, b"\x00\x00\x00\x00\x00\x00\x00\x00").expect("write");
    let result = detect_format(&path);
    assert!(matches!(result, Err(ProcessingError::UnsupportedFormat(_))));
}

#[tokio::test]
async fn test_integrity_invalid_zip_fails() {
    // Force validate_integrity with Epub format on a non-ZIP file.
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("bad.epub");
    std::fs::write(&path, b"not a zip file at all").expect("write");
    let result = validate_integrity(&path, &DetectedFormat::Epub);
    assert!(matches!(result, Err(ProcessingError::IntegrityError(_))));
}
