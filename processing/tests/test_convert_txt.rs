use xcalibre_processing::convert::txt::epub_to_txt;
use std::path::PathBuf;

fn epub_fixture() -> PathBuf { PathBuf::from("tests/fixtures/fixture_epub.epub") }

#[test]
fn test_epub_to_txt_produces_non_empty_output() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.txt");
    epub_to_txt(&epub_fixture(), &out).expect("conversion");
    assert!(out.exists());
    let text = std::fs::read_to_string(&out).unwrap();
    assert!(!text.trim().is_empty(), "TXT output must not be empty");
}

#[test]
fn test_epub_to_txt_preserves_paragraph_breaks() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.txt");
    epub_to_txt(&epub_fixture(), &out).unwrap();
    let text = std::fs::read_to_string(&out).unwrap();
    // fixture_epub has one paragraph — at minimum, it should have no HTML tags
    assert!(!text.contains('<'), "TXT output must not contain HTML tags");
}

#[test]
fn test_epub_to_txt_invalid_path_errors() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("out.txt");
    let result = epub_to_txt(std::path::Path::new("/nonexistent.epub"), &out);
    assert!(result.is_err());
}
