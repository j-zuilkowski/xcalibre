# RMP-04a — OPF Write-back & Calibre Round-trip (Red: Failing Tests)

> HOW TO USE: Write every file exactly as shown, run every shell block.
> Prerequisite: rmp01b complete (OPF write-back needs library paths established).
> TDD role: RED — define the OPF serialization and round-trip contract via failing tests.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R04a-T01 | Failing tests: OPF 2.x serialization | ⬜ |
| R04a-T02 | Failing tests: OPF round-trip (read → edit → write → re-read) | ⬜ |
| R04a-T03 | Failing tests: write-back trigger on metadata edit | ⬜ |

---

## R04a-T01

Write `processing/tests/test_opf_write.rs` with this exact content:
```rust
//! Tests for OPF sidecar generation (RMP-04).
//! FAIL until rmp04b implements opf::write().

use xcalibre_processing::opf::{write_opf_sidecar, OPFMetadata};
use std::path::PathBuf;

fn sample_meta() -> OPFMetadata {
    OPFMetadata {
        title: Some("Test Book".into()),
        authors: vec!["Alice Tester".into()],
        language: Some("en".into()),
        publisher: Some("Test Press".into()),
        published: Some("2024-01-01".into()),
        description: Some("A test book.".into()),
        isbn: Some("9780000000001".into()),
        series: Some("Test Series".into()),
        series_index: Some(1.0),
        tags: vec!["fiction".into(), "test".into()],
        calibre_id: None,
    }
}

#[test]
fn test_write_opf_creates_file() {
    let dir = tempfile::tempdir().unwrap();
    let opf_path = dir.path().join("metadata.opf");
    write_opf_sidecar(&opf_path, &sample_meta()).expect("write");
    assert!(opf_path.exists());
}

#[test]
fn test_written_opf_is_valid_xml() {
    let dir = tempfile::tempdir().unwrap();
    let opf_path = dir.path().join("metadata.opf");
    write_opf_sidecar(&opf_path, &sample_meta()).unwrap();
    let content = std::fs::read_to_string(&opf_path).unwrap();
    // Must be valid XML
    assert!(content.starts_with("<?xml") || content.starts_with("<package"),
            "OPF must start with XML declaration or <package>");
    assert!(content.contains("<dc:title>Test Book</dc:title>"));
    assert!(content.contains("Alice Tester"));
}

#[test]
fn test_opf_contains_series_meta() {
    let dir = tempfile::tempdir().unwrap();
    let opf_path = dir.path().join("metadata.opf");
    write_opf_sidecar(&opf_path, &sample_meta()).unwrap();
    let content = std::fs::read_to_string(&opf_path).unwrap();
    // Calibre uses custom:series meta tag
    assert!(content.contains("calibre:series") || content.contains("series"),
            "OPF must contain series metadata");
}

#[test]
fn test_opf_without_series_omits_series_tag() {
    let dir = tempfile::tempdir().unwrap();
    let opf_path = dir.path().join("metadata.opf");
    let mut meta = sample_meta();
    meta.series = None;
    write_opf_sidecar(&opf_path, &meta).unwrap();
    let content = std::fs::read_to_string(&opf_path).unwrap();
    assert!(!content.contains("calibre:series"), "no series — tag must be absent");
}
```

Then run:
```bash
cargo test --workspace 2>&1 | grep -E "^error" | head -10
```

Expected: `xcalibre_processing::opf` module not found. RED confirmed.

```bash
git add processing/tests/test_opf_write.rs
git commit -m "R04a-T01: failing tests for OPF serialization"
```

---

## R04a-T02

