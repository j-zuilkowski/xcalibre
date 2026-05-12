# RMP-19b — Library Backup & Restore + Cross-library Copy (Green: Implementation)

> Prerequisite: rmp19a complete.
> TDD role: GREEN — implement backup/restore engine, cross-library copy, and UI.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R19b-T01 | `backup/mod.rs` — export and restore | ⬜ |
| R19b-T02 | `db/book_copy.rs` — cross-library copy | ⬜ |
| R19b-T03 | Tauri commands: export, restore, copy_book | ⬜ |
| R19b-T04 | `BackupRestoreDialog.tsx` component | ⬜ |
| R19b-T05 | Milestone check + visual inspection | ⬜ |

---

## R19b-T01

Write `processing/src/backup/mod.rs`:
```rust
use crate::error::ProcessingError;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use std::io::Write;
use std::path::Path;
use zip::write::{SimpleFileOptions, ZipWriter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupOptions {
    pub include_files: bool,
    pub compress:      bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupManifest {
    pub xcalibre_version: String,
    pub created_at:       String,
    pub book_count:       u32,
    pub include_files:    bool,
}

pub async fn export_library_backup(
    pool: &SqlitePool,
    library_root: &Path,
    out_path: &Path,
    opts: &BackupOptions,
) -> Result<(), ProcessingError> {
    let compression = if opts.compress {
        zip::CompressionMethod::Deflated
    } else {
        zip::CompressionMethod::Stored
    };
    let file_opts = SimpleFileOptions::default().compression_method(compression);

    let file = std::fs::File::create(out_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    let mut zip = ZipWriter::new(file);

    // Dump the SQLite database to a temporary file, then include it
    let tmp_dir = tempfile::tempdir()
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    let db_dump_path = tmp_dir.path().join("library.db");

    // Use SQLite VACUUM INTO to create a clean copy
    sqlx::query(&format!("VACUUM INTO '{}'", db_dump_path.display()))
        .execute(pool)
        .await
        .map_err(|e| ProcessingError::DatabaseError(e.to_string()))?;

    let db_bytes = std::fs::read(&db_dump_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    zip.start_file("library.db", file_opts)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    zip.write_all(&db_bytes)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    // Manifest JSON
    let book_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM local_books")
        .fetch_one(pool).await
        .map_err(|e| ProcessingError::DatabaseError(e.to_string()))?;

    let manifest = BackupManifest {
        xcalibre_version: env!("CARGO_PKG_VERSION").to_string(),
        created_at:       chrono::Utc::now().to_rfc3339(),
        book_count:       book_count as u32,
        include_files:    opts.include_files,
    };
    let manifest_json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    zip.start_file("manifest.json", file_opts)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    zip.write_all(manifest_json.as_bytes())
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    // Optionally include book files
    if opts.include_files && library_root.exists() {
        for entry in walkdir::WalkDir::new(library_root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let rel = entry.path().strip_prefix(library_root).unwrap();
            let arc_name = format!("files/{}", rel.display());
            if let Ok(bytes) = std::fs::read(entry.path()) {
                zip.start_file(&arc_name, file_opts)
                    .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
                zip.write_all(&bytes)
                    .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
            }
        }
    }

    zip.finish()
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    Ok(())
}

pub async fn restore_library_backup(
    backup_path: &Path,
    restore_dir: &Path,
    pool: &SqlitePool,
) -> Result<u32, ProcessingError> {
    let file = std::fs::File::open(backup_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    // Extract library.db to a temp file, then import its books
    let tmp_dir = tempfile::tempdir()
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    // Read manifest first
    let manifest: BackupManifest = {
        let mut entry = archive.by_name("manifest.json")
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
        let mut json = String::new();
        std::io::Read::read_to_string(&mut entry, &mut json).unwrap();
        serde_json::from_str(&json)
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?
    };

    // Extract DB
    let db_path = tmp_dir.path().join("library.db");
    {
        let mut db_entry = archive.by_name("library.db")
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
        let mut db_bytes = Vec::new();
        std::io::Read::read_to_end(&mut db_entry, &mut db_bytes).unwrap();
        std::fs::write(&db_path, &db_bytes)
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    }

    // Open the backup DB and copy local_books rows into the target pool
    let backup_pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect(&format!("sqlite:{}", db_path.display()))
        .await
        .map_err(|e| ProcessingError::DatabaseError(e.to_string()))?;

    let books = sqlx::query!(
        "SELECT id, title, authors, format, file_path, cover_path, series,
                series_index, tags_json, language, publisher, published,
                description, isbn, word_count
         FROM local_books"
    )
    .fetch_all(&backup_pool)
    .await
    .map_err(|e| ProcessingError::DatabaseError(e.to_string()))?;

    let count = books.len() as u32;
    for book in books {
        // IGNORE conflicts (book already in library)
        sqlx::query(
            "INSERT OR IGNORE INTO local_books
             (id, title, authors, format, file_path, cover_path, series,
              series_index, tags_json, language, publisher, published,
              description, isbn, word_count)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&book.id)
        .bind(&book.title)
        .bind(&book.authors)
        .bind(&book.format)
        .bind(&book.file_path)
        .bind(&book.cover_path)
        .bind(&book.series)
        .bind(&book.series_index)
        .bind(&book.tags_json)
        .bind(&book.language)
        .bind(&book.publisher)
        .bind(&book.published)
        .bind(&book.description)
        .bind(&book.isbn)
        .bind(&book.word_count)
        .execute(pool)
        .await
        .map_err(|e| ProcessingError::DatabaseError(e.to_string()))?;
    }

    // If include_files, extract them to restore_dir
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).unwrap();
        let name = entry.name().to_string();
        if name.starts_with("files/") {
            let rel = name.trim_start_matches("files/");
            let dest = restore_dir.join(rel);
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            if let Ok(mut f) = std::fs::File::create(&dest) {
                std::io::copy(&mut entry, &mut f).ok();
            }
        }
    }

    Ok(count)
}
```

