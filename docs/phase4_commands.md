# Phase 4 — Reading Features

> HOW TO USE: For every "Write `path`" line → call write_file with that path and content.
> For every "Then run:" block → call your shell tool for each command.
> DO NOT print code as output. Write it to disk using your tools.
> Prerequisite: Phase 3 complete and all tests green.
> Prerequisite: Phase 1 T29–T35 complete (Tauri + React scaffolding in place).
> Status: ✅ done

## Status

| Task | Title | Status |
|------|-------|--------|
| P4-T01 | EPUB renderer in Tauri WebView | ✅ |
| P4-T02 | Reader CSS (typography, dark mode) | ✅ |
| P4-T03 | CFI-based reading position | ✅ |
| P4-T04 | Persist reading position to DB | ✅ |
| P4-T05 | Progress percent calculation | ✅ |
| P4-T06 | Bookmarks — data model + DB table | ✅ |
| P4-T07 | Bookmarks — UI (add, list, jump to) | ✅ |
| P4-T08 | Font size + theme settings | ✅ |
| P4-T09 | Settings persistence (local JSON) | ✅ |
| P4-T10 | Reader tests | ✅ |

---

## P4-T01

Write `src-tauri/src/epub_protocol.rs` with this exact content:
```rust
use sqlx::SqlitePool;
use std::io::Read;
use std::sync::Arc;
use tauri::{AppHandle, Manager};

pub fn epub_handler(
    app: &AppHandle,
    req: tauri::http::Request<Vec<u8>>,
) -> tauri::http::Response<Vec<u8>> {
    match handle(app, req) {
        Ok(resp) => resp,
        Err(_)   => tauri::http::Response::builder()
            .status(404)
            .body(b"not found".to_vec())
            .unwrap_or_else(|_| tauri::http::Response::new(b"not found".to_vec())),
    }
}

fn handle(
    app: &AppHandle,
    req: tauri::http::Request<Vec<u8>>,
) -> Result<tauri::http::Response<Vec<u8>>, Box<dyn std::error::Error>> {
    let url = req.uri().to_string();
    let without_scheme = url
        .strip_prefix("epub://")
        .ok_or("bad scheme")?;
    let (job_id, href) = without_scheme
        .split_once('/')
        .ok_or("missing path")?;

    let pool = app.state::<Arc<SqlitePool>>();
    let rt = tokio::runtime::Handle::current();
    let file_path: Option<String> = rt.block_on(async {
        sqlx::query_as::<_, (String,)>("SELECT file_path FROM jobs WHERE id = ?")
            .bind(job_id)
            .fetch_optional(pool.inner().as_ref())
            .await
            .ok()
            .flatten()
            .map(|(p,)| p)
    });

    let file_path = file_path.ok_or("job not found")?;
    let file = std::fs::File::open(&file_path)?;
    let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file))?;
    let mut entry = archive.by_name(href)?;
    let mut bytes = Vec::new();
    entry.read_to_end(&mut bytes)?;

    let mime = mime_for(href);
    let resp = tauri::http::Response::builder()
        .status(200)
        .header("Content-Type", mime)
        .body(bytes)?;
    Ok(resp)
}

fn mime_for(href: &str) -> &'static str {
    if href.ends_with(".html") || href.ends_with(".xhtml") || href.ends_with(".htm") {
        "text/html"
    } else if href.ends_with(".css") {
        "text/css"
    } else if href.ends_with(".png") {
        "image/png"
    } else if href.ends_with(".jpg") || href.ends_with(".jpeg") {
        "image/jpeg"
    } else if href.ends_with(".gif") {
        "image/gif"
    } else if href.ends_with(".svg") {
        "image/svg+xml"
    } else {
        "application/octet-stream"
    }
}
```

In `src-tauri/src/main.rs`, register the epub protocol in the builder. Add `use std::sync::Arc;` and `use sqlx::SqlitePool;` to the use block, then inside `tauri::Builder::default()` chain, add:
```rust
        .register_uri_scheme_protocol("epub", |app, req| {
            crate::epub_protocol::epub_handler(app, req)
        })
```

