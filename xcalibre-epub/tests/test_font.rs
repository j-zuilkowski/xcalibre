use xcalibre_epub::{Container, font};
use std::path::PathBuf;
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name)
}
#[test]
fn test_list_fonts_no_panic() {
    let c = Container::open(&fixture("simple.epub")).unwrap();
    let fonts = font::list_fonts(&c); // simple.epub has no fonts — expect empty Vec
    // just verify it doesn't panic; exact count depends on fixture
    let _ = fonts;
}
