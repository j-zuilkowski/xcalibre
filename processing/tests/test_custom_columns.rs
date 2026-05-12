use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::db::custom_columns::{
    create_custom_column, list_custom_columns, update_custom_column,
    delete_custom_column, NewCustomColumn,
};

async fn setup() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_create_and_list_custom_columns() {
    let pool = setup().await;
    let new = NewCustomColumn {
        library_id:      None,
        name:            "read_status".into(),
        label:           "Read Status".into(),
        col_type:        "text".into(),
        is_multiple:     false,
        display_in_grid: true,
    };
    create_custom_column(&pool, &new).await.expect("create");
    let cols = list_custom_columns(&pool, None).await.expect("list");
    assert_eq!(cols.len(), 1);
    assert_eq!(cols[0].name, "read_status");
    assert_eq!(cols[0].label, "Read Status");
}

#[tokio::test]
async fn test_column_types_accepted() {
    let pool = setup().await;
    for (i, col_type) in ["text","integer","float","bool","date","list"].iter().enumerate() {
        let new = NewCustomColumn {
            library_id: None, name: format!("col_{i}"),
            label: format!("Col {i}"), col_type: col_type.to_string(),
            is_multiple: false, display_in_grid: true,
        };
        create_custom_column(&pool, &new).await
            .expect(&format!("create {col_type} column"));
    }
    let cols = list_custom_columns(&pool, None).await.unwrap();
    assert_eq!(cols.len(), 6);
}

#[tokio::test]
async fn test_update_custom_column_label() {
    let pool = setup().await;
    let new = NewCustomColumn {
        library_id: None, name: "rating".into(), label: "Old Label".into(),
        col_type: "integer".into(), is_multiple: false, display_in_grid: true,
    };
    create_custom_column(&pool, &new).await.unwrap();
    let cols = list_custom_columns(&pool, None).await.unwrap();
    let id = cols[0].id.clone();

    update_custom_column(&pool, &id, "New Label", true).await.expect("update");
    let updated = list_custom_columns(&pool, None).await.unwrap();
    assert_eq!(updated[0].label, "New Label");
}

#[tokio::test]
async fn test_delete_custom_column() {
    let pool = setup().await;
    let new = NewCustomColumn {
        library_id: None, name: "to_delete".into(), label: "Delete Me".into(),
        col_type: "text".into(), is_multiple: false, display_in_grid: false,
    };
    create_custom_column(&pool, &new).await.unwrap();
    let cols = list_custom_columns(&pool, None).await.unwrap();
    let id = cols[0].id.clone();

    delete_custom_column(&pool, &id).await.expect("delete");
    let after = list_custom_columns(&pool, None).await.unwrap();
    assert!(after.is_empty());
}
