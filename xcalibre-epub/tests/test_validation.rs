use xcalibre_epub::{Container, validation};
use std::path::PathBuf;
fn fixture(n: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(n)
}
#[test]
fn test_valid_epub_produces_no_errors() {
    let c = Container::open(&fixture("simple.epub")).unwrap();
    let issues = validation::validate(&c).unwrap();
    let errors: Vec<_> = issues.iter()
        .filter(|i| i.severity == validation::Severity::Error).collect();
    assert!(errors.is_empty(), "simple.epub should have no validation errors: {:?}", errors);
}
