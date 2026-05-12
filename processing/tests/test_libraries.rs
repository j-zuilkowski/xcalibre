//! Tests for multi-library support (RMP-01).
//! Tests for library CRUD queries (RMP-01).

use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::db::library_queries::{
    create_library, delete_library, get_active_library, list_libraries,
    set_active_library, update_library, NewLibrary,
};

async fn setup() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_create_and_list_libraries() {
    let pool = setup().await;
    let lib = NewLibrary {
        name: "My Books".into(),
        db_path: "/tmp/my_books.db".into(),
        cover_dir: "/tmp/covers".into(),
        layout: "in_place".into(),
        xs_url: None,
    };
    let id = create_library(&pool, &lib).await.expect("create");
    assert!(!id.is_empty());

    let rows = list_libraries(&pool).await.expect("list");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].name, "My Books");
    assert_eq!(rows[0].layout, "in_place");
}

#[tokio::test]
async fn test_set_active_library() {
    let pool = setup().await;
    let id = create_library(&pool, &NewLibrary {
        name: "Lib A".into(),
        db_path: "/tmp/a.db".into(),
        cover_dir: "/tmp/a_covers".into(),
        layout: "in_place".into(),
        xs_url: None,
    }).await.unwrap();

    set_active_library(&pool, &id).await.expect("set_active");
    let active = get_active_library(&pool).await.expect("get_active").expect("should be Some");
    assert_eq!(active.id, id);
    assert_eq!(active.is_active, true);
}

#[tokio::test]
async fn test_only_one_active_library() {
    let pool = setup().await;
    let id_a = create_library(&pool, &NewLibrary {
        name: "A".into(), db_path: "/tmp/aa.db".into(),
        cover_dir: "/tmp/ac".into(), layout: "in_place".into(), xs_url: None,
    }).await.unwrap();
    let id_b = create_library(&pool, &NewLibrary {
        name: "B".into(), db_path: "/tmp/bb.db".into(),
        cover_dir: "/tmp/bc".into(), layout: "in_place".into(), xs_url: None,
    }).await.unwrap();

    set_active_library(&pool, &id_a).await.unwrap();
    set_active_library(&pool, &id_b).await.unwrap();

    let active = get_active_library(&pool).await.unwrap().unwrap();
    assert_eq!(active.id, id_b, "B should be active now");

    let rows = list_libraries(&pool).await.unwrap();
    let active_count = rows.iter().filter(|r| r.is_active).count();
    assert_eq!(active_count, 1, "exactly one active library");
}

#[tokio::test]
async fn test_update_library_name() {
    let pool = setup().await;
    let id = create_library(&pool, &NewLibrary {
        name: "Old Name".into(), db_path: "/tmp/old.db".into(),
        cover_dir: "/tmp/oc".into(), layout: "in_place".into(), xs_url: None,
    }).await.unwrap();

    update_library(&pool, &id, "New Name", None).await.expect("update");
    let rows = list_libraries(&pool).await.unwrap();
    assert_eq!(rows[0].name, "New Name");
}

#[tokio::test]
async fn test_delete_library() {
    let pool = setup().await;
    let id = create_library(&pool, &NewLibrary {
        name: "Gone".into(), db_path: "/tmp/gone.db".into(),
        cover_dir: "/tmp/gc".into(), layout: "in_place".into(), xs_url: None,
    }).await.unwrap();

    delete_library(&pool, &id).await.expect("delete");
    let rows = list_libraries(&pool).await.unwrap();
    assert!(rows.is_empty());
}
