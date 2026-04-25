# Phase 1 — Processing Engine

> HOW TO USE: For every "Write `path`" line → call write_file with that path and content.
> For every "Then run:" block → call your shell tool for each command.
> DO NOT print code as output. Write it to disk using your tools.
> Status: ✅ done

## Status

| Task | Title | Status |
|------|-------|--------|
| P1-T01 | Fix migration SQL | ✅ |
| P1-T02 | ProcessingError enum | ✅ |
| P1-T03 | SHA-256 utility | ✅ |
| P1-T04 | db/queries.rs — structs | ✅ |
| P1-T05 | db/queries.rs — CRUD functions | ✅ |
| P1-T06 | src/lib.rs — module wiring | ✅ |
| P1-T07 | src/main.rs — CLI entry point | ✅ |
| P1-T08 | Fix test imports | ✅ |
| P1-T09 | BookMetadata struct | ✅ |
| P1-T10 | EPUB metadata parser | ✅ |
| P1-T11 | PDF/MOBI metadata stubs | ✅ |
| P1-T12 | run_metadata pipeline stage | ✅ |
| P1-T13 | Metadata tests | ✅ |
| P1-T14 | ExtractedText struct | ✅ |
| P1-T15 | EPUB text extractor | ✅ |
| P1-T16 | Text normalisation utils | ✅ |
| P1-T17 | run_text pipeline stage | ✅ |
| P1-T18 | Text extraction tests | ✅ |
| P1-T19 | CoverResult struct | ✅ |
| P1-T20 | EPUB cover extractor | ✅ |
| P1-T21 | Image resize | ✅ |
| P1-T22 | run_cover pipeline stage | ✅ |
| P1-T23 | Cover tests | ✅ |
| P1-T24 | api crate skeleton | ✅ |
| P1-T25 | ApiClient + keyring auth | ✅ |
| P1-T26 | push endpoint | ✅ |
| P1-T27 | run_push + retry logic | ✅ |
| P1-T28 | Push tests | ✅ |
| P1-T29 | Tauri v2 init | ✅ |
| P1-T30 | React + Vite + Tailwind | ✅ |
| P1-T31 | Zustand store | ✅ |
| P1-T32 | LibraryView component | ✅ |
| P1-T33 | BookDetail component | ✅ |
| P1-T34 | Tauri commands | ✅ |
| P1-T35 | Connect UI to commands | ✅ |

---

## P1-T02

Write `processing/src/error.rs` with this exact content:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("database error: {0}")]
    DbError(#[from] sqlx::Error),

    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("integrity check failed: {0}")]
    IntegrityError(String),

    #[error("metadata extraction failed: {0}")]
    MetadataError(String),

    #[error("text extraction failed: {0}")]
    TextError(String),

    #[error("cover extraction failed: {0}")]
    CoverError(String),

    #[error("duplicate file already imported: job_id={0}")]
    Duplicate(String),
}
```

Then run:
```bash
cargo build --workspace
git add processing/src/error.rs
git commit -m "P1-T02: add ProcessingError enum"
```

---

## P1-T03

Write `processing/src/utils/mod.rs`:
```rust
pub mod hash;
```

Write `processing/src/utils/hash.rs`:
```rust
use crate::error::ProcessingError;
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::Path;

