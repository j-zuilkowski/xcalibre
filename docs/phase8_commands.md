# Phase 8 — Advanced Reading

> HOW TO USE: For every "Write `path`" line → call write_file with that path and content.
> For every "Then run:" block → call your shell tool for each command.
> DO NOT print code as output. Write it to disk using your tools.
> Prerequisite: Phase 7 complete and all tests green.
> Status: ✅ done

Covers annotations and highlights in the EPUB reader, a comic (CBZ/CBR) page viewer,
OS-level open for PDF and other non-rendered formats, and annotation sync to xcalibre-server.

## Status

| Task | Title | Status |
|------|-------|--------|
| P8-T01 | Migration 0009 — annotations | ✅ |
| P8-T02 | Annotations DB queries | ✅ |
| P8-T03 | Annotations Tauri commands | ✅ |
| P8-T04 | EPUB reader highlight support | ✅ |
| P8-T05 | AnnotationsSidebar UI | ✅ |
| P8-T06 | PDF + other formats — OS open | ✅ |
| P8-T07 | Comic viewer (CBZ/CBR) | ✅ |
| P8-T08 | Annotation sync to xcalibre-server | ✅ |
| P8-T09 | Tests | ✅ |

---

## P8-T01

Write `processing/src/db/migrations/0009_annotations.sql` with this exact content:
```sql
-- Reader annotations: highlights, notes, bookmarks
CREATE TABLE IF NOT EXISTS annotations (
    id          TEXT    NOT NULL PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    book_id     TEXT    NOT NULL REFERENCES local_books(id) ON DELETE CASCADE,
    type        TEXT    NOT NULL CHECK(type IN ('highlight','note','bookmark')),
    cfi         TEXT    NOT NULL,
    selected_text TEXT,
    note        TEXT,
    color       TEXT    NOT NULL DEFAULT 'yellow',
    synced      INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT    NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS annotations_book_id ON annotations(book_id);
```

Then run:
```bash
cd processing && cargo build && cd ..
git add processing/src/db/migrations/0009_annotations.sql
git commit -m "P8-T01: migration 0008 — annotations table"
```

---

## P8-T02

Write `processing/src/db/annotation_queries.rs` with this exact content:
```rust
use crate::error::ProcessingError;
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Annotation {
    pub id:            String,
    pub book_id:       String,
    pub annotation_type: String,
    pub cfi:           String,
    pub selected_text: Option<String>,
    pub note:          Option<String>,
    pub color:         String,
    pub synced:        bool,
    pub created_at:    String,
}

pub async fn create_annotation(
    pool: &SqlitePool,
    book_id: &str,
    annotation_type: &str,
    cfi: &str,
    selected_text: Option<&str>,
    note: Option<&str>,
    color: &str,
) -> Result<String, ProcessingError> {
    let id  = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO annotations
         (id, book_id, type, cfi, selected_text, note, color, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(book_id)
    .bind(annotation_type)
    .bind(cfi)
    .bind(selected_text)
    .bind(note)
    .bind(color)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(ProcessingError::DbError)?;
    Ok(id)
}

pub async fn get_annotations(
    pool: &SqlitePool,
    book_id: &str,
) -> Result<Vec<Annotation>, ProcessingError> {
    let rows = sqlx::query_as::<_, (String, String, String, String, Option<String>, Option<String>, String, i64, String)>(
        "SELECT id, book_id, type, cfi, selected_text, note, color, synced, created_at
         FROM annotations WHERE book_id = ? ORDER BY created_at ASC",
    )
    .bind(book_id)
    .fetch_all(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    Ok(rows.into_iter().map(|(id, book_id, annotation_type, cfi, selected_text, note, color, synced, created_at)| {
        Annotation { id, book_id, annotation_type, cfi, selected_text, note, color, synced: synced != 0, created_at }
    }).collect())
}

pub async fn delete_annotation(pool: &SqlitePool, id: &str) -> Result<(), ProcessingError> {
    sqlx::query("DELETE FROM annotations WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(ProcessingError::DbError)?;
    Ok(())
}

pub async fn update_annotation_note(
    pool: &SqlitePool,
    id: &str,
    note: &str,
) -> Result<(), ProcessingError> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("UPDATE annotations SET note = ?, updated_at = ? WHERE id = ?")
        .bind(note)
        .bind(&now)
        .bind(id)
        .execute(pool)
        .await
        .map_err(ProcessingError::DbError)?;
    Ok(())
}

pub async fn get_unsynced_annotations(
    pool: &SqlitePool,
) -> Result<Vec<Annotation>, ProcessingError> {
    let rows = sqlx::query_as::<_, (String, String, String, String, Option<String>, Option<String>, String, i64, String)>(
        "SELECT id, book_id, type, cfi, selected_text, note, color, synced, created_at
         FROM annotations WHERE synced = 0 ORDER BY created_at ASC",
    )
    .fetch_all(pool)
    .await
    .map_err(ProcessingError::DbError)?;

    Ok(rows.into_iter().map(|(id, book_id, annotation_type, cfi, selected_text, note, color, synced, created_at)| {
        Annotation { id, book_id, annotation_type, cfi, selected_text, note, color, synced: synced != 0, created_at }
    }).collect())
}

pub async fn mark_annotation_synced(pool: &SqlitePool, id: &str) -> Result<(), ProcessingError> {
    sqlx::query("UPDATE annotations SET synced = 1 WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(ProcessingError::DbError)?;
    Ok(())
}
```

