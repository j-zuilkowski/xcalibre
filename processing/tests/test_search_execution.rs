//! Tests for executing parsed queries against a real SQLite library DB.
//! These tests FAIL until rmp02b implements execute_query().

use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::search::execute_query;

async fn setup() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    // Insert two fixture books using actual local_books column names
    sqlx::query(
        "INSERT INTO local_books (id, title, authors_json, format, local_path,
          cover_path, progress_percent, last_opened_at, created_at, updated_at,
          publisher, series_name, series_index, description)
         VALUES
         ('b1','The Rust Programming Language','[\"Steve Klabnik\"]','EPUB','/f1',
          NULL,0,NULL,'2024-01-01','2024-01-01',
          'No Starch','Rust Series',1,'A book about Rust'),
         ('b2','Good Omens','[\"Terry Pratchett\",\"Neil Gaiman\"]','EPUB','/f2',
          NULL,0,NULL,'2024-01-01','2024-01-01',
          'Gollancz',NULL,NULL,'A comedy fantasy')",
    )
    .execute(&pool).await.unwrap();
    // Add tags for book b2
    sqlx::query("INSERT OR IGNORE INTO tags (id, name) VALUES (1, 'fantasy')")
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT OR IGNORE INTO tags (id, name) VALUES (2, 'comedy')")
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT OR IGNORE INTO book_tags (book_id, tag_id) VALUES ('b2', 1)")
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT OR IGNORE INTO book_tags (book_id, tag_id) VALUES ('b2', 2)")
        .execute(&pool).await.unwrap();
    // Add tags for book b1
    sqlx::query("INSERT OR IGNORE INTO tags (id, name) VALUES (3, 'programming')")
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT OR IGNORE INTO tags (id, name) VALUES (4, 'systems')")
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT OR IGNORE INTO book_tags (book_id, tag_id) VALUES ('b1', 3)")
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT OR IGNORE INTO book_tags (book_id, tag_id) VALUES ('b1', 4)")
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
    assert!(ids.contains(&"b2".to_string()));
}

#[tokio::test]
async fn test_and_narrows_results() {
    let pool = setup().await;
    let ids = execute_query(&pool, "author:Klabnik AND title:Rust").await.unwrap();
    assert_eq!(ids, vec!["b1"]);
}

#[tokio::test]
async fn test_not_excludes_results() {
    let pool = setup().await;
    let ids = execute_query(&pool, "format:EPUB NOT title:Rust").await.unwrap();
    assert_eq!(ids, vec!["b2"]);
}

#[tokio::test]
async fn test_no_match_returns_empty() {
    let pool = setup().await;
    let ids = execute_query(&pool, "title:Nonexistent").await.unwrap();
    assert!(ids.is_empty());
}
