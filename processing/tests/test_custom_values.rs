use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::db::custom_columns::{
    create_custom_column, NewCustomColumn,
    get_book_custom_value, set_book_custom_value, get_all_book_custom_values,
};

async fn setup() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    sqlx::query(
        "INSERT INTO local_books (id, title, authors_json, format, created_at, updated_at) \
         VALUES ('b1', 'Dune', '[]', 'EPUB', '2024-01-01', '2024-01-01')"
    ).execute(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_set_and_get_custom_value() {
    let pool = setup().await;
    let col = create_custom_column(&pool, &NewCustomColumn {
        library_id: None, name: "my_rating".into(), label: "My Rating".into(),
        col_type: "integer".into(), is_multiple: false, display_in_grid: true,
    }).await.unwrap();

    set_book_custom_value(&pool, "b1", &col.id, Some("5")).await.expect("set");
    let val = get_book_custom_value(&pool, "b1", &col.id).await.expect("get");
    assert_eq!(val.as_deref(), Some("5"));
}

#[tokio::test]
async fn test_set_custom_value_to_null() {
    let pool = setup().await;
    let col = create_custom_column(&pool, &NewCustomColumn {
        library_id: None, name: "notes".into(), label: "Notes".into(),
        col_type: "text".into(), is_multiple: false, display_in_grid: false,
    }).await.unwrap();

    set_book_custom_value(&pool, "b1", &col.id, Some("initial")).await.unwrap();
    set_book_custom_value(&pool, "b1", &col.id, None).await.unwrap();
    let val = get_book_custom_value(&pool, "b1", &col.id).await.unwrap();
    assert!(val.is_none());
}

#[tokio::test]
async fn test_get_all_book_custom_values() {
    let pool = setup().await;
    let col1 = create_custom_column(&pool, &NewCustomColumn {
        library_id: None, name: "col1".into(), label: "C1".into(),
        col_type: "text".into(), is_multiple: false, display_in_grid: true,
    }).await.unwrap();
    let col2 = create_custom_column(&pool, &NewCustomColumn {
        library_id: None, name: "col2".into(), label: "C2".into(),
        col_type: "integer".into(), is_multiple: false, display_in_grid: true,
    }).await.unwrap();

    set_book_custom_value(&pool, "b1", &col1.id, Some("hello")).await.unwrap();
    set_book_custom_value(&pool, "b1", &col2.id, Some("42")).await.unwrap();

    let all = get_all_book_custom_values(&pool, "b1").await.expect("get all");
    assert_eq!(all.len(), 2);
}
