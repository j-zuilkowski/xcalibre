use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::backup::{export_library_backup, restore_library_backup, BackupOptions};

async fn setup() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    sqlx::query(
        "INSERT INTO local_books (id, title, authors_json, format, created_at, updated_at) \
         VALUES ('b1', 'Dune', '[\"Frank Herbert\"]', 'EPUB', '2024-01-01', '2024-01-01')"
    ).execute(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_restore_recovers_books() {
    let pool = setup().await;
    let dir  = tempfile::tempdir().unwrap();
    let out  = dir.path().join("backup.xcalibre");

    export_library_backup(
        &pool, dir.path(), &out,
        &BackupOptions { include_files: false, compress: true },
    ).await.unwrap();

    let restore_dir = tempfile::tempdir().unwrap();
    let restored_pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("src/db/migrations").run(&restored_pool).await.unwrap();

    restore_library_backup(&out, restore_dir.path(), &restored_pool)
        .await.expect("restore");

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM local_books")
        .fetch_one(&restored_pool).await.unwrap();
    assert_eq!(count, 1, "restored DB must have the backed-up book");
}
