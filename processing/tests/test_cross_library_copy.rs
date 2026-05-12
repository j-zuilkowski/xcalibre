use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::db::book_copy::{copy_book_to_library, CopyBookOptions};

async fn setup() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_copy_book_appears_in_target_library() {
    let pool = setup().await;

    sqlx::query(
        "INSERT INTO libraries (id, name, db_path, cover_dir, layout, is_active)
         VALUES ('lib1', 'Library One', '/tmp/lib1.db', '/tmp/cov1', 'in_place', 1)"
    ).execute(&pool).await.unwrap();
    sqlx::query(
        "INSERT INTO local_books (id, title, authors_json, format, library_id, created_at, updated_at)
         VALUES ('b1', 'Dune', '[\"Frank Herbert\"]', 'EPUB', 'lib1', '2024-01-01', '2024-01-01')"
    ).execute(&pool).await.unwrap();
    sqlx::query(
        "INSERT INTO libraries (id, name, db_path, cover_dir, layout, is_active)
         VALUES ('lib2', 'Library Two', '/tmp/lib2.db', '/tmp/cov2', 'in_place', 0)"
    ).execute(&pool).await.unwrap();

    copy_book_to_library(
        &pool, "b1", "lib2",
        &CopyBookOptions { move_file: false },
    ).await.expect("copy");

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM local_books WHERE library_id='lib2'"
    ).fetch_one(&pool).await.unwrap();
    assert_eq!(count, 1, "copied book must appear in target library");
}

#[tokio::test]
async fn test_copy_preserves_metadata() {
    let pool = setup().await;
    for (lib_id, lib_name, db_path) in [("lib1", "Source", "/tmp/src.db"), ("lib2", "Target", "/tmp/tgt.db")] {
        sqlx::query(
            "INSERT INTO libraries (id, name, db_path, cover_dir, layout, is_active)
             VALUES (?, ?, ?, '/tmp', 'in_place', 0)"
        ).bind(lib_id).bind(lib_name).bind(db_path).execute(&pool).await.unwrap();
    }
    sqlx::query(
        "INSERT INTO local_books (id, title, authors_json, format, library_id, series_name, created_at, updated_at)
         VALUES ('b1', 'Foundation', '[\"Isaac Asimov\"]', 'EPUB', 'lib1', 'Foundation Series', '2024-01-01', '2024-01-01')"
    ).execute(&pool).await.unwrap();

    copy_book_to_library(&pool, "b1", "lib2", &CopyBookOptions { move_file: false })
        .await.unwrap();

    let series: Option<String> = sqlx::query_scalar(
        "SELECT series_name FROM local_books WHERE library_id='lib2' LIMIT 1"
    ).fetch_optional(&pool).await.unwrap().flatten();
    assert_eq!(series.as_deref(), Some("Foundation Series"));
}