In `processing/src/db/mod.rs`, add `pub mod annotation_queries;` after the existing lines.

Then run:
```bash
cargo build --workspace
git add processing/src/db/annotation_queries.rs processing/src/db/mod.rs
git commit -m "P8-T02: annotation DB queries — CRUD + sync tracking"
```

---

## P8-T03

In `src-tauri/src/commands.rs`, add these commands at the end:
```rust
#[tauri::command]
pub async fn create_annotation(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    book_id: String,
    annotation_type: String,
    cfi: String,
    selected_text: Option<String>,
    note: Option<String>,
    color: String,
) -> Result<String, String> {
    xcalibre_processing::db::annotation_queries::create_annotation(
        pool.inner().as_ref(),
        &book_id, &annotation_type, &cfi,
        selected_text.as_deref(), note.as_deref(), &color,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_annotations(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    book_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    let annotations = xcalibre_processing::db::annotation_queries::get_annotations(
        pool.inner().as_ref(), &book_id,
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(annotations.into_iter().map(|a| serde_json::json!({
        "id": a.id, "book_id": a.book_id, "type": a.annotation_type,
        "cfi": a.cfi, "selected_text": a.selected_text, "note": a.note,
        "color": a.color, "synced": a.synced, "created_at": a.created_at,
    })).collect())
}

#[tauri::command]
pub async fn delete_annotation(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    id: String,
) -> Result<(), String> {
    xcalibre_processing::db::annotation_queries::delete_annotation(pool.inner().as_ref(), &id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_annotation_note(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    id: String,
    note: String,
) -> Result<(), String> {
    xcalibre_processing::db::annotation_queries::update_annotation_note(
        pool.inner().as_ref(), &id, &note,
    )
    .await
    .map_err(|e| e.to_string())
}
```

Add all four annotation commands to `invoke_handler!` in `src-tauri/src/main.rs`.

Then run:
```bash
cargo build --workspace
git add src-tauri/src/commands.rs src-tauri/src/main.rs
git commit -m "P8-T03: annotation Tauri commands — create, get, delete, update note"
```

---

## P8-T04

