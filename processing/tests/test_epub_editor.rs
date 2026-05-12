use std::path::PathBuf;
use xcalibre_processing::editor::EpubEditor;

fn fixture_epub() -> PathBuf { PathBuf::from("tests/fixtures/fixture_epub.epub") }

#[test]
fn test_open_editor_lists_spine_items() {
    let editor = EpubEditor::open(&fixture_epub()).expect("open");
    let spine = editor.spine_items();
    assert!(!spine.is_empty(), "spine must have at least one item");
}

#[test]
fn test_read_spine_item_returns_html() {
    let editor = EpubEditor::open(&fixture_epub()).expect("open");
    let spine = editor.spine_items();
    let first = &spine[0];
    let content = editor.read_item(first).expect("read item");
    let text = String::from_utf8_lossy(&content);
    assert!(
        text.contains("<html") || text.contains("<!DOCTYPE"),
        "spine item must be HTML: {}", &text[..100.min(text.len())]
    );
}

#[test]
fn test_write_item_and_read_back() {
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("copy.epub");
    std::fs::copy(&fixture_epub(), &dest).unwrap();

    let mut editor = EpubEditor::open(&dest).expect("open");
    let spine = editor.spine_items();
    let first = spine[0].clone();

    let new_content = b"<html><body><p>Edited content</p></body></html>";
    editor.write_item(&first, new_content).expect("write");
    editor.save().expect("save");

    let editor2 = EpubEditor::open(&dest).expect("reopen");
    let read_back = editor2.read_item(&first).expect("read back");
    assert!(
        String::from_utf8_lossy(&read_back).contains("Edited content"),
        "edited content must persist after save"
    );
}

#[test]
fn test_list_all_manifest_items() {
    let editor = EpubEditor::open(&fixture_epub()).expect("open");
    let items = editor.manifest_items();
    assert!(!items.is_empty());
    let hrefs: Vec<&str> = items.iter().map(|i| i.href.as_str()).collect();
    assert!(hrefs.iter().any(|h| h.ends_with(".html") || h.ends_with(".xhtml")),
            "manifest must include HTML items: {:?}", hrefs);
}

#[test]
fn test_invalid_path_returns_error() {
    let result = EpubEditor::open(std::path::Path::new("/nonexistent.epub"));
    assert!(result.is_err());
}
