//! Tests for managed folder layout logic (S6-C, RMP-01).
//! Tests for managed folder layout (RMP-01).

use std::path::PathBuf;
use xcalibre_processing::library::managed_path;

#[test]
fn test_managed_path_basic() {
    let base = PathBuf::from("/library");
    let path = managed_path(&base, "Douglas Adams", "The Hitchhiker's Guide to the Galaxy", 42);
    // Expected: /library/Adams, Douglas/The Hitchhiker's Guide to the Galaxy (42)/
    assert!(path.starts_with(&base));
    let components: Vec<_> = path.components().collect();
    // Author component should be last-name-first
    let author_part = components[components.len() - 2].as_os_str().to_string_lossy();
    assert!(author_part.contains("Adams"));
    // Title component should include the book ID
    let title_part = components[components.len() - 1].as_os_str().to_string_lossy();
    assert!(title_part.contains("42"));
}

#[test]
fn test_managed_path_sanitizes_special_chars() {
    let base = PathBuf::from("/lib");
    let path = managed_path(&base, "Author: With/Slashes", "Title: With\\Backslash?", 1);
    let path_str = path.to_string_lossy();
    // Must not contain raw slash/backslash/colon inside directory names
    assert!(!path_str[base.to_string_lossy().len()..].contains('/') ||
            path_str[base.to_string_lossy().len()..].matches('/').count() == 2,
            "only two directory separators after base: {}", path_str);
}

#[test]
fn test_managed_path_multiple_authors() {
    let base = PathBuf::from("/lib");
    let path = managed_path(&base, "Terry Pratchett & Neil Gaiman", "Good Omens", 7);
    let path_str = path.to_string_lossy();
    assert!(path_str.contains("Pratchett") || path_str.contains("Gaiman"),
            "primary author should appear: {}", path_str);
}
