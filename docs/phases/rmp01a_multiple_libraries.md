# RMP-01a — Multiple Libraries (Red: Schema + Failing Tests)

> HOW TO USE: Write every file exactly as shown, run every shell block.
> DO NOT print code as output — write to disk using your tools.
> Prerequisite: Phase 10 (phase10_commands.md) complete and all tests green.
> TDD role: RED — write schema + failing tests. `cargo test` will fail after this file.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R01a-T01 | Migration 0011 — libraries table | ⬜ |
| R01a-T02 | Migration 0012 — update local_books for library_id | ⬜ |
| R01a-T03 | Failing tests: library CRUD | ⬜ |
| R01a-T04 | Failing tests: library config round-trip | ⬜ |
| R01a-T05 | Failing tests: managed folder layout | ⬜ |

---

## R01a-T01

Write `processing/src/db/migrations/0011_libraries.sql` with this exact content:
```sql
-- Each row is a named local library with its own DB path and cover directory.
-- layout: 'in_place' | 'managed' (S6-C decision: configurable per library)
CREATE TABLE IF NOT EXISTS libraries (
    id          TEXT    NOT NULL PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    name        TEXT    NOT NULL,
    db_path     TEXT    NOT NULL UNIQUE,
    cover_dir   TEXT    NOT NULL,
    layout      TEXT    NOT NULL DEFAULT 'in_place'
                        CHECK (layout IN ('in_place', 'managed')),
    xs_url      TEXT,
    is_active   INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS libraries_is_active ON libraries(is_active);
```

Then run:
```bash
cargo build --workspace
git add processing/src/db/migrations/0011_libraries.sql
git commit -m "R01a-T01: migration 0011 — libraries table"
```

---

## R01a-T02

Write `processing/src/db/migrations/0012_books_library_id.sql` with this exact content:
```sql
-- Add library_id FK to local_books so each book belongs to a library.
-- NULL means the book was imported before multi-library support (legacy).
ALTER TABLE local_books ADD COLUMN library_id TEXT REFERENCES libraries(id) ON DELETE CASCADE;
CREATE INDEX IF NOT EXISTS local_books_library_id ON local_books(library_id);
```

Then run:
```bash
cargo build --workspace
git add processing/src/db/migrations/0012_books_library_id.sql
git commit -m "R01a-T02: migration 0012 — add library_id to local_books"
```

---

## R01a-T03

Write `processing/tests/test_libraries.rs` with this exact content:
```rust
//! Tests for multi-library support (RMP-01).
//! These tests FAIL until rmp01b implements the library queries.

use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::db::library_queries::{
    create_library, delete_library, get_active_library, list_libraries,
    set_active_library, update_library, LibraryRow, NewLibrary,
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
```

Then run:
```bash
cargo test --workspace 2>&1 | head -40
```

Expected: compile error — `library_queries` module does not exist yet. This is correct RED state.

```bash
git add processing/tests/test_libraries.rs
git commit -m "R01a-T03: failing tests for library CRUD"
```

---

## R01a-T04

