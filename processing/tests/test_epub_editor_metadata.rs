use std::path::PathBuf;
use xcalibre_processing::editor::EpubEditor;

fn fixture_epub() -> PathBuf { PathBuf::from("tests/fixtures/fixture_epub.epub") }

#[test]
fn test_get_metadata_returns_title() {
    let editor = EpubEditor::open(&fixture_epub()).expect("open");
    let meta = editor.metadata();
    assert!(meta.title.is_some(), "fixture epub must have a title");
}

#[test]
fn test_set_title_saves_to_opf() {
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("meta_test.epub");
    std::fs::copy(&fixture_epub(), &dest).unwrap();

    let mut editor = EpubEditor::open(&dest).expect("open");
    editor.set_title("Modified Title");
    editor.save().expect("save");

    let editor2 = EpubEditor::open(&dest).expect("reopen");
    assert_eq!(editor2.metadata().title.as_deref(), Some("Modified Title"));
}

#[test]
fn test_set_author_saves_to_opf() {
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("author_test.epub");
    std::fs::copy(&fixture_epub(), &dest).unwrap();

    let mut editor = EpubEditor::open(&dest).expect("open");
    editor.set_authors(&["Test Author"]);
    editor.save().expect("save");

    let editor2 = EpubEditor::open(&dest).expect("reopen");
    assert_eq!(editor2.metadata().authors, vec!["Test Author"]);
}
