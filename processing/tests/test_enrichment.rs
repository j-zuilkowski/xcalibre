use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_api::enrichment::open_library::OLBook;
use xcalibre_processing::db::enrichment_cache;
use xcalibre_processing::metadata::{self, enrichment, BookMetadata};

async fn setup_db() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_isbn_from_epub() {
    let path = std::path::PathBuf::from("tests/fixtures/fixture_epub.epub");
    let result = metadata::isbn::from_epub(&path);
    assert_eq!(result.as_deref(), Some("9780000000000"));
}

#[tokio::test]
async fn test_build_suggestions_fills_empty_fields() {
    let existing = BookMetadata::default();
    let ol = OLBook {
        title: Some("Test Book".to_string()),
        ..OLBook::default()
    };
    let gb = xcalibre_api::enrichment::google_books::GBBook::default();
    let suggestions = enrichment::build_suggestions(&existing, &ol, &gb);
    assert!(suggestions.iter().any(|s| s.field == "title" && s.value == "Test Book"));
}

#[tokio::test]
async fn test_enrichment_cache_round_trip() {
    let pool = setup_db().await;
    enrichment_cache::set_cache(&pool, "9780123456789", r#"{"title":"Cached"}"#, "{}")
        .await
        .unwrap();
    let cached = enrichment_cache::get_cache(&pool, "9780123456789")
        .await
        .unwrap();
    assert!(cached.is_some());
    let (ol_json, _) = cached.unwrap();
    assert!(ol_json.contains("Cached"));
}