pub fn sha256_file(path: &Path) -> Result<String, ProcessingError> {
    let mut file = std::fs::File::open(path).map_err(ProcessingError::IoError)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = file.read(&mut buf).map_err(ProcessingError::IoError)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
```

Then run:
```bash
cargo build --workspace
git add processing/src/utils/mod.rs processing/src/utils/hash.rs
git commit -m "P1-T03: add sha256_file utility"
```

---

## P1-T04

Write `processing/src/db/mod.rs`:
```rust
pub mod queries;
```

Write `processing/src/db/queries.rs`:
```rust
use chrono::Utc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct NewJob {
    pub id:                String,
    pub file_path:         String,
    pub file_sha256:       String,
    pub format:            String,
    pub status:            String,
    pub retry_count:       i64,
    pub next_retry_at:     Option<String>,
    pub xs_book_id: Option<String>,
    pub push_step:         i64,
    pub error_message:     Option<String>,
}

impl NewJob {
    pub fn new(file_path: &str, format: &str) -> Self {
        Self {
            id:                Uuid::new_v4().to_string(),
            file_path:         file_path.to_string(),
            file_sha256:       String::new(),
            format:            format.to_string(),
            status:            "PENDING".to_string(),
            retry_count:       0,
            next_retry_at:     None,
            xs_book_id: None,
            push_step:         0,
            error_message:     None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Job {
    pub id:                String,
    pub file_path:         String,
    pub file_sha256:       String,
    pub format:            String,
    pub status:            String,
    pub retry_count:       i64,
    pub next_retry_at:     Option<String>,
    pub xs_book_id: Option<String>,
    pub push_step:         i64,
    pub error_message:     Option<String>,
    pub created_at:        String,
    pub updated_at:        String,
}
```

Then run:
```bash
cargo build --workspace
git add processing/src/db/mod.rs processing/src/db/queries.rs
git commit -m "P1-T04: add NewJob and Job structs"
```

---

## P1-T05

Add the following to the end of `processing/src/db/queries.rs`.
Do not remove anything already in the file.

```rust
use crate::error::ProcessingError;
use sqlx::SqlitePool;

pub async fn create_job(pool: &SqlitePool, job: &NewJob) -> Result<String, ProcessingError> {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO jobs
         (id, file_path, file_sha256, format, status, retry_count,
          next_retry_at, xs_book_id, push_step, error_message,
          created_at, updated_at)
         VALUES (?,?,?,?,?,?,?,?,?,?,?,?)",
    )
    .bind(&job.id)
    .bind(&job.file_path)
    .bind(&job.file_sha256)
    .bind(&job.format)
    .bind(&job.status)
    .bind(job.retry_count)
    .bind(&job.next_retry_at)
    .bind(&job.xs_book_id)
    .bind(job.push_step)
    .bind(&job.error_message)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(ProcessingError::DbError)?;
    Ok(job.id.clone())
}

pub async fn get_job(pool: &SqlitePool, id: &str) -> Result<Option<Job>, ProcessingError> {
    let row = sqlx::query_as::<_, (String, String, String, String, String, i64, Option<String>, Option<String>, i64, Option<String>, String, String)>(
        "SELECT id, file_path, file_sha256, format, status, retry_count,
                next_retry_at, xs_book_id, push_step, error_message,
                created_at, updated_at
         FROM jobs WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(ProcessingError::DbError)?
    .map(|(id, file_path, file_sha256, format, status, retry_count,
           next_retry_at, xs_book_id, push_step, error_message,
           created_at, updated_at)| Job {
        id, file_path, file_sha256, format, status, retry_count,
        next_retry_at, xs_book_id, push_step, error_message,
        created_at, updated_at,
    });
    Ok(row)
}

pub async fn update_job_status(
    pool: &SqlitePool,
    id: &str,
    status: &str,
) -> Result<(), ProcessingError> {
    let now = Utc::now().to_rfc3339();
    sqlx::query("UPDATE jobs SET status = ?, updated_at = ? WHERE id = ?")
        .bind(status)
        .bind(&now)
        .bind(id)
        .execute(pool)
        .await
        .map_err(ProcessingError::DbError)?;
    Ok(())
}

pub async fn list_jobs_by_status(
    pool: &SqlitePool,
    status: &str,
) -> Result<Vec<Job>, ProcessingError> {
    let rows = sqlx::query_as::<_, (String, String, String, String, String, i64, Option<String>, Option<String>, i64, Option<String>, String, String)>(
        "SELECT id, file_path, file_sha256, format, status, retry_count,
                next_retry_at, xs_book_id, push_step, error_message,
                created_at, updated_at
         FROM jobs WHERE status = ? ORDER BY created_at ASC",
    )
    .bind(status)
    .fetch_all(pool)
    .await
    .map_err(ProcessingError::DbError)?
    .into_iter()
    .map(|(id, file_path, file_sha256, format, status, retry_count,
           next_retry_at, xs_book_id, push_step, error_message,
           created_at, updated_at)| Job {
        id, file_path, file_sha256, format, status, retry_count,
        next_retry_at, xs_book_id, push_step, error_message,
        created_at, updated_at,
    })
    .collect();
    Ok(rows)
}

Then run:
```bash
cargo build --workspace
git add processing/src/db/queries.rs
git commit -m "P1-T05: add create_job, get_job, update_job_status, list_jobs_by_status"
```

---

### ✅ Milestone check — after P1-T05
```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
```
Note: `cargo test` will not pass until P1-T06 creates lib.rs. Run build only here.

---

## P1-T06

Write `processing/src/lib.rs`:
```rust
pub mod db;
pub mod error;
pub mod pipeline;
pub mod plugins;
pub mod utils;
```

Write `processing/src/pipeline/mod.rs`:
```rust
pub mod ingest;
```

Then run:
```bash
cargo build --workspace
git add processing/src/lib.rs processing/src/pipeline/mod.rs
git commit -m "P1-T06: add lib.rs and pipeline/mod.rs"
```

---

## P1-T07

Write `processing/src/main.rs`:
```rust
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "xcalibre", version, about = "xCalibre ebook processor")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Ingest an ebook file into the local job database
    Ingest {
        #[arg(value_name = "FILE")]
        path: PathBuf,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    match cli.command {
        Commands::Ingest { path } => {
            println!("Ingesting: {}", path.display());
        }
    }
    Ok(())
}
```

Then run:
```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
git add processing/src/main.rs
git commit -m "P1-T07: add CLI entry point with ingest subcommand"
```

---

## P1-T08

Write `processing/tests/test_db.rs` with this exact content:

```rust
use xcalibre_processing::db::queries::{self, NewJob, Job};
use sqlx::{Pool, Sqlite, sqlite::SqlitePoolOptions};

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
async fn test_create_and_fetch_job() {
    let pool = setup_db().await;
    let mut job = NewJob::new("/tmp/test.epub", "EPUB");
    job.file_sha256 = "dummy_sha256_hash_for_testing".to_string();
    let job_id: String = queries::create_job(&pool, &job).await.expect("create");
    let fetched_job: Job = queries::get_job(&pool, &job_id).await.expect("fetch").expect("not found");
    assert_eq!(fetched_job.id, job_id);
    assert_eq!(fetched_job.file_path, job.file_path);
    assert_eq!(fetched_job.file_sha256, job.file_sha256);
    assert_eq!(fetched_job.format, job.format);
    assert_eq!(fetched_job.status, job.status);
    assert_eq!(fetched_job.retry_count, job.retry_count);
    assert_eq!(fetched_job.next_retry_at, job.next_retry_at);
    assert_eq!(fetched_job.xs_book_id, job.xs_book_id);
    assert_eq!(fetched_job.push_step, job.push_step);
    assert_eq!(fetched_job.error_message, job.error_message);
    assert!(!fetched_job.created_at.is_empty());
    assert!(!fetched_job.updated_at.is_empty());
}

#[tokio::test]
async fn test_update_job_status() {
    let pool = setup_db().await;
    let mut job = NewJob::new("/tmp/test.epub", "EPUB");
    job.file_sha256 = "dummy_sha256_hash_for_testing".to_string();
    let job_id: String = queries::create_job(&pool, &job).await.expect("create");
    queries::update_job_status(&pool, &job_id, "COMPLETED").await.expect("update");
    let fetched_job: Job = queries::get_job(&pool, &job_id).await.expect("fetch").expect("not found");
    assert_eq!(fetched_job.status, "COMPLETED");
}

#[tokio::test]
async fn test_list_jobs_by_status() {
    let pool = setup_db().await;
    let mut job1 = NewJob::new("/tmp/test1.epub", "EPUB");
    job1.file_sha256 = "sha1".to_string();
    let mut job2 = NewJob::new("/tmp/test2.epub", "EPUB");
    job2.file_sha256 = "sha2".to_string();
    let mut job3 = NewJob::new("/tmp/test3.epub", "EPUB");
    job3.file_sha256 = "sha3".to_string();
    job3.status = "COMPLETED".to_string();
    queries::create_job(&pool, &job1).await.expect("job1");
    queries::create_job(&pool, &job2).await.expect("job2");
    queries::create_job(&pool, &job3).await.expect("job3");
    let pending_jobs: Vec<Job> = queries::list_jobs_by_status(&pool, "PENDING").await.expect("list");
    assert_eq!(pending_jobs.len(), 2);
    for job in pending_jobs {
        assert_eq!(job.status, "PENDING");
    }
}
```

Write `processing/tests/test_detect.rs` with this exact content:

```rust
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
    assert_eq!(detect_format(&path).expect("detect"), DetectedFormat::Epub);
}

#[tokio::test]
async fn test_detect_pdf() {
    let path = PathBuf::from("tests/fixtures/fixture_pdf.pdf");
    assert_eq!(detect_format(&path).expect("detect"), DetectedFormat::Pdf);
}

#[tokio::test]
async fn test_detect_cbz() {
    let path = PathBuf::from("tests/fixtures/fixture_cbz.cbz");
    assert_eq!(detect_format(&path).expect("detect"), DetectedFormat::Cbz);
}

#[tokio::test]
async fn test_epub_not_cbz() {
    // A ZIP without mimetype entry must be Cbz, not Epub
    let path = PathBuf::from("tests/fixtures/fixture_cbz.cbz");
    assert_eq!(detect_format(&path).expect("detect"), DetectedFormat::Cbz);
}

#[tokio::test]
async fn test_integrity_epub_valid() {
    let path = PathBuf::from("tests/fixtures/fixture_epub.epub");
    let fmt = detect_format(&path).expect("detect");
    assert!(validate_integrity(&path, &fmt).is_ok());
}

#[tokio::test]
async fn test_ingest_creates_job() {
    let pool = setup_db().await;
    let path = PathBuf::from("tests/fixtures/fixture_epub.epub");
    let result: IngestResult = run_ingest(&pool, &path).await.expect("ingest");
    assert!(!result.job_id.is_empty());
    assert_eq!(result.format, DetectedFormat::Epub);
    let expected_sha256 = sha256_file(&path).expect("sha256");
    assert_eq!(result.sha256, expected_sha256);
    let job: Option<Job> = queries::get_job(&pool, &result.job_id).await.expect("get_job");
    let job = job.expect("job missing");
    assert_eq!(job.file_sha256, result.sha256);
    assert_eq!(job.format, "EPUB");
    assert_eq!(job.status, "PENDING");
}

#[tokio::test]
async fn test_detect_unknown_format_errors() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("zero.bin");
    std::fs::write(&path, b"\x00\x00\x00\x00\x00\x00\x00\x00").expect("write");
    assert!(matches!(detect_format(&path), Err(ProcessingError::UnsupportedFormat(_))));
}

#[tokio::test]
async fn test_integrity_invalid_zip_fails() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("bad.epub");
    std::fs::write(&path, b"not a zip file at all").expect("write");
    assert!(matches!(validate_integrity(&path, &DetectedFormat::Epub), Err(ProcessingError::IntegrityError(_))));
}
```

Also add `"macros"` to the sqlx features in `processing/Cargo.toml`:
```toml
sqlx = { version = "0.8", default-features = false, features = ["sqlite", "runtime-tokio-rustls", "migrate", "macros"] }
```

Then run:
```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
git add processing/tests/test_db.rs processing/tests/test_detect.rs processing/Cargo.toml
git commit -m "P1-T08: fix test imports — use xcalibre_processing:: prefix"
```

---

### ✅ Milestone check — after P1-T08
```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

---

## P1-T08b — Regenerate test fixtures (complete EPUB)

The fixture_epub.epub on disk is minimal (mimetype only). Metadata, text, and cover tests need a structurally complete EPUB. Write the fixture generator and run it once.

In `processing/Cargo.toml`, add a `[[bin]]` section after `[dev-dependencies]`:
```toml
[[bin]]
name = "gen_fixtures"
path = "src/bin/gen_fixtures.rs"
```

Write `processing/src/bin/gen_fixtures.rs`:
```rust
use std::fs::File;
use std::io::Write;
use zip::write::FileOptions;
use zip::ZipWriter;

fn main() {
    // fixture_epub.epub — complete EPUB with spine, metadata, and body text
    {
        let f = File::create("tests/fixtures/fixture_epub.epub").unwrap();
        let mut zip = ZipWriter::new(f);
        let stored = FileOptions::default()
            .compression_method(zip::CompressionMethod::Stored)
            .unix_permissions(0o644);

        zip.start_file("mimetype", stored).unwrap();
        zip.write_all(b"application/epub+zip").unwrap();

        zip.start_file("META-INF/container.xml", stored).unwrap();
        zip.write_all(br#"<?xml version="1.0"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf"
              media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#).unwrap();

        zip.start_file("OEBPS/content.opf", stored).unwrap();
        zip.write_all(br#"<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="2.0" unique-identifier="uid">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/"
            xmlns:opf="http://www.idpf.org/2007/opf">
    <dc:title>Fixture Book</dc:title>
    <dc:creator opf:role="aut">Test Author</dc:creator>
    <dc:language>en</dc:language>
    <dc:identifier id="uid">urn:isbn:9780000000000</dc:identifier>
  </metadata>
  <manifest>
    <item id="ch1" href="chapter1.xhtml"
          media-type="application/xhtml+xml"/>
  </manifest>
  <spine>
    <itemref idref="ch1"/>
  </spine>
</package>"#).unwrap();

        zip.start_file("OEBPS/chapter1.xhtml", stored).unwrap();
        zip.write_all(br#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head><title>Chapter 1</title></head>
<body><p>This is the fixture chapter one body text for testing purposes.</p></body>
</html>"#).unwrap();

        zip.finish().unwrap();
    }

    // fixture_pdf.pdf
    {
        let mut f = File::create("tests/fixtures/fixture_pdf.pdf").unwrap();
        f.write_all(b"%PDF-1.4\n1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n\
2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n\
3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>\nendobj\n\
xref\n0 4\n0000000000 65535 f \n0000000010 00000 n \n\
0000000053 00000 n \n0000000102 00000 n \n\
trailer\n<< /Size 4 /Root 1 0 R >>\nstartxref\n149\n%%EOF").unwrap();
    }

    // fixture_mobi.mobi — 60 zeros + BOOKMOBI + 64 zeros (>= 132 bytes)
    {
        let mut f = File::create("tests/fixtures/fixture_mobi.mobi").unwrap();
        f.write_all(&vec![0u8; 60]).unwrap();
        f.write_all(b"BOOKMOBI").unwrap();
        f.write_all(&vec![0u8; 64]).unwrap();
    }

    // fixture_cbz.cbz — ZIP without mimetype entry
    {
        let f = File::create("tests/fixtures/fixture_cbz.cbz").unwrap();
        let mut zip = ZipWriter::new(f);
        let stored = FileOptions::default()
            .compression_method(zip::CompressionMethod::Stored)
            .unix_permissions(0o644);
        zip.start_file("page001.jpg", stored).unwrap();
        zip.write_all(b"JFIF_placeholder").unwrap();
        zip.finish().unwrap();
    }

    // fixture_cbr.cbr — RAR v4 magic bytes only
    {
        let mut f = File::create("tests/fixtures/fixture_cbr.cbr").unwrap();
        f.write_all(b"Rar!\x1a\x07\x00").unwrap();
    }

    // fixture_txt.txt
    {
        let mut f = File::create("tests/fixtures/fixture_txt.txt").unwrap();
        f.write_all(b"Hello world\n").unwrap();
    }

    // fixture_zero.bin — all zeros (unknown format)
    {
        let mut f = File::create("tests/fixtures/fixture_zero.bin").unwrap();
        f.write_all(&vec![0u8; 128]).unwrap();
    }

    println!("Fixtures written to tests/fixtures/");
}
```

Then run:
```bash
cargo run --bin gen_fixtures --manifest-path processing/Cargo.toml
cargo test --workspace
git add processing/src/bin/gen_fixtures.rs processing/Cargo.toml processing/tests/fixtures/
git commit -m "P1-T08b: add complete fixture generator; regenerate test fixtures"
```

---

## P1-T09

In `processing/src/lib.rs`, add `pub mod metadata;` after the existing module declarations.

Write `processing/src/metadata/mod.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BookMetadata {
    pub title:        Option<String>,
    pub authors:      Vec<String>,
    pub language:     Option<String>,
    pub publisher:    Option<String>,
    pub published:    Option<String>,
    pub description:  Option<String>,
    pub isbn:         Option<String>,
    pub series:       Option<String>,
    pub series_index: Option<f32>,
    pub tags:         Vec<String>,
}

pub mod epub;
pub mod pdf;
pub mod mobi;
```

Then run:
```bash
cargo build --workspace
git add processing/src/metadata/mod.rs processing/src/lib.rs
git commit -m "P1-T09: add BookMetadata struct"
```

---

## P1-T10

In `processing/Cargo.toml`, add `roxmltree = "0.20"` under `[dependencies]`.

Write `processing/src/metadata/epub.rs`:
```rust
use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use std::io::Read;
use std::path::Path;

pub fn extract(path: &Path) -> Result<BookMetadata, ProcessingError> {
    let file = std::fs::File::open(path).map_err(ProcessingError::IoError)?;
    let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file))
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;

    let opf_path = {
        let mut container = archive
            .by_name("META-INF/container.xml")
            .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;
        let mut xml = String::new();
        container.read_to_string(&mut xml).map_err(ProcessingError::IoError)?;
        let doc = roxmltree::Document::parse(&xml)
            .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;
        doc.descendants()
            .find(|n| n.tag_name().name() == "rootfile")
            .and_then(|n| n.attribute("full-path"))
            .map(|s| s.to_string())
            .ok_or_else(|| ProcessingError::MetadataError("no rootfile in container.xml".into()))?
    };

    let opf_xml = {
        let mut opf = archive
            .by_name(&opf_path)
            .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;
        let mut xml = String::new();
        opf.read_to_string(&mut xml).map_err(ProcessingError::IoError)?;
        xml
    };

    let doc = roxmltree::Document::parse(&opf_xml)
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;

    let mut meta = BookMetadata::default();

    for node in doc.descendants() {
        match node.tag_name().name() {
            "title"       => meta.title       = node.text().map(str::trim).map(String::from),
            "creator"     => meta.authors.push(node.text().unwrap_or("").trim().to_string()),
            "language"    => meta.language    = node.text().map(str::trim).map(String::from),
            "publisher"   => meta.publisher   = node.text().map(str::trim).map(String::from),
            "date"        => meta.published   = node.text().map(str::trim).map(String::from),
            "description" => meta.description = node.text().map(str::trim).map(String::from),
            "identifier"  => {
                let scheme = node.attribute("opf:scheme")
                    .or_else(|| node.attribute("scheme"))
                    .unwrap_or("");
                if scheme.eq_ignore_ascii_case("isbn") {
                    meta.isbn = node.text().map(str::trim).map(String::from);
                }
            }
            _ => {}
        }
    }

    meta.authors.retain(|a| !a.is_empty());
    Ok(meta)
}
```

Then run:
```bash
cargo build --workspace
git add processing/src/metadata/epub.rs processing/Cargo.toml
git commit -m "P1-T10: add EPUB OPF metadata extractor"
```

---

### ✅ Milestone check — after P1-T10
```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

---

## P1-T11

Write `processing/src/metadata/pdf.rs`:
```rust
use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use std::path::Path;

pub fn extract(_path: &Path) -> Result<BookMetadata, ProcessingError> {
    Ok(BookMetadata::default())
}
```

Write `processing/src/metadata/mobi.rs`:
```rust
use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use std::path::Path;

pub fn extract(_path: &Path) -> Result<BookMetadata, ProcessingError> {
    Ok(BookMetadata::default())
}
```

Then run:
```bash
cargo build --workspace
git add processing/src/metadata/pdf.rs processing/src/metadata/mobi.rs
git commit -m "P1-T11: add PDF and MOBI metadata stubs"
```

---

## P1-T12

In `processing/src/pipeline/mod.rs`, add `pub mod metadata;` after the existing line.

Write `processing/src/pipeline/metadata.rs`:
```rust
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
        DetectedFormat::Epub                        => metadata::epub::extract(path)?,
        DetectedFormat::Pdf                         => metadata::pdf::extract(path)?,
        DetectedFormat::Mobi | DetectedFormat::Azw3 => metadata::mobi::extract(path)?,
        _                                           => BookMetadata::default(),
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
```

Then run:
```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
git add processing/src/pipeline/metadata.rs processing/src/pipeline/mod.rs
git commit -m "P1-T12: add run_metadata pipeline stage"
```

---

## P1-T13

Write `processing/tests/test_metadata.rs`:
```rust
use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::metadata;
use xcalibre_processing::pipeline::ingest::run_ingest;
use xcalibre_processing::pipeline::metadata::run_metadata;
use std::path::PathBuf;

async fn setup_db() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_epub_metadata_ok() {
    let path = PathBuf::from("tests/fixtures/fixture_epub.epub");
    let result = metadata::epub::extract(&path);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_pdf_metadata_stub() {
    let path = PathBuf::from("tests/fixtures/fixture_pdf.pdf");
    let meta = metadata::pdf::extract(&path).unwrap();
    assert!(meta.title.is_none());
}

#[tokio::test]
async fn test_run_metadata_inserts_row() {
    let pool = setup_db().await;
    let path = PathBuf::from("tests/fixtures/fixture_epub.epub");
    let ingest = run_ingest(&pool, &path).await.unwrap();
    run_metadata(&pool, &ingest, &path).await.unwrap();
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM job_metadata WHERE job_id = ?")
        .bind(&ingest.job_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(row.0, 1);
}
```

Then run:
```bash
cargo test --workspace
git add processing/tests/test_metadata.rs
git commit -m "P1-T13: add metadata tests"
```

---

## P1-T14

In `processing/src/lib.rs`, add `pub mod text;` after the existing module declarations.

Write `processing/src/text/mod.rs`:
```rust
pub mod epub;

#[derive(Debug, Clone)]
pub struct ExtractedText {
    pub full_text:  String,
    pub word_count: usize,
}
```

Then run:
```bash
cargo build --workspace
git add processing/src/text/mod.rs processing/src/lib.rs
git commit -m "P1-T14: add ExtractedText struct"
```

---

## P1-T15

In `processing/Cargo.toml`, add `regex = "1"` under `[dependencies]`.

Write `processing/src/text/epub.rs`:
```rust
use crate::error::ProcessingError;
use crate::text::ExtractedText;
use regex::Regex;
use std::io::Read;
use std::path::Path;

pub fn extract(path: &Path) -> Result<ExtractedText, ProcessingError> {
    let file = std::fs::File::open(path).map_err(ProcessingError::IoError)?;
    let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file))
        .map_err(|e| ProcessingError::TextError(e.to_string()))?;

    let opf_path = {
        let mut c = archive
            .by_name("META-INF/container.xml")
            .map_err(|e| ProcessingError::TextError(e.to_string()))?;
        let mut xml = String::new();
        c.read_to_string(&mut xml).map_err(ProcessingError::IoError)?;
        let doc = roxmltree::Document::parse(&xml)
            .map_err(|e| ProcessingError::TextError(e.to_string()))?;
        doc.descendants()
            .find(|n| n.tag_name().name() == "rootfile")
            .and_then(|n| n.attribute("full-path"))
            .map(String::from)
            .ok_or_else(|| ProcessingError::TextError("no rootfile".into()))?
    };

    let opf_xml = {
        let mut f = archive
            .by_name(&opf_path)
            .map_err(|e| ProcessingError::TextError(e.to_string()))?;
        let mut s = String::new();
        f.read_to_string(&mut s).map_err(ProcessingError::IoError)?;
        s
    };

    let opf_doc = roxmltree::Document::parse(&opf_xml)
        .map_err(|e| ProcessingError::TextError(e.to_string()))?;

    let opf_dir = std::path::Path::new(&opf_path)
        .parent()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();

    let manifest: std::collections::HashMap<String, String> = opf_doc
        .descendants()
        .filter(|n| n.tag_name().name() == "item")
        .filter_map(|n| {
            let id   = n.attribute("id")?.to_string();
            let href = n.attribute("href")?.to_string();
            Some((id, href))
        })
        .collect();

    let spine_hrefs: Vec<String> = opf_doc
        .descendants()
        .filter(|n| n.tag_name().name() == "itemref")
        .filter_map(|n| {
            let idref = n.attribute("idref")?;
            let href  = manifest.get(idref)?;
            let full  = if opf_dir.is_empty() {
                href.clone()
            } else {
                format!("{}/{}", opf_dir, href)
            };
            Some(full)
        })
        .collect();

    let tag_re = Regex::new(r"<[^>]+>")
        .map_err(|e| ProcessingError::TextError(e.to_string()))?;

    let mut parts = Vec::new();
    for href in &spine_hrefs {
        if let Ok(mut f) = archive.by_name(href) {
            let mut html = String::new();
            if f.read_to_string(&mut html).is_ok() {
                let plain = tag_re.replace_all(&html, " ");
                let collapsed: String = plain.split_whitespace().collect::<Vec<_>>().join(" ");
                if !collapsed.is_empty() {
                    parts.push(collapsed);
                }
            }
        }
    }

    let full_text  = parts.join("\n\n");
    let word_count = full_text.split_whitespace().count();
    Ok(ExtractedText { full_text, word_count })
}
```

Then run:
```bash
cargo build --workspace
git add processing/src/text/epub.rs processing/Cargo.toml
git commit -m "P1-T15: add EPUB text extractor"
```

---

### ✅ Milestone check — after P1-T15
```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

---

## P1-T16

In `processing/src/utils/mod.rs`, add `pub mod normalise;` after the existing line.

Write `processing/src/utils/normalise.rs`:
```rust
pub fn normalise(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

pub fn word_count(text: &str) -> usize {
    text.split_whitespace().count()
}
```

Then run:
```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
git add processing/src/utils/normalise.rs processing/src/utils/mod.rs
git commit -m "P1-T16: add text normalisation utils"
```

---

## P1-T17

In `processing/src/pipeline/mod.rs`, add `pub mod text;` after the existing lines.

Write `processing/src/pipeline/text.rs`:
```rust
use crate::error::ProcessingError;
use crate::pipeline::ingest::IngestResult;
use crate::plugins::DetectedFormat;
use crate::text;
use sqlx::SqlitePool;
use std::path::Path;
use tracing::info;

pub async fn run_text(
    pool: &SqlitePool,
    result: &IngestResult,
    path: &Path,
) -> Result<(), ProcessingError> {
    let extracted = match result.format {
        DetectedFormat::Epub => text::epub::extract(path)?,
        _                    => return Ok(()),
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

    info!(job_id = %result.job_id, words = extracted.word_count, "text extracted");
    Ok(())
}
```

Then run:
```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
git add processing/src/pipeline/text.rs processing/src/pipeline/mod.rs
git commit -m "P1-T17: add run_text pipeline stage"
```

---

## P1-T18

Write `processing/tests/test_text.rs`:
```rust
use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::pipeline::ingest::run_ingest;
use xcalibre_processing::pipeline::text::run_text;
use xcalibre_processing::text;
use std::path::PathBuf;

async fn setup_db() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_epub_text_extract() {
    let path = PathBuf::from("tests/fixtures/fixture_epub.epub");
    let result = text::epub::extract(&path).unwrap();
    assert!(result.word_count > 0);
}

#[tokio::test]
async fn test_run_text_inserts_row() {
    let pool = setup_db().await;
    let path = PathBuf::from("tests/fixtures/fixture_epub.epub");
    let ingest = run_ingest(&pool, &path).await.unwrap();
    run_text(&pool, &ingest, &path).await.unwrap();
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM job_text WHERE job_id = ?")
        .bind(&ingest.job_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(row.0, 1);
}
```

Then run:
```bash
cargo test --workspace
git add processing/tests/test_text.rs
git commit -m "P1-T18: add text extraction tests"
```

---

## P1-T19

In `processing/src/lib.rs`, add `pub mod cover;` after the existing module declarations.

Write `processing/src/cover/mod.rs`:
```rust
pub mod epub;
pub mod resize;

#[derive(Debug, Clone)]
pub struct CoverResult {
    pub data:      Vec<u8>,
    pub mime_type: String,
    pub width:     u32,
    pub height:    u32,
}
```

Then run:
```bash
cargo build --workspace
git add processing/src/cover/mod.rs processing/src/lib.rs
git commit -m "P1-T19: add CoverResult struct"
```

---

## P1-T20

In `processing/Cargo.toml`, add under `[dependencies]`:
```toml
image = { version = "0.25", default-features = false, features = ["jpeg", "png"] }
```

Write `processing/src/cover/epub.rs`:
```rust
use crate::cover::CoverResult;
use crate::error::ProcessingError;
use std::io::Read;
use std::path::Path;

pub fn extract(path: &Path) -> Result<Option<CoverResult>, ProcessingError> {
    let file = std::fs::File::open(path).map_err(ProcessingError::IoError)?;
    let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file))
        .map_err(|e| ProcessingError::CoverError(e.to_string()))?;

    let opf_path = {
        let mut c = archive
            .by_name("META-INF/container.xml")
            .map_err(|e| ProcessingError::CoverError(e.to_string()))?;
        let mut xml = String::new();
        c.read_to_string(&mut xml).map_err(ProcessingError::IoError)?;
        let doc = roxmltree::Document::parse(&xml)
            .map_err(|e| ProcessingError::CoverError(e.to_string()))?;
        doc.descendants()
            .find(|n| n.tag_name().name() == "rootfile")
            .and_then(|n| n.attribute("full-path"))
            .map(String::from)
            .ok_or_else(|| ProcessingError::CoverError("no rootfile".into()))?
    };

    let opf_xml = {
        let mut f = archive
            .by_name(&opf_path)
            .map_err(|e| ProcessingError::CoverError(e.to_string()))?;
        let mut s = String::new();
        f.read_to_string(&mut s).map_err(ProcessingError::IoError)?;
        s
    };

    let opf_dir = std::path::Path::new(&opf_path)
        .parent()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();

    let doc = roxmltree::Document::parse(&opf_xml)
        .map_err(|e| ProcessingError::CoverError(e.to_string()))?;

    let cover_id = doc.descendants()
        .find(|n| n.tag_name().name() == "meta" && n.attribute("name") == Some("cover"))
        .and_then(|n| n.attribute("content"))
        .map(String::from);

    let cover_href = doc.descendants()
        .filter(|n| n.tag_name().name() == "item")
        .find(|n| {
            let matches_id = cover_id.as_deref().map_or(false, |id| n.attribute("id") == Some(id));
            let has_prop   = n.attribute("properties").map_or(false, |p| p.contains("cover-image"));
            matches_id || has_prop
        })
        .and_then(|n| n.attribute("href"))
        .map(|h| {
            if opf_dir.is_empty() { h.to_string() } else { format!("{}/{}", opf_dir, h) }
        });

    let href = match cover_href {
        Some(h) => h,
        None    => return Ok(None),
    };

    let mut bytes = Vec::new();
    archive
        .by_name(&href)
        .map_err(|e| ProcessingError::CoverError(e.to_string()))?
        .read_to_end(&mut bytes)
        .map_err(ProcessingError::IoError)?;

    let mime_type = match href.rsplit('.').next().unwrap_or("").to_lowercase().as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png"          => "image/png",
        "gif"          => "image/gif",
        "webp"         => "image/webp",
        _              => "image/jpeg",
    }.to_string();

    let img = image::load_from_memory(&bytes)
        .map_err(|e| ProcessingError::CoverError(e.to_string()))?;

    Ok(Some(CoverResult {
        data: bytes,
        mime_type,
        width:  img.width(),
        height: img.height(),
    }))
}
```

Then run:
```bash
cargo build --workspace
git add processing/src/cover/epub.rs processing/Cargo.toml
git commit -m "P1-T20: add EPUB cover extractor"
```

---

### ✅ Milestone check — after P1-T20
```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

---

## P1-T21

Write `processing/src/cover/resize.rs`:
```rust
use crate::cover::CoverResult;
use crate::error::ProcessingError;

pub const COVER_MAX_W: u32 = 500;
pub const COVER_MAX_H: u32 = 750;

pub fn resize_cover(cover: CoverResult) -> Result<CoverResult, ProcessingError> {
    let img = image::load_from_memory(&cover.data)
        .map_err(|e| ProcessingError::CoverError(e.to_string()))?;

    let resized = img.resize(COVER_MAX_W, COVER_MAX_H, image::imageops::FilterType::Lanczos3);
    let (w, h)  = (resized.width(), resized.height());

    let mut out = std::io::Cursor::new(Vec::new());
    resized
        .write_to(&mut out, image::ImageFormat::Jpeg)
        .map_err(|e| ProcessingError::CoverError(e.to_string()))?;

    Ok(CoverResult {
        data:      out.into_inner(),
        mime_type: "image/jpeg".to_string(),
        width:     w,
        height:    h,
    })
}
```

Then run:
```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
git add processing/src/cover/resize.rs
git commit -m "P1-T21: add cover resize to 500x750 JPEG"
```

---

## P1-T22

In `processing/src/pipeline/mod.rs`, add `pub mod cover;` after the existing lines.

Write `processing/src/pipeline/cover.rs`:
```rust
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
        _                    => return Ok(None),
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
```

Then run:
```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
git add processing/src/pipeline/cover.rs processing/src/pipeline/mod.rs
git commit -m "P1-T22: add run_cover pipeline stage"
```

---

## P1-T23

Write `processing/tests/test_cover.rs`:
```rust
use xcalibre_processing::cover::{epub, resize, CoverResult};
use std::path::PathBuf;

#[test]
fn test_epub_cover_extract() {
    let path = PathBuf::from("tests/fixtures/fixture_epub.epub");
    let result = epub::extract(&path);
    assert!(result.is_ok());
}

#[test]
fn test_resize_cover() {
    let img = image::DynamicImage::new_rgb8(1000, 1500);
    let mut bytes = std::io::Cursor::new(Vec::new());
    img.write_to(&mut bytes, image::ImageFormat::Jpeg).unwrap();
    let cover = CoverResult {
        data:      bytes.into_inner(),
        mime_type: "image/jpeg".to_string(),
        width:     1000,
        height:    1500,
    };
    let resized = resize::resize_cover(cover).unwrap();
    assert!(resized.width  <= 500);
    assert!(resized.height <= 750);
}
```

Then run:
```bash
cargo test --workspace
git add processing/tests/test_cover.rs
git commit -m "P1-T23: add cover extraction tests"
```

---

## P1-T24

In the root `Cargo.toml`, add `"api"` to the `members` array under `[workspace]`.

Write `api/Cargo.toml`:
```toml
[package]
name = "xcalibre-api"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow     = "1"
reqwest    = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
serde      = { version = "1", features = ["derive"] }
serde_json = "1"
tokio      = { version = "1", features = ["full"] }
tracing    = "0.1"
keyring    = "2"
thiserror  = "1"
```

Write `api/src/lib.rs`:
```rust
pub mod client;
pub mod push;
```

Then run:
```bash
cargo build --workspace
git add api/Cargo.toml api/src/lib.rs Cargo.toml
git commit -m "P1-T24: add api crate skeleton"
```

---

### ✅ Milestone check — after P1-T24
```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

---

## P1-T25

Write `api/src/client.rs`:
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("auth error: {0}")]
    Auth(String),
    #[error("API error {status}: {body}")]
    Api { status: u16, body: String },
}

const SERVICE: &str = "xcalibre";
const ACCOUNT: &str = "xs_token";

pub struct ApiClient {
    base_url: String,
    token:    String,
    http:     reqwest::Client,
}

impl ApiClient {
    pub fn from_keyring(base_url: &str) -> Result<Self, ApiError> {
        let entry = keyring::Entry::new(SERVICE, ACCOUNT)
            .map_err(|e| ApiError::Auth(e.to_string()))?;
        let token = entry.get_password()
            .map_err(|e| ApiError::Auth(e.to_string()))?;
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(ApiError::Http)?;
        Ok(Self { base_url: base_url.to_string(), token, http })
    }

    pub(crate) fn http(&self)     -> &reqwest::Client { &self.http }
    pub(crate) fn base_url(&self) -> &str             { &self.base_url }
    pub(crate) fn token(&self)    -> &str             { &self.token }
}
```

Then run:
```bash
cargo build --workspace
git add api/src/client.rs
git commit -m "P1-T25: add ApiClient with keyring auth"
```

---

## P1-T26

Write `api/src/push.rs`:
```rust
use crate::client::{ApiClient, ApiError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct PushPayload {
    pub job_id:   String,
    pub format:   String,
    pub sha256:   String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct PushResponse {
    pub book_id: String,
}

pub async fn push_book(
    client: &ApiClient,
    payload: &PushPayload,
) -> Result<PushResponse, ApiError> {
    let resp = client
        .http()
        .post(format!("{}/books", client.base_url()))
        .bearer_auth(client.token())
        .json(payload)
        .send()
        .await?;

    let status = resp.status().as_u16();
    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(ApiError::Api { status, body });
    }
    Ok(resp.json::<PushResponse>().await?)
}
```

Then run:
```bash
cargo build --workspace
git add api/src/push.rs
git commit -m "P1-T26: add push_book endpoint"
```

---

## P1-T27

In `processing/Cargo.toml`, add under `[dependencies]`:
```toml
xcalibre-api = { path = "../api" }
```

In `processing/src/pipeline/mod.rs`, add `pub mod push;` after the existing lines.

Write `processing/src/pipeline/push.rs`:
```rust
use crate::db::queries;
use crate::error::ProcessingError;
use crate::pipeline::ingest::IngestResult;
use sqlx::SqlitePool;
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

    let payload = push::PushPayload {
        job_id:   result.job_id.clone(),
        format:   result.format.to_string(),
        sha256:   result.sha256.clone(),
        metadata: metadata_json,
    };

    match push::push_book(client, &payload).await {
        Ok(resp) => {
            queries::update_job_status(pool, &result.job_id, "COMPLETED").await?;
            sqlx::query(
                "UPDATE jobs SET xs_book_id = ?, updated_at = datetime() WHERE id = ?",
            )
            .bind(&resp.book_id)
            .bind(&result.job_id)
            .execute(pool)
            .await
            .map_err(ProcessingError::DbError)?;
            info!(job_id = %result.job_id, book_id = %resp.book_id, "push complete");
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
```

Then run:
```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
git add processing/src/pipeline/push.rs processing/src/pipeline/mod.rs processing/Cargo.toml
git commit -m "P1-T27: add run_push with retry logic"
```

---

## P1-T28

Write `processing/tests/test_push.rs`:
```rust
use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::pipeline::ingest::run_ingest;
use xcalibre_processing::pipeline::push::run_push;
use std::path::PathBuf;

async fn setup_db() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_push_skipped_without_client() {
    let pool   = setup_db().await;
    let path   = PathBuf::from("tests/fixtures/fixture_epub.epub");
    let ingest = run_ingest(&pool, &path).await.unwrap();
    let result = run_push(&pool, &ingest, serde_json::Value::Null, None).await;
    assert!(result.is_ok());
}
```

Then run:
```bash
cargo test --workspace
git add processing/tests/test_push.rs
git commit -m "P1-T28: add push pipeline tests"
```

---

### ✅ Milestone check — after P1-T28
```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

---

## P1-T29

Run:
```bash
cargo install tauri-cli --version "^2" --locked
cargo tauri init --app-name xcalibre --window-title "xCalibre" --dist-dir ../ui/dist --dev-url http://localhost:5173 --before-dev-command "npm run dev" --before-build-command "npm run build"
```

When prompted for the `src-tauri` directory, accept the default (creates `src-tauri/` in the workspace root).

Then in `src-tauri/Cargo.toml`, ensure these dependencies are present:
```toml
[dependencies]
tauri          = { version = "2", features = ["protocol-asset"] }
tauri-build    = { version = "2", features = [] }
sqlx           = { version = "0.8", default-features = false, features = ["sqlite", "runtime-tokio-rustls", "migrate", "macros"] }
tokio          = { version = "1", features = ["full"] }
serde          = { version = "1", features = ["derive"] }
serde_json     = "1"
xcalibre-processing = { path = "../processing" }
xcalibre-api        = { path = "../api" }
zip            = "0.6"

[build-dependencies]
tauri-build = { version = "2", features = [] }
```

Add `"src-tauri"` to the `members` array in the root `Cargo.toml`.

Then run:
```bash
cargo build --workspace
git add src-tauri/ Cargo.toml Cargo.lock
git commit -m "P1-T29: Tauri v2 scaffold"
```

---

## P1-T30

Do NOT use `npm create vite` — write the config files directly.

Write `ui/package.json` with this exact content:
```json
{
  "name": "xcalibre-ui",
  "private": true,
  "version": "0.0.1",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview"
  },
  "dependencies": {
    "@tauri-apps/api": "^2",
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "zustand": "^4.5.2"
  },
  "devDependencies": {
    "@types/react": "^18.2.66",
    "@types/react-dom": "^18.2.22",
    "@vitejs/plugin-react": "^4.2.1",
    "autoprefixer": "^10.4.19",
    "postcss": "^8.4.38",
    "tailwindcss": "^3.4.3",
    "typescript": "^5.2.2",
    "vite": "^5.2.0"
  }
}
```

Write `ui/tsconfig.json` with this exact content:
```json
{
  "files": [],
  "references": [
    { "path": "./tsconfig.app.json" },
    { "path": "./tsconfig.node.json" }
  ]
}
```

Write `ui/tsconfig.app.json` with this exact content:
```json
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true
  },
  "include": ["src"]
}
```

Write `ui/tsconfig.node.json` with this exact content:
```json
{
  "compilerOptions": {
    "target": "ES2022",
    "lib": ["ES2023"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowSyntheticDefaultImports": true,
    "strict": true,
    "noEmit": true
  },
  "include": ["vite.config.ts"]
}
```

Write `ui/vite.config.ts` with this exact content:
```ts
import { defineConfig } from "vite"
import react from "@vitejs/plugin-react"

export default defineConfig({
  plugins: [react()],
  build: { outDir: "dist" },
})
```

Write `ui/index.html` with this exact content:
```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>xCalibre</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
```

Write `ui/postcss.config.js` with this exact content:
```js
export default {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
  },
}
```

Write `ui/tailwind.config.js` with this exact content:
```js
/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  darkMode: "class",
  theme: { extend: {} },
  plugins: [],
}
```

Write `ui/src/index.css` with this exact content:
```css
@tailwind base;
@tailwind components;
@tailwind utilities;
```

Write `ui/src/main.tsx` with this exact content:
```tsx
import React from "react"
import ReactDOM from "react-dom/client"
import App from "./App"
import "./index.css"

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
)
```

Then run:
```bash
cd ui && npm install && npm run build && cd ..
git add ui/
git commit -m "P1-T30: React + Vite + Tailwind scaffold (written directly)"
```

Replace the content of `ui/tailwind.config.js` with:
```js
/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  darkMode: "class",
  theme: { extend: {} },
  plugins: [],
}
```

Replace the content of `ui/src/index.css` with:
```css
@tailwind base;
@tailwind components;
@tailwind utilities;
```

Write `ui/src/App.tsx` with this exact content (stub — T35 will replace it):
```tsx
export default function App() {
  return <div>xCalibre loading…</div>
}
```

Then run:
```bash
cd ui && npm install && npm run build && cd ..
git add ui/
git commit -m "P1-T30: React + Vite + Tailwind scaffold"
```

---

## P1-T31

Write `ui/src/store/libraryStore.ts` with this exact content:
```ts
import { create } from "zustand"
import { invoke } from "@tauri-apps/api/core"