Write `ui/src/reader/highlights.ts` with this exact content:
```ts
import { invoke } from "@tauri-apps/api/core"

export type HighlightColor = "yellow" | "green" | "blue" | "pink" | "purple"

export interface Annotation {
  id: string
  book_id: string
  type: "highlight" | "note" | "bookmark"
  cfi: string
  selected_text: string | null
  note: string | null
  color: HighlightColor
  synced: boolean
  created_at: string
}

/** Apply saved annotations to the reader iframe document. */
export function applyHighlights(
  iframeDoc: Document,
  annotations: Annotation[],
): void {
  for (const ann of annotations) {
    if (ann.type !== "highlight" || !ann.selected_text) continue
    try {
      // Use the browser's find API to locate and wrap the selected text.
      // This is a best-effort approach; CFI-based restoration is preferred
      // but requires a full CFI library integration.
      const walker = iframeDoc.createTreeWalker(
        iframeDoc.body,
        NodeFilter.SHOW_TEXT,
      )
      let node: Node | null
      while ((node = walker.nextNode())) {
        const text = node.textContent ?? ""
        const idx  = text.indexOf(ann.selected_text)
        if (idx !== -1 && node.parentElement) {
          const range = iframeDoc.createRange()
          range.setStart(node, idx)
          range.setEnd(node, idx + ann.selected_text.length)
          const mark  = iframeDoc.createElement("mark")
          mark.dataset.annotationId = ann.id
          mark.style.backgroundColor = colorToCSS(ann.color)
          mark.style.cursor = "pointer"
          range.surroundContents(mark)
          break
        }
      }
    } catch {
      // Skip annotations that cannot be applied (e.g., across element boundaries)
    }
  }
}

/** Listen for text selection in the reader iframe and prompt to highlight. */
export function setupHighlightListener(
  iframeDoc: Document,
  bookId: string,
  currentCfi: string,
  onAnnotationCreated: () => void,
): () => void {
  const handler = async () => {
    const selection = iframeDoc.getSelection()
    if (!selection || selection.isCollapsed) return
    const text = selection.toString().trim()
    if (text.length < 3) return

    const color: HighlightColor = "yellow"
    await invoke("create_annotation", {
      bookId,
      annotationType: "highlight",
      cfi: currentCfi,
      selectedText: text,
      note: null,
      color,
    }).catch(console.error)

    selection.removeAllRanges()
    onAnnotationCreated()
  }

  iframeDoc.addEventListener("mouseup", handler)
  return () => iframeDoc.removeEventListener("mouseup", handler)
}

function colorToCSS(color: HighlightColor): string {
  const map: Record<HighlightColor, string> = {
    yellow: "rgba(255,255,0,0.4)",
    green:  "rgba(0,255,0,0.3)",
    blue:   "rgba(0,180,255,0.3)",
    pink:   "rgba(255,100,180,0.3)",
    purple: "rgba(180,0,255,0.3)",
  }
  return map[color] ?? map.yellow
}
```

Then run:
```bash
cd ui && npm run build && cd ..
git add ui/src/reader/highlights.ts
git commit -m "P8-T04: EPUB reader highlight support — apply saved highlights, selection listener"
```

---

## P8-T05