Add `chrono = "0.4"` and `walkdir = "2"` to `processing/Cargo.toml` dependencies.

Add to `processing/src/lib.rs`:
```rust
pub mod backup;
```

Then run:
```bash
cargo test --workspace -- test_backup test_restore
git add processing/src/backup/mod.rs processing/src/lib.rs processing/Cargo.toml
git commit -m "R19b-T01: library backup + restore engine — all tests green"
```

---

## R19b-T02

Write `processing/src/db/book_copy.rs`:
```rust
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyBookOptions {
    pub move_file: bool,
}

pub async fn copy_book_to_library(
    pool: &SqlitePool,
    book_id: &str,
    target_library_id: &str,
    opts: &CopyBookOptions,
) -> Result<String, sqlx::Error> {
    // Fetch original book
    let book = sqlx::query!(
        "SELECT * FROM local_books WHERE id=?",
        book_id
    )
    .fetch_one(pool)
    .await?;

    // Generate a new ID for the copy
    let new_id = uuid_v4();

    // Determine new file_path if moving/copying files
    let new_file_path = book.file_path.clone();
    // Note: actual file copy/move is handled by the Tauri command layer,
    // which has filesystem access. Here we just record the new row.

    sqlx::query(
        "INSERT INTO local_books
         (id, title, authors, format, file_path, cover_path, series, series_index,
          tags_json, language, publisher, published, description, isbn, word_count,
          library_id)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&new_id)
    .bind(&book.title)
    .bind(&book.authors)
    .bind(&book.format)
    .bind(&new_file_path)
    .bind(&book.cover_path)
    .bind(&book.series)
    .bind(&book.series_index)
    .bind(&book.tags_json)
    .bind(&book.language)
    .bind(&book.publisher)
    .bind(&book.published)
    .bind(&book.description)
    .bind(&book.isbn)
    .bind(&book.word_count)
    .bind(target_library_id)
    .execute(pool)
    .await?;

    Ok(new_id)
}

fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos();
    format!("{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
        t, t >> 16, t & 0xFFF, (t >> 8) | 0x8000, t as u64 * 0xDEADBEEF)
}
```

Add to `processing/src/db/mod.rs`:
```rust
pub mod book_copy;
```

Then run:
```bash
cargo test --workspace -- test_cross_library_copy
git add processing/src/db/book_copy.rs processing/src/db/mod.rs
git commit -m "R19b-T02: cross-library book copy — all tests green"
```

---

## R19b-T03