export interface Book {
  id: string
  title: string
  authors: string[]
  format: string
  cover_path: string | null
  progress_percent: number
  last_opened_at: string | null
}

interface LibraryState {
  books: Book[]
  loading: boolean
  error: string | null
  fetchBooks: () => Promise<void>
}

export const useLibraryStore = create<LibraryState>((set) => ({
  books: [],
  loading: false,
  error: null,
  fetchBooks: async () => {
    set({ loading: true, error: null })
    try {
      const books = await invoke<Book[]>("list_books")
      set({ books, loading: false })
    } catch (e) {
      set({ error: String(e), loading: false })
    }
  },
}))
```

Write `ui/src/store/settingsStore.ts` with this exact content:
```ts
import { create } from "zustand"
import { persist } from "zustand/middleware"

interface SettingsState {
  fontSize: number
  theme: "light" | "dark" | "sepia"
  setFontSize: (size: number) => void
  setTheme: (theme: "light" | "dark" | "sepia") => void
}

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set) => ({
      fontSize: 18,
      theme: "light",
      setFontSize: (fontSize) => set({ fontSize }),
      setTheme: (theme) => set({ theme }),
    }),
    { name: "xcalibre-settings" },
  ),
)
```

Then run:
```bash
git add ui/src/store/
git commit -m "P1-T31: add Zustand library and settings stores"
```

---

## P1-T32

Write `ui/src/components/LibraryView.tsx` with this exact content:
```tsx
import { useEffect } from "react"
import { useLibraryStore } from "../store/libraryStore"

