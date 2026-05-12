use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::backup::{export_library_backup, BackupOptions};

async fn setup() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_backup_creates_zip() {
    let pool = setup().await;
    let dir  = tempfile::tempdir().unwrap();
    let out  = dir.path().join("backup.xcalibre");

    export_library_backup(
        &pool,
        dir.path(),
        &out,
        &BackupOptions { include_files: false, compress: true },
    ).await.expect("backup");

    assert!(out.exists(), "backup file must be created");
    let bytes = std::fs::read(&out).unwrap();
    assert_eq!(&bytes[..2], b"PK", "backup must be a ZIP file");
}

#[tokio::test]
async fn test_backup_contains_database() {
    let pool = setup().await;
    let dir  = tempfile::tempdir().unwrap();
    let out  = dir.path().join("backup.xcalibre");

    export_library_backup(
        &pool, dir.path(), &out,
        &BackupOptions { include_files: false, compress: true },
    ).await.unwrap();

    let file = std::fs::File::open(&out).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    let names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().to_string())
        .collect();
    assert!(
        names.iter().any(|n| n.ends_with(".db") || n == "library.db"),
        "backup must contain the SQLite database: {:?}", names
    );
}

#[tokio::test]
async fn test_backup_includes_metadata_json() {
    let pool = setup().await;
    let dir  = tempfile::tempdir().unwrap();
    let out  = dir.path().join("backup.xcalibre");

    export_library_backup(
        &pool, dir.path(), &out,
        &BackupOptions { include_files: false, compress: true },
    ).await.unwrap();

    let file = std::fs::File::open(&out).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    let names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().to_string())
        .collect();
    assert!(
        names.iter().any(|n| n.contains("manifest") || n.ends_with(".json")),
        "backup must contain a manifest JSON: {:?}", names
    );
}
