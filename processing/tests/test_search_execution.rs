//! Tests for executing parsed queries against a real SQLite library DB.
//! These tests FAIL until rmp02b implements execute_query().

use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::search::execute_query;

async fn setup() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    // Insert two fixture books
    sqlx::query(
        "INSERT INTO local_books (id, title, authors_json, format, file_path,
          file_sha256, status, progress_percent, cover_path, last_opened_at,
          series, series_index, publisher, language, tags_json)
         VALUES
         ('b1','The Rust Programming Language','[\"Steve Klabnik\"]','EPUB','/f1',
          'sha1','READY',0,NULL,NULL,'Rust Series',1,'No Starch','en','[\"programming\",\"systems\"]'),
         ('b2','Good Omens','[\"Terry Pratchett\",\"Neil Gaiman\"]','EPUB','/f2',
          'sha2','READY',0,NULL,NULL,NULL,NULL,'Gollancz','en','[\"fantasy\",\"comedy\"]')",
    )
    .execute(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_bare_term_matches_title() {
    let pool = setup().await;
    let ids = execute_query(&pool, "Rust").await.expect("execute");
    assert!(ids.contains(&"b1".to_string()), "bare 'Rust' should match b1");
}

#[tokio::test]
async fn test_field_title_match() {
    let pool = setup().await;
    let ids = execute_query(&pool, "title:\"Good Omens\"").await.unwrap();
    assert_eq!(ids, vec!["b2"]);
}

#[tokio::test]
async fn test_field_author_match() {
    let pool = setup().await;
    let ids = execute_query(&pool, "author:Gaiman").await.unwrap();
    assert_eq!(ids, vec!["b2"]);
}

#[tokio::test]
async fn test_field_tag_match() {
    let pool = setup().await;
    let ids = execute_query(&pool, "tag:fantasy").await.unwrap();
    assert_eq!(ids, vec!["b2"]);
}

#[tokio::test]
async fn test_and_narrows_results() {
    let pool = setup().await;
    // Only b1 matches both
    let ids = execute_query(&pool, "author:Klabnik AND tag:programming").await.unwrap();
    assert_eq!(ids, vec!["b1"]);
}

#[tokio::test]
async fn test_not_excludes_results() {
    let pool = setup().await;
    let ids = execute_query(&pool, "format:EPUB NOT tag:fantasy").await.unwrap();
    assert!(ids.contains(&"b1".to_string()));
    assert!(!ids.contains(&"b2".to_string()));
}

#[tokio::test]
async fn test_no_match_returns_empty() {
    let pool = setup().await;
    let ids = execute_query(&pool, "title:Nonexistent").await.unwrap();
    assert!(ids.is_empty());
}