export function LibraryView() {
  const { books, loading, error, fetchBooks } = useLibraryStore()

  useEffect(() => {
    fetchBooks()
  }, [fetchBooks])

  if (loading) return <div className="p-8 text-gray-500">Loading…</div>
  if (error)   return <div className="p-8 text-red-500">{error}</div>
  if (books.length === 0)
    return <div className="p-8 text-gray-400">No books yet. Drag a file to ingest.</div>

  return (
    <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4 p-6">
      {books.map((book) => (
        <div
          key={book.id}
          className="flex flex-col rounded-lg overflow-hidden shadow hover:shadow-md transition-shadow cursor-pointer bg-white dark:bg-gray-800"
        >
          <div className="h-48 bg-gray-200 dark:bg-gray-700 flex items-center justify-center">
            {book.cover_path ? (
              <img
                src={`asset://${book.cover_path}`}
                alt={book.title}
                className="h-full w-full object-cover"
              />
            ) : (
              <span className="text-gray-400 text-sm">{book.format}</span>
            )}
          </div>
          <div className="p-2">
            <p className="text-sm font-medium truncate dark:text-white">{book.title}</p>
            <p className="text-xs text-gray-500 truncate">{book.authors.join(", ")}</p>
            {book.progress_percent > 0 && (
              <div className="mt-1 h-1 bg-gray-200 rounded-full">
                <div
                  className="h-1 bg-blue-500 rounded-full"
                  style={{ width: `${book.progress_percent}%` }}
                />
              </div>
            )}
          </div>
        </div>
      ))}
    </div>
  )
}
```

Then run:
```bash
git add ui/src/components/LibraryView.tsx
git commit -m "P1-T32: add LibraryView grid component"
```

---

## P1-T33

Write `ui/src/components/BookDetail.tsx` with this exact content:
```tsx
import { Book } from "../store/libraryStore"

