//! Tests for KFX format support (RMP-07, S5-A: full implementation).
//! FAIL until rmp07b implements the KFX parser.

use std::path::PathBuf;
use xcalibre_processing::plugins::{detect_format, DetectedFormat};
use xcalibre_processing::metadata::BookMetadata;
use xcalibre_processing::text::ExtractedText;

fn kfx_fixture() -> PathBuf {
    PathBuf::from("tests/fixtures/fixture.kfx")
}

/// Build a minimal .kfx fixture (SQLite DB with the required KFX tables).
fn create_kfx_fixture() {
    // KFX is a SQLite database; create one with the minimal structure
    let path = kfx_fixture();
    if path.exists() { return; }
    // Use rusqlite to create a minimal KFX-like SQLite DB
    // KFX magic: first 16 bytes are SQLite header "SQLite format 3\000"
    // KFX has tables: fragments, fragment_properties, etc.
    // We create a minimal version that our parser can read.
    let conn = rusqlite::Connection::open(&path).unwrap();
    conn.execute_batch("
        CREATE TABLE fragments (
            id          TEXT PRIMARY KEY,
            ftype       TEXT NOT NULL,
            payload     BLOB
        );
        CREATE TABLE fragment_properties (
            fragment_id TEXT NOT NULL,
            key         TEXT NOT NULL,
            value       TEXT
        );
        INSERT INTO fragments VALUES ('meta', '$270', NULL);
        INSERT INTO fragment_properties VALUES ('meta', 'title', 'KFX Test Book');
        INSERT INTO fragment_properties VALUES ('meta', 'author', 'KFX Author');
        INSERT INTO fragment_properties VALUES ('meta', 'language', 'en');
        INSERT INTO fragments VALUES ('content_1', '$608', X'48656C6C6F20776F726C64');
    ").unwrap();
}

#[test]
fn test_detect_kfx_format() {
    create_kfx_fixture();
    let fmt = detect_format(&kfx_fixture()).expect("detect");
    assert_eq!(fmt, DetectedFormat::Kfx, "KFX SQLite file must be detected as Kfx");
}

#[test]
fn test_kfx_metadata_title() {
    create_kfx_fixture();
    let meta = xcalibre_processing::metadata::kfx::extract(&kfx_fixture())
        .expect("kfx metadata");
    assert_eq!(meta.title.as_deref(), Some("KFX Test Book"));
}

#[test]
fn test_kfx_metadata_author() {
    create_kfx_fixture();
    let meta = xcalibre_processing::metadata::kfx::extract(&kfx_fixture()).unwrap();
    assert_eq!(meta.authors, vec!["KFX Author"]);
}

#[test]
fn test_kfx_text_extraction() {
    create_kfx_fixture();
    let text = xcalibre_processing::text::kfx::extract(&kfx_fixture())
        .expect("kfx text");
    assert!(text.word_count > 0, "KFX text extraction must return non-empty text");
    assert!(text.full_text.contains("Hello") || text.full_text.contains("world"),
            "extracted text: {}", text.full_text);
}