Write `processing/tests/test_library_config.rs` with this exact content:
```rust
//! Tests for LibraryConfig persistence (RMP-01).
//! These tests FAIL until rmp01b implements LibraryConfig.

use std::path::PathBuf;
use xcalibre_processing::config::{LibraryConfig, LibraryEntry};

#[test]
fn test_library_config_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("config.json");

    let mut cfg = LibraryConfig::new(config_path.clone());
    cfg.add_library(LibraryEntry {
        id: "lib-1".into(),
        name: "Main Library".into(),
        db_path: PathBuf::from("/tmp/main.db"),
        cover_dir: PathBuf::from("/tmp/covers"),
        layout: "in_place".into(),
        xs_url: None,
    });
    cfg.set_active("lib-1");
    cfg.save().expect("save");

    let loaded = LibraryConfig::load(config_path).expect("load");
    assert_eq!(loaded.libraries().len(), 1);
    assert_eq!(loaded.active_id().unwrap(), "lib-1");
    assert_eq!(loaded.libraries()[0].name, "Main Library");
}

#[test]
fn test_library_config_active_switches() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("config.json");

    let mut cfg = LibraryConfig::new(config_path.clone());
    cfg.add_library(LibraryEntry {
        id: "a".into(), name: "A".into(),
        db_path: PathBuf::from("/tmp/a.db"),
        cover_dir: PathBuf::from("/tmp/ac"),
        layout: "in_place".into(), xs_url: None,
    });
    cfg.add_library(LibraryEntry {
        id: "b".into(), name: "B".into(),
        db_path: PathBuf::from("/tmp/b.db"),
        cover_dir: PathBuf::from("/tmp/bc"),
        layout: "managed".into(), xs_url: None,
    });
    cfg.set_active("b");
    cfg.save().unwrap();

    let loaded = LibraryConfig::load(config_path).unwrap();
    assert_eq!(loaded.active_id().unwrap(), "b");
    assert_eq!(loaded.active_library().unwrap().layout, "managed");
}
```

Then run:
```bash
cargo test --workspace 2>&1 | head -40
```

Expected: compile error — `config` module does not yet export `LibraryConfig`. RED state confirmed.

```bash
git add processing/tests/test_library_config.rs
git commit -m "R01a-T04: failing tests for LibraryConfig persistence"
```

---

## R01a-T05

Write `processing/tests/test_managed_layout.rs` with this exact content:
```rust
//! Tests for managed folder layout logic (S6-C, RMP-01).
//! These tests FAIL until rmp01b implements managed_path().

use std::path::PathBuf;
use xcalibre_processing::library::managed_path;

#[test]
fn test_managed_path_basic() {
    let base = PathBuf::from("/library");
    let path = managed_path(&base, "Douglas Adams", "The Hitchhiker's Guide to the Galaxy", 42);
    // Expected: /library/Adams, Douglas/The Hitchhiker's Guide to the Galaxy (42)/
    assert!(path.starts_with(&base));
    let components: Vec<_> = path.components().collect();
    // Author component should be last-name-first
    let author_part = components[components.len() - 2].as_os_str().to_string_lossy();
    assert!(author_part.contains("Adams"));
    // Title component should include the book ID
    let title_part = components[components.len() - 1].as_os_str().to_string_lossy();
    assert!(title_part.contains("42"));
}

#[test]
fn test_managed_path_sanitizes_special_chars() {
    let base = PathBuf::from("/lib");
    let path = managed_path(&base, "Author: With/Slashes", "Title: With\\Backslash?", 1);
    let path_str = path.to_string_lossy();
    // Must not contain raw slash/backslash/colon inside directory names
    assert!(!path_str[base.to_string_lossy().len()..].contains('/') ||
            path_str[base.to_string_lossy().len()..].matches('/').count() == 2,
            "only two directory separators after base: {}", path_str);
}

#[test]
fn test_managed_path_multiple_authors() {
    let base = PathBuf::from("/lib");
    let path = managed_path(&base, "Terry Pratchett & Neil Gaiman", "Good Omens", 7);
    let path_str = path.to_string_lossy();
    assert!(path_str.contains("Pratchett") || path_str.contains("Gaiman"),
            "primary author should appear: {}", path_str);
}
```

Then run:
```bash
cargo test --workspace 2>&1 | head -40
```

Expected: compile error — `library::managed_path` does not exist yet. RED confirmed.

```bash
git add processing/tests/test_managed_layout.rs
git commit -m "R01a-T05: failing tests for managed folder layout"
```

---

### ✅ RED Checkpoint

```bash
cargo test --workspace 2>&1 | grep -E "^error" | head -20
```

All errors should be compile-time `unresolved import` or `module not found` errors — no runtime panics.
Proceed to **rmp01b** to write the implementations.
