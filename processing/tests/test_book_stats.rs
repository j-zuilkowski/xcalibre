use xcalibre_processing::stats::{compute_book_stats, BookStats};
use std::path::PathBuf;

fn epub_fixture() -> PathBuf { PathBuf::from("tests/fixtures/fixture_epub.epub") }

#[test]
fn test_book_stats_word_count() {
    let stats = compute_book_stats(&epub_fixture()).expect("stats");
    assert!(stats.word_count > 0, "word count must be positive");
}

#[test]
fn test_book_stats_page_count_estimate() {
    let stats = compute_book_stats(&epub_fixture()).expect("stats");
    assert!(stats.page_count_estimate > 0, "page count estimate must be positive");
    assert_eq!(
        stats.page_count_estimate,
        ((stats.word_count as f64 / 250.0).ceil() as u32).max(1)
    );
}

#[test]
fn test_book_stats_reading_time_minutes() {
    let stats = compute_book_stats(&epub_fixture()).expect("stats");
    assert!(stats.reading_time_minutes > 0 || stats.word_count < 238,
            "reading time must be positive for non-trivial books");
}

#[test]
fn test_book_stats_character_count() {
    let stats = compute_book_stats(&epub_fixture()).expect("stats");
    assert!(stats.character_count > 0);
    assert!(stats.character_count >= stats.word_count,
            "characters must be >= words");
}

#[test]
fn test_book_stats_invalid_path_errors() {
    let result = compute_book_stats(std::path::Path::new("/nonexistent.epub"));
    assert!(result.is_err());
}
