use xcalibre_processing::convert::pdf::epub_to_pdf;
use std::path::PathBuf;

fn epub_fixture() -> PathBuf { PathBuf::from("tests/fixtures/fixture_epub.epub") }

#[test]
fn test_epub_to_pdf_creates_file() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.pdf");
    epub_to_pdf(&epub_fixture(), &out).expect("conversion");
    assert!(out.exists(), "PDF output file must be created");
}

#[test]
fn test_epub_to_pdf_has_pdf_magic() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.pdf");
    epub_to_pdf(&epub_fixture(), &out).unwrap();
    let bytes = std::fs::read(&out).unwrap();
    assert!(bytes.starts_with(b"%PDF-"), "output must be a valid PDF: {:?}", &bytes[..8.min(bytes.len())]);
}

#[test]
fn test_epub_to_pdf_minimum_size() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.pdf");
    epub_to_pdf(&epub_fixture(), &out).unwrap();
    let meta = std::fs::metadata(&out).unwrap();
    assert!(meta.len() > 500, "PDF must be non-trivial in size");
}

#[test]
fn test_epub_to_pdf_invalid_path_errors() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("out.pdf");
    let result = epub_to_pdf(std::path::Path::new("/nonexistent.epub"), &out);
    assert!(result.is_err());
}
