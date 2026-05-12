use xcalibre_processing::polish::pretty_print_epub;
use std::path::PathBuf;

fn epub_fixture() -> PathBuf { PathBuf::from("tests/fixtures/fixture_epub.epub") }

#[test]
fn test_pretty_print_creates_output() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("pretty.epub");
    pretty_print_epub(&epub_fixture(), &out).expect("pretty-print");
    assert!(out.exists(), "pretty-printed EPUB must be created");
}

#[test]
fn test_pretty_print_output_is_valid_zip() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("pretty.epub");
    pretty_print_epub(&epub_fixture(), &out).unwrap();
    let bytes = std::fs::read(&out).unwrap();
    assert_eq!(&bytes[..2], b"PK");
}

#[test]
fn test_pretty_print_spine_items_are_indented() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("pretty.epub");
    pretty_print_epub(&epub_fixture(), &out).unwrap();
    let file = std::fs::File::open(&out).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).unwrap();
        if entry.name().ends_with(".xhtml") || entry.name().ends_with(".html") {
            let mut content = String::new();
            std::io::Read::read_to_string(&mut entry, &mut content).unwrap();
            assert!(content.contains('\n'), "pretty-printed HTML must contain newlines");
            break;
        }
    }
}