In `src-tauri/src/commands.rs`, add this function:
```rust
#[tauri::command]
pub async fn get_spine(
    pool: tauri::State<'_, std::sync::Arc<sqlx::SqlitePool>>,
    job_id: String,
) -> Result<Vec<String>, String> {
    use std::io::Read;

    let row: Option<(String,)> =
        sqlx::query_as("SELECT file_path FROM jobs WHERE id = ?")
            .bind(&job_id)
            .fetch_optional(pool.inner().as_ref())
            .await
            .map_err(|e| e.to_string())?;

    let file_path = row.ok_or("job not found")?.0;
    let file = std::fs::File::open(&file_path).map_err(|e| e.to_string())?;
    let mut archive =
        zip::ZipArchive::new(std::io::BufReader::new(file)).map_err(|e| e.to_string())?;

    let container_xml = {
        let mut entry = archive.by_name("META-INF/container.xml").map_err(|e| e.to_string())?;
        let mut s = String::new();
        entry.read_to_string(&mut s).map_err(|e| e.to_string())?;
        s
    };
    let doc = roxmltree::Document::parse(&container_xml).map_err(|e| e.to_string())?;
    let opf_path = doc
        .descendants()
        .find(|n| n.tag_name().name() == "rootfile")
        .and_then(|n| n.attribute("full-path"))
        .ok_or("no rootfile")?
        .to_string();

    let opf_xml = {
        let mut entry = archive.by_name(&opf_path).map_err(|e| e.to_string())?;
        let mut s = String::new();
        entry.read_to_string(&mut s).map_err(|e| e.to_string())?;
        s
    };
    let doc = roxmltree::Document::parse(&opf_xml).map_err(|e| e.to_string())?;
    let hrefs: Vec<String> = doc
        .descendants()
        .filter(|n| n.tag_name().name() == "itemref")
        .filter_map(|n| n.attribute("idref"))
        .filter_map(|idref| {
            doc.descendants()
                .find(|n| n.tag_name().name() == "item" && n.attribute("id") == Some(idref))
                .and_then(|n| n.attribute("href"))
                .map(String::from)
        })
        .collect();

    Ok(hrefs)
}
```

In `src-tauri/Cargo.toml`, add `zip = "0.6"` and `roxmltree = "0.20"` under `[dependencies]` if not already present.

Write `ui/src/components/ReaderView.tsx` with this exact content:
```tsx
import { useState, useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"

interface Props {
  jobId: string
  onClose: () => void
}

export function ReaderView({ jobId, onClose }: Props) {
  const [spine, setSpine] = useState<string[]>([])
  const [index, setIndex] = useState(0)

  useEffect(() => {
    invoke<string[]>("get_spine", { jobId }).then(setSpine).catch(console.error)
  }, [jobId])

  const href = spine[index]

  return (
    <div className="fixed inset-0 bg-white dark:bg-gray-900 flex flex-col z-50">
      <div className="flex items-center justify-between px-4 py-2 border-b border-gray-200 dark:border-gray-700">
        <button
          onClick={() => setIndex((i) => Math.max(0, i - 1))}
          disabled={index === 0}
          className="px-3 py-1 text-sm bg-gray-100 hover:bg-gray-200 rounded disabled:opacity-40"
        >
          ← Prev
        </button>
        <span className="text-sm text-gray-500">
          {index + 1} / {spine.length}
        </span>
        <button
          onClick={() => setIndex((i) => Math.min(spine.length - 1, i + 1))}
          disabled={index >= spine.length - 1}
          className="px-3 py-1 text-sm bg-gray-100 hover:bg-gray-200 rounded disabled:opacity-40"
        >
          Next →
        </button>
        <button onClick={onClose} className="ml-4 px-3 py-1 text-sm bg-red-100 hover:bg-red-200 rounded">
          Close
        </button>
      </div>
      {href && (
        <iframe
          key={href}
          src={`epub://${jobId}/${href}`}
          className="flex-1 w-full border-0"
          title="reader"
        />
      )}
    </div>
  )
}
```

Then run:
```bash
cargo build --workspace
cd ui && npm run build && cd ..
git add src-tauri/src/epub_protocol.rs src-tauri/src/commands.rs ui/src/components/ReaderView.tsx
git commit -m "P4-T01: add EPUB renderer via custom epub:// protocol and ReaderView"
```

---

## P4-T02

Write `ui/src/reader.css` with this exact content:
```css
:root {
  --bg: #fefefe;
  --fg: #1a1a1a;
  --font: Georgia, serif;
  --size: 18px;
  --line: 1.7;
  --max-w: 680px;
}

