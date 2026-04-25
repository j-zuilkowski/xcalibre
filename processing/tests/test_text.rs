use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::pipeline::ingest::run_ingest;
use xcalibre_processing::pipeline::text::run_text;
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
async fn test_run_text_inserts_row() {
    let pool = setup_db().await;
    let path = PathBuf::from("tests/fixtures/fixture_epub.epub");
    let ingest = run_ingest(&pool, &path).await.unwrap();
    run_text(&pool, &ingest, &path).await.unwrap();
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM job_text WHERE job_id = ?")
        .bind(&ingest.job_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(row.0, 1);
}