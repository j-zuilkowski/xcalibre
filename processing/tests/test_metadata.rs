use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::metadata;
use xcalibre_processing::pipeline::ingest::run_ingest;
use xcalibre_processing::pipeline::metadata::run_metadata;
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
async fn test_epub_metadata_ok() {
    let path = PathBuf::from("tests/fixtures/fixture_epub.epub");
    let result = metadata::epub::extract(&path);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_pdf_metadata_stub() {
    let path = PathBuf::from("tests/fixtures/fixture_pdf.pdf");
    let meta = metadata::pdf::extract(&path).unwrap();
    assert!(meta.title.is_none());
}

#[tokio::test]
async fn test_run_metadata_inserts_row() {
    let pool = setup_db().await;
    let path = PathBuf::from("tests/fixtures/fixture_epub.epub");
    let ingest = run_ingest(&pool, &path).await.unwrap();
    run_metadata(&pool, &ingest, &path).await.unwrap();
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM job_metadata WHERE job_id = ?")
        .bind(&ingest.job_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(row.0, 1);
}