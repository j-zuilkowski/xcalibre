//! Tests that update_book_metadata() writes a sidecar .opf file when
//! the book's directory uses managed layout.

use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::db::queries::update_book_metadata_with_opf;
use xcalibre_processing::metadata::BookMetadata;

async fn setup() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_opf_written_on_metadata_update() {
    let pool = setup().await;
    let dir = tempfile::tempdir().unwrap();
    let book_dir = dir.path().join("Author/Title (1)");
    std::fs::create_dir_all(&book_dir).unwrap();
    let book_path = book_dir.join("book.epub");
    std::fs::write(&book_path, b"placeholder").unwrap();
    let now = chrono::Utc::now().to_rfc3339();

    // Insert a book with a known local_path inside book_dir
    sqlx::query(
        "INSERT INTO local_books (id, title, authors_json, format, local_path,
          progress_percent, created_at, updated_at) VALUES ('bk1','Old','[\"A\"]','EPUB',?,
          0, ?, ?)",
    )
    .bind(book_path.to_string_lossy().as_ref())
    .bind(&now)
    .bind(&now)
    .execute(&pool).await.unwrap();

    let meta = BookMetadata {
        title: Some("New Title".into()),
        authors: vec!["New Author".into()],
        ..Default::default()
    };
    update_book_metadata_with_opf(&pool, "bk1", &meta).await.expect("update");

    let opf_path = book_dir.join("metadata.opf");
    assert!(opf_path.exists(), "OPF sidecar should have been written");
    let content = std::fs::read_to_string(&opf_path).unwrap();
    assert!(content.contains("New Title"));
}