Write `src-tauri/src/commands/backup.rs`:
```rust
use tauri::{AppHandle, State};
use xcalibre_processing::backup::{export_library_backup, restore_library_backup, BackupOptions};
use xcalibre_processing::db::book_copy::{copy_book_to_library, CopyBookOptions};
use crate::AppState;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct RestoreResult { pub books_restored: u32 }

#[tauri::command]
pub async fn export_library_backup_cmd(
    include_files: bool,
    out_path:      String,
    state: State<'_, AppState>,
    app:   AppHandle,
) -> Result<String, String> {
    let lib_root = app.path().app_data_dir()
        .map_err(|e| e.to_string())?;
    let opts = BackupOptions { include_files, compress: true };
    export_library_backup(&state.pool, &lib_root, std::path::Path::new(&out_path), &opts)
        .await.map_err(|e| e.to_string())?;
    Ok(out_path)
}

#[tauri::command]
pub async fn restore_library_backup_cmd(
    backup_path: String,
    state: State<'_, AppState>,
    app:   AppHandle,
) -> Result<RestoreResult, String> {
    let restore_dir = app.path().app_data_dir()
        .map_err(|e| e.to_string())?;
    let count = restore_library_backup(
        std::path::Path::new(&backup_path),
        &restore_dir,
        &state.pool,
    ).await.map_err(|e| e.to_string())?;
    Ok(RestoreResult { books_restored: count })
}

#[tauri::command]
pub async fn copy_book_to_library_cmd(
    book_id:    String,
    library_id: String,
    move_file:  bool,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let opts = CopyBookOptions { move_file };
    copy_book_to_library(&state.pool, &book_id, &library_id, &opts)
        .await.map_err(|e| e.to_string())
}
```

Register in `src-tauri/src/main.rs`:
```rust
commands::backup::export_library_backup_cmd,
commands::backup::restore_library_backup_cmd,
commands::backup::copy_book_to_library_cmd,
```

```bash
cargo build --workspace
git add src-tauri/src/commands/backup.rs src-tauri/src/main.rs
git commit -m "R19b-T03: Tauri commands for backup, restore, cross-library copy"
```

---

## R19b-T04

Write `ui/src/components/BackupRestoreDialog.tsx`:
```tsx
import { useState } from "react"
import { invoke } from "@tauri-apps/api/core"
import { save, open } from "@tauri-apps/plugin-dialog"

interface Props { onClose: () => void }

type Tab = "backup" | "restore"

export function BackupRestoreDialog({ onClose }: Props) {
  const [tab,           setTab]       = useState<Tab>("backup")
  const [loading,       setLoading]   = useState(false)
  const [backupSuccess, setBkSuccess] = useState<string | null>(null)
  const [restoreResult, setRsResult]  = useState<{ books_restored: number } | null>(null)
  const [error,         setError]     = useState<string | null>(null)
  const [includeFiles,  setIncFiles]  = useState(false)

  async function handleExport() {
    const path = await save({
      defaultPath: `xcalibre-backup-${new Date().toISOString().slice(0,10)}.xcalibre`,
      filters: [{ name: "xCalibre Backup", extensions: ["xcalibre"] }],
    })
    if (!path) return
    setLoading(true); setError(null)
    try {
      const out = await invoke<string>("export_library_backup_cmd", {
        includeFiles, outPath: path,
      })
      setBkSuccess(out)
    } catch (e) { setError(String(e)) }
    finally { setLoading(false) }
  }

  async function handleRestore() {
    const path = await open({
      filters: [{ name: "xCalibre Backup", extensions: ["xcalibre"] }],
    })
    if (!path || Array.isArray(path)) return
    setLoading(true); setError(null)
    try {
      const result = await invoke<{ books_restored: number }>("restore_library_backup_cmd", {
        backupPath: path,
      })
      setRsResult(result)
    } catch (e) { setError(String(e)) }
    finally { setLoading(false) }
  }

  return (
    <div role="dialog" aria-modal="true" style={{
      position: "fixed", inset: 0, background: "rgba(0,0,0,0.5)",
      display: "flex", alignItems: "center", justifyContent: "center", zIndex: 1000,
    }}>
      <div style={{
        background: "var(--bg-surface, #1e1e2e)", borderRadius: "12px",
        padding: "2rem", minWidth: "400px", color: "var(--text-primary, #cdd6f4)",
      }}>
        <h2 style={{ margin: "0 0 1.5rem" }}>Backup & Restore</h2>

        <div style={{ display: "flex", gap: "0.5rem", marginBottom: "1.5rem" }}>
          {(["backup", "restore"] as Tab[]).map(t => (
            <button
              key={t}
              data-testid={`${t}-tab`}
              onClick={() => { setTab(t); setError(null); setBkSuccess(null); setRsResult(null) }}
              style={{
                padding: "0.4rem 1rem",
                background: tab === t ? "var(--blue, #89b4fa)" : "var(--bg-overlay, #313244)",
                border: "none", borderRadius: "6px", cursor: "pointer",
                color: tab === t ? "#1e1e2e" : "inherit", fontWeight: tab === t ? 600 : 400,
              }}
            >
              {t === "backup" ? "Export Backup" : "Restore"}
            </button>
          ))}
        </div>

        {error && <p style={{ color: "var(--red, #f38ba8)", fontSize: "0.9rem", marginBottom: "1rem" }}>{error}</p>}

        {tab === "backup" && (
          <>
            <label style={{ display: "flex", alignItems: "center", gap: "0.5rem", marginBottom: "1rem" }}>
              <input
                type="checkbox"
                checked={includeFiles}
                onChange={e => setIncFiles(e.target.checked)}
              />
              Include book files (larger backup)
            </label>
            {backupSuccess && (
              <div data-testid="backup-success" style={{
                background: "var(--green-dim, #1e3a2a)", borderRadius: "6px",
                padding: "0.75rem", marginBottom: "1rem", fontSize: "0.9rem",
              }}>
                Backup saved to: <code style={{ wordBreak: "break-all" }}>{backupSuccess}</code>
              </div>
            )}
            <button
              data-testid="export-backup-btn"
              onClick={handleExport}
              disabled={loading}
              style={{
                width: "100%", padding: "0.6rem",
                background: "var(--blue, #89b4fa)", border: "none",
                borderRadius: "6px", cursor: "pointer", color: "#1e1e2e",
                fontWeight: 600, opacity: loading ? 0.6 : 1,
              }}
            >
              {loading ? "Exporting…" : "Choose Location & Export"}
            </button>
          </>
        )}

        {tab === "restore" && (
          <>
            {restoreResult && (
              <div data-testid="restore-success" style={{
                background: "var(--green-dim, #1e3a2a)", borderRadius: "6px",
                padding: "0.75rem", marginBottom: "1rem", fontSize: "0.9rem",
              }}>
                Restored <strong>{restoreResult.books_restored}</strong> book(s) successfully.
              </div>
            )}
            <p style={{ fontSize: "0.9rem", color: "var(--text-muted, #6c7086)", marginBottom: "1rem" }}>
              Select a .xcalibre backup file to restore. Existing books will not be duplicated.
            </p>
            <button
              data-testid="restore-backup-btn"
              onClick={handleRestore}
              disabled={loading}
              style={{
                width: "100%", padding: "0.6rem",
                background: "var(--blue, #89b4fa)", border: "none",
                borderRadius: "6px", cursor: "pointer", color: "#1e1e2e",
                fontWeight: 600, opacity: loading ? 0.6 : 1,
              }}
            >
              {loading ? "Restoring…" : "Choose Backup File & Restore"}
            </button>
          </>
        )}

        <div style={{ display: "flex", justifyContent: "flex-end", marginTop: "1.5rem" }}>
          <button
            onClick={onClose}
            style={{
              padding: "0.4rem 1rem", background: "var(--bg-overlay, #313244)",
              border: "none", borderRadius: "6px", cursor: "pointer", color: "inherit",
            }}
          >
            Close
          </button>
        </div>
      </div>
    </div>
  )
}
```