@media (prefers-color-scheme: dark) {
  :root {
    --bg: #1c1c1e;
    --fg: #e0e0e0;
  }
}

body {
  background: var(--bg);
  color: var(--fg);
  font-family: var(--font);
  font-size: var(--size);
  line-height: var(--line);
  max-width: var(--max-w);
  margin: 2rem auto;
  padding: 0 1rem;
}

img {
  max-width: 100%;
  height: auto;
}

a {
  color: #3b82f6;
}

.theme-sepia {
  --bg: #f4ecd8;
  --fg: #433422;
}
```

Then run:
```bash
cd ui && npm run build && cd ..
git add ui/src/reader.css
git commit -m "P4-T02: add reader CSS with dark mode and sepia theme support"
```

---

## P4-T03

Write `ui/src/reader/cfi.ts` with this exact content:
```ts
export function getCurrentCfi(spineIndex: number): string {
  const elements = document.body.querySelectorAll("*")
  let elementIndex = 0
  for (let i = 0; i < elements.length; i++) {
    const rect = (elements[i] as HTMLElement).getBoundingClientRect()
    if (rect.top >= 0) {
      elementIndex = i
      break
    }
  }
  const scrollY = Math.round(window.scrollY)
  return `${spineIndex}:${elementIndex}:${scrollY}`
}

export function restoreCfi(cfi: string, _doc: Document): void {
  const parts = cfi.split(":")
  if (parts.length < 3) return
  const elementIndex = parseInt(parts[1], 10)
  const scrollY = parseInt(parts[2], 10)
  const elements = document.body.querySelectorAll("*")
  if (elementIndex < elements.length) {
    ;(elements[elementIndex] as HTMLElement).scrollIntoView()
  } else {
    window.scrollTo(0, scrollY)
  }
}
```

Then run:
```bash
cd ui && npm run build && cd ..
git add ui/src/reader/cfi.ts
git commit -m "P4-T03: add lightweight CFI position tracker"
```

---

## P4-T04

Write `processing/src/db/migrations/0003_reading_position.sql` with this exact content:
```sql
ALTER TABLE local_books ADD COLUMN reading_cfi TEXT;
```
Note: `last_opened_at` already exists in `0001_jobs.sql` — do NOT add it again.

In `processing/src/db/queries.rs`, add the following function at the end of the file. Do not remove anything already in the file.
```rust
pub async fn update_reading_position(
    pool: &SqlitePool,
    book_id: &str,
    position: &str,
) -> Result<(), ProcessingError> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "UPDATE local_books SET reading_cfi=?, last_opened_at=?, updated_at=? WHERE id=?",
    )
    .bind(position)
    .bind(&now)
    .bind(&now)
    .bind(book_id)
    .execute(pool)
    .await
    .map_err(ProcessingError::DbError)?;
    Ok(())
}
```

In `src-tauri/src/commands.rs`, add this function:
```rust
#[tauri::command]
pub async fn update_position(
    pool: tauri::State<'_, std::sync::Arc<sqlx::SqlitePool>>,
    book_id: String,
    position: String,
) -> Result<(), String> {
    xcalibre_processing::db::queries::update_reading_position(
        pool.inner().as_ref(),
        &book_id,
        &position,
    )
    .await
    .map_err(|e| e.to_string())
}
```

Then run:
```bash
cargo build --workspace
git add processing/src/db/migrations/0003_reading_position.sql processing/src/db/queries.rs src-tauri/src/commands.rs
git commit -m "P4-T04: persist reading position — migration + query + Tauri command"
```

---

## P4-T05

Add the following to the end of `ui/src/reader/cfi.ts`. Do not remove anything already in the file.
```ts
export function calcProgress(
  spineIndex: number,
  totalSpineItems: number,
  scrollRatio: number,
): number {
  const perItem = 100 / totalSpineItems
  return Math.min(100, spineIndex * perItem + scrollRatio * perItem)
}
```

In `processing/src/db/queries.rs`, add the following function at the end of the file. Do not remove anything already in the file.
```rust
pub async fn update_progress_percent(
    pool: &SqlitePool,
    book_id: &str,
    percent: f64,
) -> Result<(), ProcessingError> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "UPDATE local_books SET progress_percent=?, updated_at=? WHERE id=?",
    )
    .bind(percent)
    .bind(&now)
    .bind(book_id)
    .execute(pool)
    .await
    .map_err(ProcessingError::DbError)?;
    Ok(())
}
```

Then run:
```bash
cargo build --workspace
cd ui && npm run build && cd ..
git add ui/src/reader/cfi.ts processing/src/db/queries.rs
git commit -m "P4-T05: add progress percent calculation and persistence"
```

---

### ✅ Milestone check — after P4-T05
```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
```
Fix any issues before continuing.

---

## P4-T06

Write `processing/src/db/migrations/0004_bookmarks.sql` with this exact content:
```sql
CREATE TABLE bookmarks (
    id         TEXT PRIMARY KEY,
    book_id    TEXT NOT NULL REFERENCES local_books(id) ON DELETE CASCADE,
    cfi        TEXT NOT NULL,
    label      TEXT,
    created_at TEXT NOT NULL
);
CREATE INDEX idx_bookmarks_book ON bookmarks(book_id);
```

In `processing/src/db/queries.rs`, add the following at the end of the file. Do not remove anything already in the file.
```rust
#[derive(Debug, Clone)]
pub struct Bookmark {
    pub id:         String,
    pub book_id:    String,
    pub cfi:        String,
    pub label:      Option<String>,
    pub created_at: String,
}