interface Props {
  book: Book
  onClose: () => void
}

export function BookDetail({ book, onClose }: Props) {
  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-white dark:bg-gray-800 rounded-xl shadow-2xl w-full max-w-md p-6">
        <button
          onClick={onClose}
          className="float-right text-gray-400 hover:text-gray-600 text-xl leading-none"
        >
          ×
        </button>
        <h2 className="text-lg font-semibold dark:text-white mb-1">{book.title}</h2>
        <p className="text-sm text-gray-500 mb-4">{book.authors.join(", ")}</p>
        <p className="text-xs text-gray-400 uppercase tracking-wide mb-1">Format</p>
        <p className="text-sm dark:text-gray-300 mb-4">{book.format}</p>
        {book.progress_percent > 0 && (
          <>
            <p className="text-xs text-gray-400 uppercase tracking-wide mb-1">Progress</p>
            <p className="text-sm dark:text-gray-300">{Math.round(book.progress_percent)}%</p>
          </>
        )}
      </div>
    </div>
  )
}
```

Then run:
```bash
git add ui/src/components/BookDetail.tsx
git commit -m "P1-T33: add BookDetail modal component"
```

---

## P1-T34

Write `src-tauri/src/commands.rs` with this exact content:
```rust
use std::sync::Arc;
use sqlx::SqlitePool;

