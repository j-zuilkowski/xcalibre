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
