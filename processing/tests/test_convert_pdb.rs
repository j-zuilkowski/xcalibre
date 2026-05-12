use xcalibre_processing::convert::pdb::{epub_to_pdb, epub_to_pml, epub_to_rb};
use std::path::PathBuf;

fn epub_fixture() -> PathBuf { PathBuf::from("tests/fixtures/fixture_epub.epub") }

#[test]
fn test_epub_to_pdb_creates_file() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.pdb");
    epub_to_pdb(&epub_fixture(), &out).expect("conversion");
    assert!(out.exists(), "PDB output must be created");
    assert!(std::fs::metadata(&out).unwrap().len() > 100, "PDB must be non-empty");
}

#[test]
fn test_epub_to_pml_creates_file() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.pml");
    epub_to_pml(&epub_fixture(), &out).expect("conversion");
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(!content.trim().is_empty(), "PML must not be empty");
}

#[test]
fn test_epub_to_rb_creates_file() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.rb");
    epub_to_rb(&epub_fixture(), &out).expect("conversion");
    assert!(out.exists(), "RocketBook (.rb) output must be created");
    assert!(std::fs::metadata(&out).unwrap().len() > 100);
}