#[tauri::command]
pub async fn list_books(
    pool: tauri::State<'_, Arc<SqlitePool>>,
) -> Result<Vec<serde_json::Value>, String> {
    let rows = sqlx::query_as::<_, (String, String, String, String, Option<String>, f64, Option<String>)>(
        "SELECT id, title, authors_json, format, cover_path, progress_percent, last_opened_at
         FROM local_books ORDER BY last_opened_at DESC NULLS LAST",
    )
    .fetch_all(pool.inner().as_ref())
    .await
    .map_err(|e| e.to_string())?;

    let books = rows
        .into_iter()
        .map(|(id, title, authors_json, format, cover_path, progress_percent, last_opened_at)| {
            let authors: Vec<String> =
                serde_json::from_str(&authors_json).unwrap_or_default();
            serde_json::json!({
                "id": id,
                "title": title,
                "authors": authors,
                "format": format,
                "cover_path": cover_path,
                "progress_percent": progress_percent,
                "last_opened_at": last_opened_at,
            })
        })
        .collect();

    Ok(books)
}

#[tauri::command]
pub async fn ingest_file(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    path: String,
) -> Result<String, String> {
    let path = std::path::Path::new(&path);
    let result = xcalibre_processing::pipeline::ingest::run_ingest(
        pool.inner().as_ref(),
        path,
    )
    .await
    .map_err(|e| e.to_string())?;
    Ok(result.job_id)
}

