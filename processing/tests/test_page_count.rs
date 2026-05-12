use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::db::page_count::{
    update_book_page_count, get_book_page_count,
};

async fn setup() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    sqlx::query(
        "INSERT INTO local_books (id, title, authors_json, format, created_at, updated_at)
         VALUES ('b1', 'Dune', '[]', 'EPUB', '2024-01-01', '2024-01-01')"
    ).execute(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_update_and_get_page_count() {
    let pool = setup().await;
    update_book_page_count(&pool, "b1", 752).await.expect("update");
    let count = get_book_page_count(&pool, "b1").await.expect("get");
    assert_eq!(count, Some(752));
}

#[tokio::test]
async fn test_page_count_null_by_default() {
    let pool = setup().await;
    let count = get_book_page_count(&pool, "b1").await.expect("get");
    assert!(count.is_none(), "page count should be null before first computation");
}

#[tokio::test]
async fn test_update_page_count_overwrites() {
    let pool = setup().await;
    update_book_page_count(&pool, "b1", 100).await.unwrap();
    update_book_page_count(&pool, "b1", 200).await.unwrap();
    let count = get_book_page_count(&pool, "b1").await.unwrap();
    assert_eq!(count, Some(200));
}
