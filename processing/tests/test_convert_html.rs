use xcalibre_processing::convert::html::epub_to_html;
use std::path::PathBuf;

fn epub_fixture() -> PathBuf { PathBuf::from("tests/fixtures/fixture_epub.epub") }

#[test]
fn test_epub_to_html_produces_valid_html() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.html");
    epub_to_html(&epub_fixture(), &out).expect("conversion");
    let html = std::fs::read_to_string(&out).unwrap();
    assert!(html.contains("<!DOCTYPE html>") || html.contains("<html"),
            "output must be valid HTML: {}", &html[..100.min(html.len())]);
}

#[test]
fn test_epub_to_html_contains_body_text() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.html");
    epub_to_html(&epub_fixture(), &out).unwrap();
    let html = std::fs::read_to_string(&out).unwrap();
    assert!(html.contains("<body"), "output must have <body>");
    assert!(html.contains("fixture") || html.contains("chapter") || html.len() > 200,
            "output must contain book content");
}
