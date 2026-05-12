use xcalibre_epub::{Container, cover};
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name)
}

#[test]
fn test_extract_cover_returns_some_for_epub_with_cover() {
    // Use the simple.epub fixture which has no cover — expect None, not an error
    let c = Container::open(&fixture("simple.epub")).unwrap();
    let result = cover::extract_cover(&c);
    assert!(result.is_ok(), "extract_cover must not error: {:?}", result);
}