Write `ui/src/components/AnnotationsSidebar.tsx` with this exact content:
```tsx
import { useEffect, useState } from "react"
import { invoke } from "@tauri-apps/api/core"
import { Annotation } from "../reader/highlights"

interface Props {
  bookId: string
  onJumpTo: (cfi: string) => void
}

const COLOR_LABELS: Record<string, string> = {
  yellow: "🟡", green: "🟢", blue: "🔵", pink: "🩷", purple: "🟣",
}

export function AnnotationsSidebar({ bookId, onJumpTo }: Props) {
  const [annotations, setAnnotations] = useState<Annotation[]>([])
  const [editingId,   setEditingId]   = useState<string | null>(null)
  const [noteText,    setNoteText]    = useState("")

  const load = () =>
    invoke<Annotation[]>("get_annotations", { bookId })
      .then(setAnnotations)
      .catch(() => {})

  useEffect(() => { load() }, [bookId])

  const remove = async (id: string) => {
    await invoke("delete_annotation", { id }).catch(console.error)
    load()
  }

  const saveNote = async (id: string) => {
    await invoke("update_annotation_note", { id, note: noteText }).catch(console.error)
    setEditingId(null)
    load()
  }

  if (annotations.length === 0) {
    return (
      <div className="p-4 text-sm text-gray-400 dark:text-gray-500">
        No annotations yet. Select text in the reader to highlight.
      </div>
    )
  }

  return (
    <div className="flex flex-col gap-2 p-3 overflow-y-auto">
      <p className="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">
        Annotations ({annotations.length})
      </p>
      {annotations.map((ann) => (
        <div
          key={ann.id}
          className="rounded-lg border border-gray-200 dark:border-gray-700 p-2 bg-white dark:bg-gray-800 text-sm"
        >
          <div className="flex items-start justify-between gap-1">
            <button
              onClick={() => onJumpTo(ann.cfi)}
              className="flex-1 text-left text-gray-700 dark:text-gray-300 hover:text-blue-500 truncate"
              title="Jump to location"
            >
              {COLOR_LABELS[ann.color] ?? "📌"}{" "}
              {ann.selected_text ?? ann.type}
            </button>
            <button
              onClick={() => remove(ann.id)}
              className="text-gray-400 hover:text-red-500 shrink-0 text-xs"
            >
              ✕
            </button>
          </div>

          {ann.note && editingId !== ann.id && (
            <p
              className="mt-1 text-xs text-gray-500 dark:text-gray-400 cursor-pointer"
              onClick={() => { setEditingId(ann.id); setNoteText(ann.note ?? "") }}
            >
              {ann.note}
            </p>
          )}

          {editingId === ann.id ? (
            <div className="mt-1 flex gap-1">
              <input
                value={noteText}
                onChange={(e) => setNoteText(e.target.value)}
                className="flex-1 text-xs border rounded px-1 py-0.5 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
              />
              <button onClick={() => saveNote(ann.id)} className="text-xs px-1.5 bg-blue-500 text-white rounded">✓</button>
              <button onClick={() => setEditingId(null)} className="text-xs px-1.5 bg-gray-200 dark:bg-gray-600 rounded dark:text-white">✕</button>
            </div>
          ) : (
            !ann.note && (
              <button
                onClick={() => { setEditingId(ann.id); setNoteText("") }}
                className="mt-1 text-xs text-blue-400 hover:text-blue-500"
              >
                + Add note
              </button>
            )
          )}
        </div>
      ))}
    </div>
  )
}
```

Then run:
```bash
cd ui && npm run build && cd ..
git add ui/src/components/AnnotationsSidebar.tsx
git commit -m "P8-T05: AnnotationsSidebar UI — view, jump to, delete, edit note"
```

---

## P8-T06

Write `ui/src/components/FormatOpener.tsx` with this exact content:
```tsx
import { invoke } from "@tauri-apps/api/core"

interface Props {
  bookId: string
  format: string
  filePath: string
}

const RENDERABLE_IN_APP = new Set(["EPUB", "CBZ", "CBR"])

/** For formats the in-app reader cannot render, open in the OS default app. */
export function FormatOpener({ bookId, format, filePath }: Props) {
  if (RENDERABLE_IN_APP.has(format.toUpperCase())) return null

  const open = () =>
    invoke("open_in_os", { path: filePath }).catch(console.error)

  return (
    <button
      onClick={open}
      className="flex items-center gap-2 px-4 py-2 rounded-lg bg-blue-500 text-white hover:bg-blue-600 text-sm font-medium"
    >
      Open {format.toUpperCase()} in {getAppHint(format)}
    </button>
  )
}

function getAppHint(format: string): string {
  const f = format.toUpperCase()
  if (f === "PDF")   return "PDF viewer"
  if (f === "MOBI" || f === "AZW3" || f === "AZW4") return "Kindle / Books"
  return "default app"
}
```

