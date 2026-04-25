use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::pipeline::ingest::run_ingest;
use xcalibre_processing::pipeline::push::run_push;
use std::path::PathBuf;

async fn setup_db() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_push_skipped_without_client() {
    let pool   = setup_db().await;
    let path   = PathBuf::from("tests/fixtures/fixture_epub.epub");
    let ingest = run_ingest(&pool, &path).await.unwrap();
    let result = run_push(&pool, &ingest, serde_json::Value::Null, None).await;
    assert!(result.is_ok());
}