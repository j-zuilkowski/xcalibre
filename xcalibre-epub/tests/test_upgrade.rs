use xcalibre_epub::{Container, upgrade};
use std::path::PathBuf;
fn fixture(n: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(n)
}
#[test]
fn test_epub2_upgrade_produces_epub3_version() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("upgraded.epub");
    std::fs::copy(fixture("simple.epub"), &out).unwrap();
    let mut c = Container::open(&out).unwrap();
    upgrade::epub2_to_epub3(&mut c).unwrap();
    c.save().unwrap();
    // Re-open and check the OPF version attribute
    let c2 = Container::open(&out).unwrap();
    let opf_bytes = c2.read_item(c2.opf_path()).unwrap();
    let opf_str = String::from_utf8_lossy(&opf_bytes);
    assert!(opf_str.contains("version=\"3."), "OPF version must be 3.x after upgrade: {}", opf_str);
}
