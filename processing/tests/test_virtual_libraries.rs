use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::db::vlib_queries::{
    create_virtual_library, list_virtual_libraries, update_virtual_library,
    delete_virtual_library, VirtualLibrary, NewVirtualLibrary,
};

async fn setup() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_create_and_list_virtual_libraries() {
    let pool = setup().await;
    let new = NewVirtualLibrary {
        library_id:  None,
        name:        "Science Fiction".into(),
        search_expr: "tag:science-fiction".into(),
        sort_field:  "title".into(),
        sort_asc:    true,
    };
    create_virtual_library(&pool, &new).await.expect("create");
    let libs = list_virtual_libraries(&pool, None).await.expect("list");
    assert_eq!(libs.len(), 1);
    assert_eq!(libs[0].name, "Science Fiction");
    assert_eq!(libs[0].search_expr, "tag:science-fiction");
}

#[tokio::test]
async fn test_update_virtual_library() {
    let pool = setup().await;
    let new = NewVirtualLibrary {
        library_id:  None,
        name:        "Old Name".into(),
        search_expr: "author:tolkien".into(),
        sort_field:  "title".into(),
        sort_asc:    true,
    };
    create_virtual_library(&pool, &new).await.unwrap();
    let libs = list_virtual_libraries(&pool, None).await.unwrap();
    let id = &libs[0].id.clone();

    update_virtual_library(&pool, id, "New Name", "author:tolkien", "added_at", false)
        .await.expect("update");

    let updated = list_virtual_libraries(&pool, None).await.unwrap();
    assert_eq!(updated[0].name, "New Name");
    assert_eq!(updated[0].sort_field, "added_at");
    assert!(!updated[0].sort_asc);
}

#[tokio::test]
async fn test_delete_virtual_library() {
    let pool = setup().await;
    let new = NewVirtualLibrary {
        library_id:  None,
        name:        "To Delete".into(),
        search_expr: "format:epub".into(),
        sort_field:  "title".into(),
        sort_asc:    true,
    };
    create_virtual_library(&pool, &new).await.unwrap();
    let libs = list_virtual_libraries(&pool, None).await.unwrap();
    let id = libs[0].id.clone();

    delete_virtual_library(&pool, &id).await.expect("delete");
    let after = list_virtual_libraries(&pool, None).await.unwrap();
    assert!(after.is_empty());
}

#[tokio::test]
async fn test_virtual_library_scoped_to_library() {
    let pool = setup().await;
    for (name, lib_id) in [("A", Some("lib1")), ("B", Some("lib2")), ("C", None)] {
        let new = NewVirtualLibrary {
            library_id:  lib_id.map(String::from),
            name:        name.into(),
            search_expr: "format:epub".into(),
            sort_field:  "title".into(),
            sort_asc:    true,
        };
        create_virtual_library(&pool, &new).await.unwrap();
    }
    let lib1_only = list_virtual_libraries(&pool, Some("lib1")).await.unwrap();
    assert_eq!(lib1_only.len(), 1);
    assert_eq!(lib1_only[0].name, "A");

    let global = list_virtual_libraries(&pool, None).await.unwrap();
    assert_eq!(global.len(), 3);
}