Write `processing/tests/test_opf_roundtrip.rs` with this exact content:
```rust
//! Round-trip test: import Calibre metadata.opf → edit → write → re-read.

use xcalibre_processing::opf::{read_opf, write_opf_sidecar};
use std::path::PathBuf;

fn write_sample_calibre_opf(path: &PathBuf) {
    // Minimal Calibre-compatible OPF 2.x
    std::fs::write(path, r#"<?xml version='1.0' encoding='utf-8'?>
<package xmlns="http://www.idpf.org/2007/opf" version="2.0" unique-identifier="uuid_id">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/"
            xmlns:opf="http://www.idpf.org/2007/opf"
            xmlns:calibre="http://calibre.kovidgoyal.net/2009/metadata">
    <dc:title>Original Title</dc:title>
    <dc:creator opf:role="aut">Original Author</dc:creator>
    <dc:language>en</dc:language>
    <dc:publisher>Original Press</dc:publisher>
    <meta name="calibre:series" content="Original Series"/>
    <meta name="calibre:series_index" content="1"/>
  </metadata>
</package>"#).unwrap();
}

#[test]
fn test_read_calibre_opf() {
    let dir = tempfile::tempdir().unwrap();
    let opf_path = dir.path().join("metadata.opf");
    write_sample_calibre_opf(&opf_path);
    let meta = read_opf(&opf_path).expect("read");
    assert_eq!(meta.title.as_deref(), Some("Original Title"));
    assert_eq!(meta.authors, vec!["Original Author"]);
    assert_eq!(meta.series.as_deref(), Some("Original Series"));
    assert_eq!(meta.series_index, Some(1.0));
}

#[test]
fn test_round_trip_preserves_fields() {
    let dir = tempfile::tempdir().unwrap();
    let opf_path = dir.path().join("metadata.opf");
    write_sample_calibre_opf(&opf_path);

    let mut meta = read_opf(&opf_path).unwrap();
    meta.title = Some("New Title".into());
    meta.authors = vec!["New Author".into()];
    write_opf_sidecar(&opf_path, &meta).unwrap();

    let re_read = read_opf(&opf_path).unwrap();
    assert_eq!(re_read.title.as_deref(), Some("New Title"));
    assert_eq!(re_read.authors, vec!["New Author"]);
    // Series must survive the round-trip
    assert_eq!(re_read.series.as_deref(), Some("Original Series"));
}
```

Then run:
```bash
cargo test --workspace 2>&1 | grep -E "^error" | head -10
```

RED confirmed (`read_opf` and `write_opf_sidecar` not yet defined).

```bash
git add processing/tests/test_opf_roundtrip.rs
git commit -m "R04a-T02: failing tests for OPF round-trip"
```

---

## R04a-T03

Write `processing/tests/test_opf_trigger.rs` with this exact content:
```rust
//! Tests that update_book_metadata() writes a sidecar .opf file when
//! the book's directory uses managed layout.

use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::db::queries::update_book_metadata_with_opf;
use xcalibre_processing::metadata::BookMetadata;

async fn setup() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_opf_written_on_metadata_update() {
    let pool = setup().await;
    let dir = tempfile::tempdir().unwrap();
    let book_dir = dir.path().join("Author/Title (1)");
    std::fs::create_dir_all(&book_dir).unwrap();
    let book_path = book_dir.join("book.epub");
    std::fs::write(&book_path, b"placeholder").unwrap();

    // Insert a book with a known file_path inside book_dir
    sqlx::query(
        "INSERT INTO local_books (id, title, authors_json, format, file_path,
          file_sha256, status, progress_percent) VALUES ('bk1','Old','[\"A\"]','EPUB',?,
          'sha1','READY',0)",
    )
    .bind(book_path.to_string_lossy().as_ref())
    .execute(&pool).await.unwrap();

    let meta = BookMetadata {
        title: Some("New Title".into()),
        authors: vec!["New Author".into()],
        ..Default::default()
    };
    update_book_metadata_with_opf(&pool, "bk1", &meta).await.expect("update");

    let opf_path = book_dir.join("metadata.opf");
    assert!(opf_path.exists(), "OPF sidecar should have been written");
    let content = std::fs::read_to_string(&opf_path).unwrap();
    assert!(content.contains("New Title"));
}
```

Then run:
```bash
cargo test --workspace 2>&1 | grep -E "^error" | head -10
```

RED confirmed (`update_book_metadata_with_opf` not yet defined).

```bash
git add processing/tests/test_opf_trigger.rs
git commit -m "R04a-T03: failing test for OPF write-back trigger on metadata edit"
```

---

### ✅ RED Checkpoint

Proceed to **rmp04b**.
