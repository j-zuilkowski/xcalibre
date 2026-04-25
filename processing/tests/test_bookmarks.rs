use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::db::queries;

async fn setup_db() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    pool
}

async fn insert_book(pool: &sqlx::SqlitePool) -> String {
    let id  = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO local_books (id, title, authors_json, format, created_at, updated_at)
         VALUES (?,?,?,?,?,?)",
    )
    .bind(&id)
    .bind("Test Book")
    .bind("[]")
    .bind("EPUB")
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .unwrap();
    id
}

#[tokio::test]
async fn test_add_and_list_bookmarks() {
    let pool    = setup_db().await;
    let book_id = insert_book(&pool).await;
    queries::add_bookmark(&pool, &book_id, "0:0:0", None).await.unwrap();
    queries::add_bookmark(&pool, &book_id, "1:5:200", Some("Chapter 2")).await.unwrap();
    let bms = queries::list_bookmarks(&pool, &book_id).await.unwrap();
    assert_eq!(bms.len(), 2);
}

#[tokio::test]
async fn test_delete_bookmark() {
    let pool    = setup_db().await;
    let book_id = insert_book(&pool).await;
    let bm_id = queries::add_bookmark(&pool, &book_id, "0:0:0", None).await.unwrap();
    queries::delete_bookmark(&pool, &bm_id).await.unwrap();
    let bms = queries::list_bookmarks(&pool, &book_id).await.unwrap();
    assert!(bms.is_empty());
}