In `src-tauri/src/commands.rs`, add this command at the end:
```rust
#[tauri::command]
pub async fn open_in_os(path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    std::process::Command::new("open")
        .arg(&path)
        .spawn()
        .map_err(|e| e.to_string())?;

    #[cfg(target_os = "windows")]
    std::process::Command::new("explorer")
        .arg(&path)
        .spawn()
        .map_err(|e| e.to_string())?;

    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open")
        .arg(&path)
        .spawn()
        .map_err(|e| e.to_string())?;

    Ok(())
}
```

Add `commands::open_in_os` to the `invoke_handler!`.

Then run:
```bash
cargo build --workspace
cd ui && npm run build && cd ..
git add ui/src/components/FormatOpener.tsx src-tauri/src/commands.rs src-tauri/src/main.rs
git commit -m "P8-T06: OS-level open for PDF, MOBI, AZW3 and other non-rendered formats"
```

---

## P8-T07

Write `ui/src/components/ComicViewer.tsx` with this exact content:
```tsx
import { useEffect, useState } from "react"
import { invoke } from "@tauri-apps/api/core"

interface Props {
  bookId: string
  onClose: () => void
}

export function ComicViewer({ bookId, onClose }: Props) {
  const [pages,       setPages]       = useState<string[]>([])
  const [currentPage, setCurrentPage] = useState(0)
  const [loading,     setLoading]     = useState(true)

  useEffect(() => {
    invoke<string[]>("list_comic_pages", { bookId })
      .then((p) => { setPages(p); setLoading(false) })
      .catch(() => setLoading(false))
  }, [bookId])

  const prev = () => setCurrentPage((p) => Math.max(0, p - 1))
  const next = () => setCurrentPage((p) => Math.min(pages.length - 1, p + 1))

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "ArrowRight" || e.key === "ArrowDown") next()
      if (e.key === "ArrowLeft"  || e.key === "ArrowUp")   prev()
      if (e.key === "Escape") onClose()
    }
    window.addEventListener("keydown", handler)
    return () => window.removeEventListener("keydown", handler)
  }, [pages.length])

  if (loading) {
    return (
      <div className="fixed inset-0 bg-black flex items-center justify-center z-50">
        <p className="text-white">Loading pages…</p>
      </div>
    )
  }

  if (pages.length === 0) {
    return (
      <div className="fixed inset-0 bg-black flex flex-col items-center justify-center z-50 gap-4">
        <p className="text-white">No pages found in this archive.</p>
        <button onClick={onClose} className="text-sm px-4 py-2 bg-white text-black rounded">Close</button>
      </div>
    )
  }

  return (
    <div className="fixed inset-0 bg-black flex flex-col z-50">
      {/* Toolbar */}
      <div className="flex items-center justify-between px-4 py-2 bg-black/80">
        <button onClick={onClose} className="text-white text-sm hover:text-gray-300">✕ Close</button>
        <span className="text-white text-sm">
          Page {currentPage + 1} / {pages.length}
        </span>
        <div className="flex gap-2">
          <button onClick={prev} disabled={currentPage === 0}
            className="text-white text-sm px-2 disabled:opacity-30">◀</button>
          <button onClick={next} disabled={currentPage === pages.length - 1}
            className="text-white text-sm px-2 disabled:opacity-30">▶</button>
        </div>
      </div>

      {/* Page image */}
      <div className="flex-1 flex items-center justify-center overflow-hidden">
        <img
          src={`asset://${pages[currentPage]}`}
          alt={`Page ${currentPage + 1}`}
          className="max-h-full max-w-full object-contain"
        />
      </div>
    </div>
  )
}
```

In `src-tauri/src/commands.rs`, add this command at the end:
```rust
#[tauri::command]
pub async fn list_comic_pages(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    book_id: String,
) -> Result<Vec<String>, String> {
    // Fetch the file path for this book
    let row = sqlx::query_as::<_, (String,)>(
        "SELECT file_path FROM local_books WHERE id = ?",
    )
    .bind(&book_id)
    .fetch_optional(pool.inner().as_ref())
    .await
    .map_err(|e| e.to_string())?
    .ok_or("book not found")?;

    let file_path = row.0;
    let path = std::path::Path::new(&file_path);

    let ext = path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if ext != "cbz" {
        return Err("Only CBZ is supported for in-app viewing; use open_in_os for CBR".to_string());
    }

    let file    = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let archive = zip::ZipArchive::new(std::io::BufReader::new(file))
        .map_err(|e| e.to_string())?;

    let temp_dir = std::env::temp_dir().join(format!("xcalibre_comic_{}", book_id));
    std::fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;

    let mut pages: Vec<String> = vec![];
    let mut archive = archive;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().to_string();
        let lower = name.to_lowercase();
        if lower.ends_with(".jpg") || lower.ends_with(".jpeg")
            || lower.ends_with(".png") || lower.ends_with(".webp")
        {
            let out_path = temp_dir.join(
                std::path::Path::new(&name)
                    .file_name()
                    .unwrap_or(std::ffi::OsStr::new(&name)),
            );
            let mut buf = vec![];
            std::io::Read::read_to_end(&mut entry, &mut buf).map_err(|e| e.to_string())?;
            std::fs::write(&out_path, &buf).map_err(|e| e.to_string())?;
            pages.push(out_path.to_string_lossy().into_owned());
        }
    }
    pages.sort();
    Ok(pages)
}
```

Add `commands::list_comic_pages` to the `invoke_handler!`.

Then run:
```bash
cargo build --workspace
cd ui && npm run build && cd ..
git add ui/src/components/ComicViewer.tsx src-tauri/src/commands.rs src-tauri/src/main.rs
git commit -m "P8-T07: ComicViewer UI + list_comic_pages command for CBZ page extraction"
```

---

## P8-T08

In `processing/src/pipeline/sync.rs` (or create it if it doesn't exist), add annotation sync logic.

Write `processing/src/pipeline/sync.rs` with this exact content:
```rust
use crate::db::annotation_queries;
use crate::error::ProcessingError;
use sqlx::SqlitePool;
use tracing::{info, warn};
use xcalibre_api::client::ApiClient;

