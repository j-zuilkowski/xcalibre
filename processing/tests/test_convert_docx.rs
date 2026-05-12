use xcalibre_processing::convert::docx::epub_to_docx;
use std::path::PathBuf;

fn epub_fixture() -> PathBuf { PathBuf::from("tests/fixtures/fixture_epub.epub") }

#[test]
fn test_epub_to_docx_creates_file() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.docx");
    epub_to_docx(&epub_fixture(), &out).expect("conversion");
    assert!(out.exists(), "DOCX output file must be created");
    // DOCX files start with PK (ZIP magic)
    let bytes = std::fs::read(&out).unwrap();
    assert_eq!(&bytes[..2], b"PK", "DOCX must be a valid ZIP/DOCX file");
}

#[test]
fn test_epub_to_docx_minimum_size() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.docx");
    epub_to_docx(&epub_fixture(), &out).unwrap();
    let metadata = std::fs::metadata(&out).unwrap();
    assert!(metadata.len() > 1000, "DOCX file must be non-trivial in size");
}
