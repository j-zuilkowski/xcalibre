use xcalibre_epub::Container;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name)
}

#[test]
fn test_open_valid_epub() {
    let c = Container::open(&fixture("simple.epub")).expect("open");
    let items = c.manifest_items();
    assert!(!items.is_empty(), "manifest must have at least one item");
}

#[test]
fn test_read_spine_item() {
    let c = Container::open(&fixture("simple.epub")).unwrap();
    let hrefs = c.spine_hrefs();
    assert!(!hrefs.is_empty(), "spine must be non-empty");
    let data = c.read_item(&hrefs[0]).expect("read first spine item");
    assert!(!data.is_empty());
}

#[test]
fn test_write_item_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("modified.epub");
    std::fs::copy(fixture("simple.epub"), &out).unwrap();
    let mut c = Container::open(&out).unwrap();
    c.write_item("test_inject.txt", b"hello world", "text/plain").unwrap();
    c.save().unwrap();

    let c2 = Container::open(&out).unwrap();
    let data = c2.read_item("test_inject.txt").unwrap();
    assert_eq!(&data, b"hello world");
}

#[test]
fn test_save_as_does_not_modify_original() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("copy.epub");
    let c = Container::open(&fixture("simple.epub")).unwrap();
    c.save_as(&out).unwrap();
    assert!(out.exists());
    // Original should still be openable
    let _ = Container::open(&fixture("simple.epub")).unwrap();
}
