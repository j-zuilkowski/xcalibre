use xcalibre_processing::convert::htmlz::epub_to_htmlz;
use std::path::PathBuf;

fn epub_fixture() -> PathBuf { PathBuf::from("tests/fixtures/fixture_epub.epub") }

#[test]
fn test_epub_to_htmlz_creates_file() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.htmlz");
    epub_to_htmlz(&epub_fixture(), &out).expect("conversion");
    assert!(out.exists(), "HTMLZ output must be created");
}

#[test]
fn test_htmlz_is_valid_zip() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.htmlz");
    epub_to_htmlz(&epub_fixture(), &out).unwrap();
    let bytes = std::fs::read(&out).unwrap();
    assert_eq!(&bytes[..2], b"PK", "HTMLZ must be a ZIP file");
}

#[test]
fn test_htmlz_contains_index_html() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.htmlz");
    epub_to_htmlz(&epub_fixture(), &out).unwrap();
    let file = std::fs::File::open(&out).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    let entry_names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().to_string())
        .collect();
    assert!(
        entry_names.iter().any(|n| n == "index.html" || n.ends_with("/index.html")),
        "HTMLZ must contain index.html: {:?}", entry_names
    );
}

#[test]
fn test_htmlz_contains_metadata_opf() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.htmlz");
    epub_to_htmlz(&epub_fixture(), &out).unwrap();
    let file = std::fs::File::open(&out).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    let entry_names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().to_string())
        .collect();
    assert!(
        entry_names.iter().any(|n| n.contains("metadata") || n.ends_with(".opf")),
        "HTMLZ should contain metadata: {:?}", entry_names
    );
}