/// Push unsynced annotations to xcalibre-server.
/// Marks each annotation as synced on success.
/// Does nothing if no API client is configured.
pub async fn sync_annotations(
    pool: &SqlitePool,
    client: Option<&ApiClient>,
) -> Result<usize, ProcessingError> {
    let client = match client {
        Some(c) => c,
        None    => {
            info!("no API client — skipping annotation sync");
            return Ok(0);
        }
    };

    let pending = annotation_queries::get_unsynced_annotations(pool).await?;
    let mut synced_count = 0;

    for ann in &pending {
        let payload = serde_json::json!({
            "id": ann.id, "book_id": ann.book_id, "type": ann.annotation_type,
            "cfi": ann.cfi, "selected_text": ann.selected_text, "note": ann.note,
            "color": ann.color, "created_at": ann.created_at,
        });

        let resp = client
            .http()
            .post(format!("{}/annotations", client.base_url()))
            .bearer_auth(client.token())
            .json(&payload)
            .send()
            .await;

        match resp {
            Ok(r) if r.status().is_success() => {
                annotation_queries::mark_annotation_synced(pool, &ann.id).await?;
                synced_count += 1;
                info!(annotation_id = %ann.id, "annotation synced");
            }
            Ok(r) => {
                warn!(annotation_id = %ann.id, status = %r.status(), "annotation sync failed");
            }
            Err(e) => {
                warn!(annotation_id = %ann.id, error = %e, "annotation sync error");
            }
        }
    }

    Ok(synced_count)
}
```

In `processing/src/pipeline/mod.rs`, add `pub mod sync;` if not already present.

In `src-tauri/src/commands.rs`, add this command at the end:
```rust
#[tauri::command]
pub async fn sync_annotations(
    pool: tauri::State<'_, Arc<SqlitePool>>,
) -> Result<usize, String> {
    // Try to build API client from keyring; silently skip if not configured
    let client = xcalibre_api::client::ApiClient::from_keyring("https://api.xcalibre.app").ok();
    xcalibre_processing::pipeline::sync::sync_annotations(
        pool.inner().as_ref(),
        client.as_ref(),
    )
    .await
    .map_err(|e| e.to_string())
}
```

Add `commands::sync_annotations` to the `invoke_handler!`.

Then run:
```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
git add processing/src/pipeline/sync.rs processing/src/pipeline/mod.rs \
        src-tauri/src/commands.rs src-tauri/src/main.rs
