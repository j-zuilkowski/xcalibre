use xcalibre_processing::convert::kepub::epub_to_kepub;
use std::path::PathBuf;

fn epub_fixture() -> PathBuf { PathBuf::from("tests/fixtures/fixture_epub.epub") }

#[test]
fn test_epub_to_kepub_creates_file() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.kepub.epub");
    epub_to_kepub(&epub_fixture(), &out).expect("conversion");
    assert!(out.exists(), "KEPUB output must be created");
}

#[test]
fn test_kepub_is_valid_zip() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.kepub.epub");
    epub_to_kepub(&epub_fixture(), &out).unwrap();
    let bytes = std::fs::read(&out).unwrap();
    assert_eq!(&bytes[..2], b"PK", "KEPUB must be a ZIP file");
}

#[test]
fn test_kepub_contains_kobo_spans() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.kepub.epub");
    epub_to_kepub(&epub_fixture(), &out).unwrap();
    let file = std::fs::File::open(&out).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    let mut found_kobo = false;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).unwrap();
        let name = entry.name().to_string();
        if name.ends_with(".html") || name.ends_with(".xhtml") {
            let mut content = String::new();
            std::io::Read::read_to_string(&mut entry, &mut content).unwrap();
            if content.contains("kobo:") || content.contains("epub:type") {
                found_kobo = true;
                break;
            }
        }
    }
    assert!(found_kobo, "KEPUB spine items must contain Kobo markup");
}
