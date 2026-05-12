//! Tests for installed_plugins DB queries (RMP-03).

use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::db::plugin_queries::{
    install_plugin, list_plugins, set_plugin_enabled, uninstall_plugin, InstalledPlugin, NewPlugin,
};

async fn setup() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_install_and_list() {
    let pool = setup().await;
    let plugin = NewPlugin {
        name: "open-library".into(),
        version: "1.0.0".into(),
        api_version: 1,
        plugin_type: "metadata_source".into(),
        dylib_path: "/tmp/open_library.dylib".into(),
    };
    install_plugin(&pool, &plugin).await.expect("install");
    let rows = list_plugins(&pool).await.expect("list");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].name, "open-library");
    assert!(rows[0].enabled);
}

#[tokio::test]
async fn test_set_enabled_false() {
    let pool = setup().await;
    install_plugin(&pool, &NewPlugin {
        name: "test-plugin".into(), version: "1.0.0".into(),
        api_version: 1, plugin_type: "store".into(),
        dylib_path: "/tmp/test.dylib".into(),
    }).await.unwrap();

    let rows = list_plugins(&pool).await.unwrap();
    let id = rows[0].id.clone();
    set_plugin_enabled(&pool, &id, false).await.expect("disable");
    let rows = list_plugins(&pool).await.unwrap();
    assert!(!rows[0].enabled);
}

#[tokio::test]
async fn test_uninstall_removes_row() {
    let pool = setup().await;
    install_plugin(&pool, &NewPlugin {
        name: "remove-me".into(), version: "1.0.0".into(),
        api_version: 1, plugin_type: "conversion_output".into(),
        dylib_path: "/tmp/rm.dylib".into(),
    }).await.unwrap();
    let rows = list_plugins(&pool).await.unwrap();
    uninstall_plugin(&pool, &rows[0].id).await.expect("uninstall");
    assert!(list_plugins(&pool).await.unwrap().is_empty());
}
