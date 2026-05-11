# RMP-03a — Plugin System (Red: Failing Tests)

> HOW TO USE: Write every file exactly as shown, run every shell block.
> DO NOT print code as output — write to disk using your tools.
> Prerequisite: rmp02b complete and all tests green.
> TDD role: RED — write failing tests for plugin loading, ABI guard, vtable dispatch.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R03a-T01 | Migration 0013 — installed_plugins table | ⬜ |
| R03a-T02 | Failing tests: plugin SDK types | ⬜ |
| R03a-T03 | Failing tests: plugin loader (version guard) | ⬜ |
| R03a-T04 | Failing tests: plugin manager DB queries | ⬜ |

---

## R03a-T01

Write `processing/src/db/migrations/0013_plugins.sql` with this exact content:
```sql
CREATE TABLE IF NOT EXISTS installed_plugins (
    id           TEXT NOT NULL PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    name         TEXT NOT NULL UNIQUE,
    version      TEXT NOT NULL,
    api_version  INTEGER NOT NULL,
    plugin_type  TEXT NOT NULL CHECK (plugin_type IN ('metadata_source','conversion_output','store')),
    dylib_path   TEXT NOT NULL,
    enabled      INTEGER NOT NULL DEFAULT 1,
    installed_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at   TEXT NOT NULL DEFAULT (datetime('now'))
);
```

Then run:
```bash
cargo build --workspace
git add processing/src/db/migrations/0013_plugins.sql
git commit -m "R03a-T01: migration 0013 — installed_plugins table"
```

---

## R03a-T02

Write `processing/tests/test_plugin_sdk.rs` with this exact content:
```rust
//! Tests for xcalibre-plugin-sdk types and PLUGIN_API_VERSION guard.
//! These tests FAIL until rmp03b creates the xcalibre-plugin-sdk crate.

use xcalibre_plugin_sdk::{PLUGIN_API_VERSION, PluginType, PluginMetadata};

#[test]
fn test_api_version_is_nonzero() {
    assert!(PLUGIN_API_VERSION > 0, "API version must be > 0");
}

#[test]
fn test_plugin_metadata_fields() {
    let meta = PluginMetadata {
        name: "test-plugin".into(),
        version: "1.0.0".into(),
        api_version: PLUGIN_API_VERSION,
        plugin_type: PluginType::MetadataSource,
    };
    assert_eq!(meta.name, "test-plugin");
    assert_eq!(meta.api_version, PLUGIN_API_VERSION);
}

#[test]
fn test_plugin_type_display() {
    assert_eq!(PluginType::MetadataSource.as_str(), "metadata_source");
    assert_eq!(PluginType::ConversionOutput.as_str(), "conversion_output");
    assert_eq!(PluginType::Store.as_str(), "store");
}
```

Then run:
```bash
cargo test --workspace 2>&1 | grep -E "^error" | head -10
```

Expected: `xcalibre_plugin_sdk` crate not found. RED confirmed.

```bash
git add processing/tests/test_plugin_sdk.rs
git commit -m "R03a-T02: failing tests for plugin SDK types"
```

---

## R03a-T03

Write `processing/tests/test_plugin_loader.rs` with this exact content:
```rust
//! Tests for plugin ZIP extraction and ABI version guard.
//! These tests FAIL until rmp03b implements PluginLoader.

use std::path::PathBuf;
use xcalibre_processing::plugins::loader::{install_plugin_zip, PluginLoadError};

#[test]
fn test_version_mismatch_is_rejected() {
    // A ZIP with a dylib that exports the wrong PLUGIN_API_VERSION must be refused.
    // We simulate this with a crafted fixture.
    let fixture = PathBuf::from("tests/fixtures/plugin_bad_version.zip");
    if !fixture.exists() { return; } // fixture generated in rmp03b
    let dir = tempfile::tempdir().unwrap();
    let result = install_plugin_zip(&fixture, dir.path());
    assert!(
        matches!(result, Err(PluginLoadError::ApiVersionMismatch { .. })),
        "expected ApiVersionMismatch, got: {:?}", result
    );
}

#[test]
fn test_missing_manifest_is_rejected() {
    // A ZIP with no plugin.json manifest must be rejected.
    let dir = tempfile::tempdir().unwrap();
    let zip_path = dir.path().join("no_manifest.zip");
    {
        let f = std::fs::File::create(&zip_path).unwrap();
        let mut z = zip::ZipWriter::new(f);
        let opts = zip::write::FileOptions::default();
        z.start_file("dummy.txt", opts).unwrap();
        z.write_all(b"hello").unwrap();
        z.finish().unwrap();
    }
    let out_dir = tempfile::tempdir().unwrap();
    let result = install_plugin_zip(&zip_path, out_dir.path());
    assert!(
        matches!(result, Err(PluginLoadError::MissingManifest)),
        "expected MissingManifest, got: {:?}", result
    );
}
```

Then run:
```bash
cargo test --workspace 2>&1 | grep -E "^error" | head -10
```

Expected: compile error — `plugins::loader` does not exist. RED confirmed.

```bash
git add processing/tests/test_plugin_loader.rs
git commit -m "R03a-T03: failing tests for plugin loader ABI guard"
```

---

## R03a-T04

Write `processing/tests/test_plugin_db.rs` with this exact content:
```rust
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
```

Then run:
```bash
cargo test --workspace 2>&1 | grep -E "^error" | head -10
```

Expected: `db::plugin_queries` not found. RED confirmed.

```bash
git add processing/tests/test_plugin_db.rs
git commit -m "R03a-T04: failing tests for plugin DB queries"
```

---

### ✅ RED Checkpoint

```bash
cargo test --workspace 2>&1 | grep -c "^error"
```

Errors expected. Proceed to **rmp03b**.