git commit -m "P8-T08: annotation sync to xcalibre-server — push unsynced, mark synced on success"
```

---

## P8-T09

Write `processing/tests/test_annotations.rs` with this exact content:
```rust
use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::db::annotation_queries;
use xcalibre_processing::pipeline::ingest::run_ingest;
use std::path::PathBuf;

async fn setup_db() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_create_and_fetch_annotation() {
    let pool   = setup_db().await;
    let path   = PathBuf::from("tests/fixtures/fixture_epub.epub");
    let ingest = run_ingest(&pool, &path).await.unwrap();

    let id = annotation_queries::create_annotation(
        &pool, &ingest.job_id, "highlight",
        "epubcfi(/6/4!/4/2/1:0)",
        Some("Hello world"), None, "yellow",
    )
    .await
    .unwrap();

    let annotations = annotation_queries::get_annotations(&pool, &ingest.job_id)
        .await
        .unwrap();
    assert_eq!(annotations.len(), 1);
    assert_eq!(annotations[0].id, id);
    assert_eq!(annotations[0].annotation_type, "highlight");
    assert_eq!(annotations[0].selected_text.as_deref(), Some("Hello world"));
    assert!(!annotations[0].synced);
}

#[tokio::test]
async fn test_delete_annotation() {
    let pool   = setup_db().await;
    let path   = PathBuf::from("tests/fixtures/fixture_epub.epub");
    let ingest = run_ingest(&pool, &path).await.unwrap();

    let id = annotation_queries::create_annotation(
        &pool, &ingest.job_id, "bookmark",
        "epubcfi(/6/4!/4/2/1:0)",
        None, None, "blue",
    )
    .await
    .unwrap();

    annotation_queries::delete_annotation(&pool, &id).await.unwrap();
    let annotations = annotation_queries::get_annotations(&pool, &ingest.job_id).await.unwrap();
    assert!(annotations.is_empty());
}

#[tokio::test]
async fn test_unsynced_annotations() {
    let pool   = setup_db().await;
    let path   = PathBuf::from("tests/fixtures/fixture_epub.epub");
    let ingest = run_ingest(&pool, &path).await.unwrap();

    let id = annotation_queries::create_annotation(
        &pool, &ingest.job_id, "note",
        "epubcfi(/6/4!/4/2/1:0)",
        None, Some("My note"), "green",
    )
    .await
    .unwrap();

    let unsynced = annotation_queries::get_unsynced_annotations(&pool).await.unwrap();
    assert_eq!(unsynced.len(), 1);

    annotation_queries::mark_annotation_synced(&pool, &id).await.unwrap();
    let unsynced = annotation_queries::get_unsynced_annotations(&pool).await.unwrap();
    assert!(unsynced.is_empty());
}
```

Then run:
```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
git add processing/tests/test_annotations.rs
git commit -m "P8-T09: annotation tests — create, fetch, delete, sync tracking"
```

---

### ✅ Milestone check — Phase 8 complete
```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cd ui && npm run build && cd ..
```
