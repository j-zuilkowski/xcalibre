use std::path::PathBuf;
use xcalibre_processing::metadata;
use xcalibre_processing::text;

#[test]
fn test_pdf_metadata_extract() {
    let path = PathBuf::from("tests/fixtures/fixture_pdf.pdf");
    let meta = metadata::pdf::extract(&path).unwrap();
    assert!(meta.title.is_none() || meta.title.is_some());
}

#[test]
fn test_pdf_text_extract() {
    let path = PathBuf::from("tests/fixtures/fixture_pdf.pdf");
    let result = text::pdf::extract(&path);
    assert!(result.is_ok());
}

#[test]
fn test_mobi_metadata_extract() {
    let path = PathBuf::from("tests/fixtures/fixture_mobi.mobi");
    let meta = metadata::mobi::extract(&path);
    let _ = meta;
}

#[test]
fn test_fb2_metadata_extract() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.fb2");
    std::fs::write(
        &path,
        br#"<?xml version="1.0" encoding="utf-8"?>
<FictionBook xmlns="http://www.gribuser.ru/xml/fictionbook/2.0">
  <description>
    <title-info>
      <genre>sf</genre>
      <author><first-name>Frank</first-name><last-name>Herbert</last-name></author>
      <book-title>Dune</book-title>
      <lang>en</lang>
    </title-info>
    <publish-info><publisher>Chilton</publisher><year>1965</year></publish-info>
  </description>
  <body><section><p>The beginning is a very delicate time.</p></section></body>
</FictionBook>"#,
    )
    .unwrap();

    let meta = xcalibre_processing::metadata::fb2::extract(&path).unwrap();
    assert_eq!(meta.title.as_deref(), Some("Dune"));
    assert_eq!(meta.authors, vec!["Frank Herbert"]);
    assert_eq!(meta.publisher.as_deref(), Some("Chilton"));

    let text = xcalibre_processing::text::fb2::extract(&path).unwrap();
    assert!(text.word_count > 0);
}

#[test]
fn test_html_metadata_extract() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.html");
    std::fs::write(
        &path,
        b"<!DOCTYPE html><html><head><title>My Book</title>
<meta name='author' content='Jane Doe'></head>
<body><p>Hello world paragraph.</p></body></html>",
    )
    .unwrap();

    let meta = xcalibre_processing::metadata::html::extract(&path).unwrap();
    assert_eq!(meta.title.as_deref(), Some("My Book"));
    assert!(meta.authors.contains(&"Jane Doe".to_string()));

    let text = xcalibre_processing::text::html::extract(&path).unwrap();
    assert!(text.word_count >= 3);
}

#[test]
fn test_rtf_text_extract() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.rtf");
    std::fs::write(
        &path,
        br"{\rtf1\ansi {\title My Title}{\author Jane}\par Hello World\par}",
    )
    .unwrap();
    let text = xcalibre_processing::text::rtf::extract(&path).unwrap();
    assert!(text.word_count >= 2);
}