#[tauri::command]
pub async fn update_position(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    book_id: String,
    position: String,
) -> Result<(), String> {
    xcalibre_processing::db::queries::update_reading_position(
        pool.inner().as_ref(),
        &book_id,
        &position,
    )
    .await
    .map_err(|e| e.to_string())
}
```

Then run:
```bash
cargo build --workspace
git add src-tauri/src/commands.rs
git commit -m "P1-T34: add Tauri commands (list_books, ingest_file, update_position)"
```

---

## P1-T35

Write `src-tauri/src/main.rs` with this exact content:
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use tauri::Manager;

mod commands;

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_dir = app.path().app_data_dir()
                .expect("failed to get app data dir");
            std::fs::create_dir_all(&app_dir).expect("failed to create app data dir");
            let db_path = app_dir.join("jobs.db");
            let pool = tauri::async_runtime::block_on(async {
                let pool = sqlx::sqlite::SqlitePoolOptions::new()
                    .connect(&format!("sqlite:{}", db_path.display()))
                    .await
                    .expect("failed to open DB");
                sqlx::migrate!("../processing/src/db/migrations")
                    .run(&pool)
                    .await
                    .expect("failed to run migrations");
                pool
            });
            app.manage(Arc::new(pool));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_books,
            commands::ingest_file,
            commands::update_position,
        ])
        .run(tauri::generate_context!())
        .expect("error running tauri application");
}
```

