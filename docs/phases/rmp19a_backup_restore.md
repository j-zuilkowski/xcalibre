# RMP-19a — Library Backup & Restore + Cross-library Copy (Red: Failing Tests)

> Prerequisite: rmp01b complete (multi-library support).
> TDD role: RED — define backup, restore, and cross-library copy APIs via failing tests.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R19a-T01 | Failing tests: library backup (export to ZIP) | ⬜ |
| R19a-T02 | Failing tests: library restore from backup ZIP | ⬜ |
| R19a-T03 | Failing tests: cross-library book copy | ⬜ |
| R19a-T04 | Failing tests: BackupRestoreDialog component | ⬜ |

---

## R19a-T01

Write `processing/tests/test_backup.rs`:
```rust
use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::backup::{export_library_backup, BackupOptions};
use std::path::PathBuf;

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
        dir.path(), // library root (empty for in-memory test)
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
```

```bash
cargo test --workspace 2>&1 | grep -E "^error" | head -5
```

Expected: `backup` module not found. RED confirmed.

```bash
git add processing/tests/test_backup.rs
git commit -m "R19a-T01: failing tests for library backup"
```

---

## R19a-T02

Write `processing/tests/test_restore.rs`:
```rust
use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::backup::{export_library_backup, restore_library_backup, BackupOptions};

async fn setup() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    // Insert test data
    sqlx::query(
        "INSERT INTO local_books (id, title, authors, format) VALUES ('b1', 'Dune', '[\"Frank Herbert\"]', 'EPUB')"
    ).execute(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_restore_recovers_books() {
    let pool = setup().await;
    let dir  = tempfile::tempdir().unwrap();
    let out  = dir.path().join("backup.xcalibre");

    // Create backup
    export_library_backup(
        &pool, dir.path(), &out,
        &BackupOptions { include_files: false, compress: true },
    ).await.unwrap();

    // Restore into a fresh DB
    let restore_dir = tempfile::tempdir().unwrap();
    let restored_pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("src/db/migrations").run(&restored_pool).await.unwrap();

    restore_library_backup(&out, restore_dir.path(), &restored_pool)
        .await.expect("restore");

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM local_books")
        .fetch_one(&restored_pool).await.unwrap();
    assert_eq!(count, 1, "restored DB must have the backed-up book");
}
```

```bash
cargo test --workspace 2>&1 | grep -E "^error" | head -5
```

RED confirmed.

```bash
git add processing/tests/test_restore.rs
git commit -m "R19a-T02: failing tests for library restore"
```

---

## R19a-T03

Write `processing/tests/test_cross_library_copy.rs`:
```rust
use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::db::book_copy::{copy_book_to_library, CopyBookOptions};

async fn setup() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_copy_book_appears_in_target_library() {
    let pool = setup().await;

    // Insert source library + book
    sqlx::query(
        "INSERT INTO libraries (id, name, db_path, cover_dir, layout, is_active)
         VALUES ('lib1', 'Library One', ':memory:', '/tmp/cov1', 'in_place', 1)"
    ).execute(&pool).await.unwrap();
    sqlx::query(
        "INSERT INTO local_books (id, title, authors, format, library_id)
         VALUES ('b1', 'Dune', '[\"Frank Herbert\"]', 'EPUB', 'lib1')"
    ).execute(&pool).await.unwrap();

    // Insert target library
    sqlx::query(
        "INSERT INTO libraries (id, name, db_path, cover_dir, layout, is_active)
         VALUES ('lib2', 'Library Two', ':memory:', '/tmp/cov2', 'in_place', 0)"
    ).execute(&pool).await.unwrap();

    copy_book_to_library(
        &pool, "b1", "lib2",
        &CopyBookOptions { move_file: false },
    ).await.expect("copy");

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM local_books WHERE library_id='lib2'"
    ).fetch_one(&pool).await.unwrap();
    assert_eq!(count, 1, "copied book must appear in target library");
}

#[tokio::test]
async fn test_copy_preserves_metadata() {
    let pool = setup().await;
    for (lib_id, lib_name) in [("lib1", "Source"), ("lib2", "Target")] {
        sqlx::query(
            "INSERT INTO libraries (id, name, db_path, cover_dir, layout, is_active)
             VALUES (?, ?, ':memory:', '/tmp', 'in_place', 0)"
        ).bind(lib_id).bind(lib_name).execute(&pool).await.unwrap();
    }
    sqlx::query(
        "INSERT INTO local_books (id, title, authors, format, library_id, series)
         VALUES ('b1', 'Foundation', '[\"Isaac Asimov\"]', 'EPUB', 'lib1', 'Foundation Series')"
    ).execute(&pool).await.unwrap();

    copy_book_to_library(&pool, "b1", "lib2", &CopyBookOptions { move_file: false })
        .await.unwrap();

    let series: Option<String> = sqlx::query_scalar(
        "SELECT series FROM local_books WHERE library_id='lib2' LIMIT 1"
    ).fetch_optional(&pool).await.unwrap().flatten();
    assert_eq!(series.as_deref(), Some("Foundation Series"));
}
```