pub async fn add_bookmark(
    pool: &SqlitePool,
    book_id: &str,
    cfi: &str,
    label: Option<&str>,
) -> Result<String, ProcessingError> {
    let id  = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO bookmarks (id, book_id, cfi, label, created_at) VALUES (?,?,?,?,?)",
    )
    .bind(&id)
    .bind(book_id)
    .bind(cfi)
    .bind(label)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(ProcessingError::DbError)?;
    Ok(id)
}

pub async fn list_bookmarks(
    pool: &SqlitePool,
    book_id: &str,
) -> Result<Vec<Bookmark>, ProcessingError> {
    let rows: Vec<(String, String, String, Option<String>, String)> = sqlx::query_as(
        "SELECT id, book_id, cfi, label, created_at FROM bookmarks WHERE book_id=? ORDER BY created_at ASC",
    )
    .bind(book_id)
    .fetch_all(pool)
    .await
    .map_err(ProcessingError::DbError)?;
    Ok(rows
        .into_iter()
        .map(|(id, book_id, cfi, label, created_at)| Bookmark {
            id,
            book_id,
            cfi,
            label,
            created_at,
        })
        .collect())
}

pub async fn delete_bookmark(pool: &SqlitePool, bookmark_id: &str) -> Result<(), ProcessingError> {
    sqlx::query("DELETE FROM bookmarks WHERE id=?")
        .bind(bookmark_id)
        .execute(pool)
        .await
        .map_err(ProcessingError::DbError)?;
    Ok(())
}
```

Then run:
```bash
cargo build --workspace
git add processing/src/db/migrations/0004_bookmarks.sql processing/src/db/queries.rs
git commit -m "P4-T06: add bookmarks table and CRUD queries"
```

---

## P4-T07

Write `ui/src/components/BookmarkPanel.tsx` with this exact content:
```tsx
import { useState, useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"

interface Bookmark {
  id: string
  book_id: string
  cfi: string
  label: string | null
  created_at: string
}

interface Props {
  bookId: string
  currentCfi: string
  onJump: (cfi: string) => void
  onClose: () => void
}

export function BookmarkPanel({ bookId, currentCfi, onJump, onClose }: Props) {
  const [bookmarks, setBookmarks] = useState<Bookmark[]>([])

  const load = () =>
    invoke<Bookmark[]>("list_bookmarks", { bookId }).then(setBookmarks).catch(console.error)

  useEffect(() => { load() }, [bookId])

  const add = async () => {
    await invoke("add_bookmark", { bookId, cfi: currentCfi, label: null })
    load()
  }

  const remove = async (id: string) => {
    await invoke("delete_bookmark", { bookmarkId: id })
    load()
  }

  return (
    <div className="w-72 h-full bg-white dark:bg-gray-900 border-l border-gray-200 dark:border-gray-700 flex flex-col p-4 overflow-y-auto">
      <div className="flex items-center justify-between mb-3">
        <h2 className="font-semibold text-gray-800 dark:text-gray-200">Bookmarks</h2>
        <button onClick={onClose} className="text-gray-500 hover:text-gray-700 text-lg">×</button>
      </div>
      <button
        onClick={add}
        className="mb-4 px-3 py-1.5 text-sm bg-blue-600 text-white rounded hover:bg-blue-700"
      >
        + Add bookmark here
      </button>
      {bookmarks.length === 0 && (
        <p className="text-sm text-gray-400">No bookmarks yet.</p>
      )}
      {bookmarks.map((bm) => (
        <div key={bm.id} className="flex items-center justify-between py-2 border-b border-gray-100 dark:border-gray-800">
          <button
            onClick={() => onJump(bm.cfi)}
            className="text-sm text-left text-blue-600 hover:underline truncate flex-1"
          >
            {bm.label ?? bm.cfi.slice(0, 20) + "…"}
          </button>
          <button
            onClick={() => remove(bm.id)}
            className="ml-2 text-red-400 hover:text-red-600 text-xs"
          >
            ✕
          </button>
        </div>
      ))}
    </div>
  )
}
```

In `src-tauri/src/commands.rs`, add these three functions:
```rust
#[tauri::command]
pub async fn add_bookmark(
    pool: tauri::State<'_, std::sync::Arc<sqlx::SqlitePool>>,
    book_id: String,
    cfi: String,
    label: Option<String>,
) -> Result<String, String> {
    xcalibre_processing::db::queries::add_bookmark(
        pool.inner().as_ref(),
        &book_id,
        &cfi,
        label.as_deref(),
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_bookmarks(
    pool: tauri::State<'_, std::sync::Arc<sqlx::SqlitePool>>,
    book_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    let bms = xcalibre_processing::db::queries::list_bookmarks(pool.inner().as_ref(), &book_id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(bms
        .into_iter()
        .map(|b| serde_json::json!({
            "id": b.id, "book_id": b.book_id,
            "cfi": b.cfi, "label": b.label, "created_at": b.created_at
        }))
        .collect())
}

#[tauri::command]
pub async fn delete_bookmark(
    pool: tauri::State<'_, std::sync::Arc<sqlx::SqlitePool>>,
    bookmark_id: String,
) -> Result<(), String> {
    xcalibre_processing::db::queries::delete_bookmark(pool.inner().as_ref(), &bookmark_id)
        .await
        .map_err(|e| e.to_string())
}
```

Then run:
```bash
cd ui && npm run build && cd ..
git add ui/src/components/BookmarkPanel.tsx src-tauri/src/commands.rs
git commit -m "P4-T07: add BookmarkPanel UI and Tauri commands"
```

---

## P4-T08

Write `ui/src/store/settingsStore.ts` with this exact content:
```ts
import { create } from "zustand"
import { persist } from "zustand/middleware"

interface Settings {
  fontSize:      number
  theme:         "light" | "dark" | "sepia"
  fontFamily:    "serif" | "sans" | "mono"
  setFontSize:   (n: number) => void
  setTheme:      (t: Settings["theme"]) => void
  setFontFamily: (f: Settings["fontFamily"]) => void
}

export const useSettingsStore = create<Settings>()(
  persist(
    (set) => ({
      fontSize:      18,
      theme:         "light",
      fontFamily:    "serif",
      setFontSize:   (fontSize)   => set({ fontSize }),
      setTheme:      (theme)      => set({ theme }),
      setFontFamily: (fontFamily) => set({ fontFamily }),
    }),
    { name: "xcalibre-settings" },
  ),
)
```

Write `ui/src/components/ReaderToolbar.tsx` with this exact content:
```tsx
import { useSettingsStore } from "../store/settingsStore"

interface Props {
  iframeRef: React.RefObject<HTMLIFrameElement>
}

export function ReaderToolbar({ iframeRef }: Props) {
  const { fontSize, theme, fontFamily, setFontSize, setTheme, setFontFamily } =
    useSettingsStore()

  const postCss = (css: Record<string, string>) => {
    iframeRef.current?.contentWindow?.postMessage({ type: "xcalibre-css", css }, "*")
  }

  return (
    <div className="flex items-center gap-4 px-4 py-2 border-b border-gray-200 dark:border-gray-700 text-sm">
      <label className="flex items-center gap-2">
        <span className="text-gray-600 dark:text-gray-400">Size</span>
        <input
          type="range"
          min={14}
          max={24}
          value={fontSize}
          onChange={(e) => {
            const n = Number(e.target.value)
            setFontSize(n)
            postCss({ "--size": `${n}px` })
          }}
          className="w-24"
        />
        <span className="w-6 text-center">{fontSize}</span>
      </label>

      <label className="flex items-center gap-2">
        <span className="text-gray-600 dark:text-gray-400">Theme</span>
        <select
          value={theme}
          onChange={(e) => {
            const t = e.target.value as Settings["theme"]
            setTheme(t)
            postCss({ "--theme": t })
          }}
          className="border border-gray-300 rounded px-1 py-0.5"
        >
          <option value="light">Light</option>
          <option value="dark">Dark</option>
          <option value="sepia">Sepia</option>
        </select>
      </label>

      <label className="flex items-center gap-2">
        <span className="text-gray-600 dark:text-gray-400">Font</span>
        <select
          value={fontFamily}
          onChange={(e) => {
            const f = e.target.value as Settings["fontFamily"]
            setFontFamily(f)
            postCss({ "--font": f === "serif" ? "Georgia,serif" : f === "sans" ? "system-ui,sans-serif" : "monospace" })
          }}
          className="border border-gray-300 rounded px-1 py-0.5"
        >
          <option value="serif">Serif</option>
          <option value="sans">Sans</option>
          <option value="mono">Mono</option>
        </select>
      </label>
    </div>
  )
}

type Settings = ReturnType<typeof useSettingsStore.getState>
```

Then run:
```bash
cd ui && npm run build && cd ..
git add ui/src/store/settingsStore.ts ui/src/components/ReaderToolbar.tsx
git commit -m "P4-T08: add reader settings store and toolbar (font, theme)"
```

---

## P4-T09

Settings persist automatically via Zustand's `persist` middleware to `localStorage` (Tauri WebView). No code changes needed.

Then run:
```bash
cargo build --workspace
cd ui && npm run build && cd ..
git add ui/src/store/settingsStore.ts
git commit -m "P4-T09: confirm settings persistence via Zustand persist middleware"
```

---

## P4-T10

Run `cd ui && npm install -D vitest && cd ..` to add vitest.

Write `ui/src/reader/cfi.test.ts` with this exact content:
```ts
import { describe, it, expect } from "vitest"
import { calcProgress } from "./cfi"

describe("calcProgress", () => {
  it("returns 0 for first item at scroll 0", () => {
    expect(calcProgress(0, 10, 0)).toBe(0)
  })
  it("returns 100 for last item at scroll 1", () => {
    expect(calcProgress(9, 10, 1)).toBe(100)
  })
  it("clamps at 100", () => {
    expect(calcProgress(10, 10, 1)).toBe(100)
  })
})
```

Write `processing/tests/test_bookmarks.rs` with this exact content:
```rust
use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::db::queries;

async fn setup_db() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    pool
}

async fn insert_book(pool: &sqlx::SqlitePool) -> String {
    let id  = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO local_books (id, title, authors_json, format, created_at, updated_at)
         VALUES (?,?,?,?,?,?)",
    )
    .bind(&id)
    .bind("Test Book")
    .bind("[]")
    .bind("EPUB")
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .unwrap();
    id
}

#[tokio::test]
async fn test_add_and_list_bookmarks() {
    let pool    = setup_db().await;
    let book_id = insert_book(&pool).await;
    queries::add_bookmark(&pool, &book_id, "0:0:0", None).await.unwrap();
    queries::add_bookmark(&pool, &book_id, "1:5:200", Some("Chapter 2")).await.unwrap();
    let bms = queries::list_bookmarks(&pool, &book_id).await.unwrap();
    assert_eq!(bms.len(), 2);
}

#[tokio::test]
async fn test_delete_bookmark() {
    let pool    = setup_db().await;
    let book_id = insert_book(&pool).await;
    let bm_id = queries::add_bookmark(&pool, &book_id, "0:0:0", None).await.unwrap();
    queries::delete_bookmark(&pool, &bm_id).await.unwrap();
    let bms = queries::list_bookmarks(&pool, &book_id).await.unwrap();
    assert!(bms.is_empty());
}
```

Then run:
```bash
cargo test --workspace
cd ui && npx vitest run && cd ..
cargo clippy --workspace -- -D warnings
git add ui/src/reader/cfi.test.ts processing/tests/test_bookmarks.rs ui/package.json
git commit -m "P4-T10: add reader and bookmark tests"
```

### ✅ Milestone check — after P4-T10
```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

Run `/review` before closing Phase 4.