Write `ui/src/App.tsx` with this exact content:
```tsx
import { LibraryView } from "./components/LibraryView"

function App() {
  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      <header className="h-12 flex items-center px-6 border-b border-gray-200 dark:border-gray-700">
        <h1 className="text-lg font-semibold dark:text-white">xCalibre</h1>
      </header>
      <main>
        <LibraryView />
      </main>
    </div>
  )
}

export default App
```

Then run:
```bash
cargo build --workspace
cd ui && npm run build && cd ..
git add src-tauri/src/main.rs ui/src/App.tsx
git commit -m "P1-T35: wire Tauri main + App shell"
```

---

---

## P1-T36 — Install Vitest + React Testing Library

Install the frontend test stack (run from repo root):

```bash
cd ui && npm install --save-dev \
  @testing-library/react \
  @testing-library/user-event \
  @testing-library/jest-dom \
  jsdom && cd ..
```

Write `ui/vitest.config.ts` with this exact content:

```ts
import { defineConfig } from "vitest/config"
import react from "@vitejs/plugin-react"

export default defineConfig({
  plugins: [react()],
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["./src/test/setup.ts"],
  },
})
```

Write `ui/src/test/setup.ts` with this exact content:

```ts
import "@testing-library/jest-dom"
import { vi, afterEach } from "vitest"

const _invokeHandlers: Record<string, unknown> = {}

export function mockInvoke(cmd: string, value: unknown) {
  _invokeHandlers[cmd] = value
}

const invokeMock = vi.fn((cmd: string) => {
  const val = _invokeHandlers[cmd]
  if (val instanceof Error) return Promise.reject(val)
  return Promise.resolve(val ?? null)
})

Object.defineProperty(window, "__TAURI_INTERNALS__", {
  value: {
    invoke: invokeMock,
    transformCallback: vi.fn((cb: unknown) => { (window as any)._cb = cb; return 1 }),
    unregisterCallback: vi.fn(),
    metadata: { currentWindow: { label: "main" } },
    convertFileSrc: (src: string) => `asset://${src}`,
  },
  writable: true,
})

afterEach(() => {
  invokeMock.mockClear()
  Object.keys(_invokeHandlers).forEach((k) => delete _invokeHandlers[k])
})

export { invokeMock }
```

Add to `ui/package.json` scripts:

```json
"test": "vitest run",
"test:watch": "vitest"
```

Then run:

```bash
cd ui && npm test 2>&1
```

Expected: 1 test file found (`cfi.test.ts`), all pass. Fix any config errors before proceeding.

---

## P1-T37 — Store tests (libraryStore + settingsStore)

Write `ui/src/store/libraryStore.test.ts`:
  Implement every test case from `localProject/TEST_SPEC.md` section
  `"ui/src/store/libraryStore.test.ts"`.

  Rules:
  - Test via `getState()` / `setState()` — do NOT render React components.
  - Reset store to initial state in `beforeEach`.
  - Use `mockInvoke()` from `../test/setup` to control `invoke` responses.
  - On error path: pass `new Error("failed")` to `mockInvoke`,
    assert `state.error` is set and `state.loading` is false.

Write `ui/src/store/settingsStore.test.ts`:
  Implement every test case from `localProject/TEST_SPEC.md` section
  `"ui/src/store/settingsStore.test.ts"`.

  Rules:
  - `localStorage.clear()` in `beforeEach`.
  - Assert persisted store state directly: `useSettingsStore.getState().theme`.

Then run:
```bash
cd ui && npm test -- --reporter=verbose 2>&1
```
All store tests must pass before continuing.

---

## P1-T38 — LibraryView component tests

Write `ui/src/components/LibraryView.test.tsx`:
  Implement every test case from `localProject/TEST_SPEC.md` section
  `"ui/src/components/LibraryView.test.tsx"`.

  Rules:
  - Props: `onSelectBook={vi.fn()}`, `selectedIds={[]}`, `onToggleSelected={vi.fn()}`.
  - Reset store in `beforeEach`: `useLibraryStore.setState({ books: [], loading: false, error: null })`.
  - For "shows skeleton while loading": set invoke to `new Promise(() => {})`,
    assert skeletons are present before the promise resolves.

Then run:
```bash
cd ui && npm test -- --reporter=verbose 2>&1
```
All tests must pass.

```bash
git add ui/vitest.config.ts ui/src/test/setup.ts ui/package.json \
        ui/src/store/libraryStore.test.ts \
        ui/src/store/settingsStore.test.ts \
        ui/src/components/LibraryView.test.tsx
git commit -m "test(phase1): vitest setup + store and LibraryView tests"
```

---

### ✅ Milestone check — after P1-T38
```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
cd ui && npm run build && npm test && cd ..
```