```bash
cargo test --workspace 2>&1 | grep -E "^error" | head -5
```

RED confirmed.

```bash
git add processing/tests/test_cross_library_copy.rs
git commit -m "R19a-T03: failing tests for cross-library book copy"
```

---

## R19a-T04

Write `ui/src/components/BackupRestoreDialog.test.tsx`:
```tsx
import { render, screen, fireEvent, waitFor } from "@testing-library/react"
import { describe, it, expect, vi, beforeEach } from "vitest"
import { BackupRestoreDialog } from "./BackupRestoreDialog"
import { mockInvoke } from "../test/setup"

beforeEach(() => {
  mockInvoke("export_library_backup", "/tmp/backup.xcalibre")
  mockInvoke("restore_library_backup", { books_restored: 42 })
})

describe("BackupRestoreDialog", () => {
  it("shows export and restore tabs", () => {
    render(<BackupRestoreDialog onClose={vi.fn()} />)
    expect(screen.getByTestId("backup-tab")).toBeInTheDocument()
    expect(screen.getByTestId("restore-tab")).toBeInTheDocument()
  })

  it("shows export button in backup tab", () => {
    render(<BackupRestoreDialog onClose={vi.fn()} />)
    expect(screen.getByTestId("export-backup-btn")).toBeInTheDocument()
  })

  it("shows success message after export", async () => {
    render(<BackupRestoreDialog onClose={vi.fn()} />)
    fireEvent.click(screen.getByTestId("export-backup-btn"))
    await waitFor(() =>
      expect(screen.getByTestId("backup-success")).toBeInTheDocument()
    )
  })

  it("shows restore button in restore tab", () => {
    render(<BackupRestoreDialog onClose={vi.fn()} />)
    fireEvent.click(screen.getByTestId("restore-tab"))
    expect(screen.getByTestId("restore-backup-btn")).toBeInTheDocument()
  })

  it("shows books-restored count after restore", async () => {
    render(<BackupRestoreDialog onClose={vi.fn()} />)
    fireEvent.click(screen.getByTestId("restore-tab"))
    fireEvent.click(screen.getByTestId("restore-backup-btn"))
    await waitFor(() =>
      expect(screen.getByTestId("restore-success")).toHaveTextContent("42")
    )
  })
})
```

```bash
cd ui && npm test 2>&1 | grep "Cannot find\|FAIL" | head -5 && cd ..
git add ui/src/components/BackupRestoreDialog.test.tsx
git commit -m "R19a-T04: failing tests for BackupRestoreDialog"
```

---

### ✅ RED Checkpoint

```bash
cargo test --workspace 2>&1 | grep -c "^error"
cd ui && npm test 2>&1 | grep -c "FAIL" && cd ..
```

Both should show failures. Proceed to **rmp19b**.
