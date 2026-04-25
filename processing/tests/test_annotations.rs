use std::path::PathBuf;

use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::db::annotation_queries;
use xcalibre_processing::db::queries;
use xcalibre_processing::pipeline::ingest::run_ingest;

async fn setup_db() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    pool
}

async fn ingest_fixture(pool: &sqlx::Pool<sqlx::Sqlite>) -> (PathBuf, xcalibre_processing::pipeline::ingest::IngestResult) {
    let path = PathBuf::from("tests/fixtures/fixture_epub.epub");
    let ingest = run_ingest(pool, &path).await.unwrap();
    let path_string = path.to_string_lossy().to_string();
    queries::upsert_local_book(
        pool,
        &ingest.job_id,
        "Fixture Book",
        &["Author Example".to_string()],
        "EPUB",
        Some(path_string.as_str()),
        None,
    )
    .await
    .unwrap();
    (path, ingest)
}

#[tokio::test]
async fn test_create_and_fetch_annotation() {
    let pool = setup_db().await;
    let (_path, ingest) = ingest_fixture(&pool).await;

    let id = annotation_queries::create_annotation(
        &pool,
        &ingest.job_id,
        "highlight",
        "epubcfi(/6/4!/4/2/1:0)",
        Some("Hello world"),
        None,
        "yellow",
    )
    .await
    .unwrap();

    let annotations = annotation_queries::get_annotations(&pool, &ingest.job_id)
        .await
        .unwrap();
    assert_eq!(annotations.len(), 1);
    assert_eq!(annotations[0].id, id);
    assert_eq!(annotations[0].annotation_type, "highlight");
    assert_eq!(annotations[0].selected_text.as_deref(), Some("Hello world"));
    assert!(!annotations[0].synced);
}

#[tokio::test]
async fn test_delete_annotation() {
    let pool = setup_db().await;
    let (_path, ingest) = ingest_fixture(&pool).await;

    let id = annotation_queries::create_annotation(
        &pool,
        &ingest.job_id,
        "bookmark",
        "epubcfi(/6/4!/4/2/1:0)",
        None,
        None,
        "blue",
    )
    .await
    .unwrap();

    annotation_queries::delete_annotation(&pool, &id).await.unwrap();
    let annotations = annotation_queries::get_annotations(&pool, &ingest.job_id)
        .await
        .unwrap();
    assert!(annotations.is_empty());
}

#[tokio::test]
async fn test_unsynced_annotations() {
    let pool = setup_db().await;
    let (_path, ingest) = ingest_fixture(&pool).await;

    let id = annotation_queries::create_annotation(
        &pool,
        &ingest.job_id,
        "note",
        "epubcfi(/6/4!/4/2/1:0)",
        None,
        Some("My note"),
        "green",
    )
    .await
    .unwrap();

    let unsynced = annotation_queries::get_unsynced_annotations(&pool)
        .await
        .unwrap();
    assert_eq!(unsynced.len(), 1);

    annotation_queries::mark_annotation_synced(&pool, &id)
        .await
        .unwrap();
    let unsynced = annotation_queries::get_unsynced_annotations(&pool)
        .await
        .unwrap();
    assert!(unsynced.is_empty());
}