Then run:
```bash
cd ui && npm test -- BackupRestoreDialog && cd ..
git add ui/src/components/BackupRestoreDialog.tsx
git commit -m "R19b-T04: BackupRestoreDialog component — all UI tests green"
```

---

## R19b-T05 — Milestone Check + Visual Inspection

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cd ui && npm test && cd ..
```

**Visual inspection:**
```bash
pkill -x xcalibre 2>/dev/null || true
cargo tauri dev &>/tmp/xcalibre_tauri_dev.log &
# Poll until xcalibre process appears — first-run compilation can take 3-5 min
for i in $(seq 1 30); do
  sleep 10
  if pgrep -x xcalibre > /dev/null 2>&1; then
    echo "xcalibre running after $((i*10))s"
    sleep 3
    break
  fi
  echo "Waiting for xcalibre… $((i*10))s elapsed"
  [ "$i" -eq 30 ] && echo "ERROR: xcalibre did not launch within 5 minutes" && exit 1
done
```

1. Verify the UI:
2. Open Settings → Backup & Restore
3. Click "Export Backup" → save to Desktop as `test.xcalibre`
4. Open the file in Finder → Show Info → verify it's a ZIP file
5. Rename `.xcalibre` to `.zip`, extract → verify `library.db` and `manifest.json` are present
6. Delete a book from the library, then restore from backup
7. Verify the deleted book reappears after restore
8. Test cross-library copy: right-click a book → "Copy to Library…" → select target → verify it appears

```bash
pkill -x xcalibre 2>/dev/null || true
```

```bash
git add -A
git commit -m "R19b-T05: RMP-19 Backup & Restore — all tests green, UI wired"
```
