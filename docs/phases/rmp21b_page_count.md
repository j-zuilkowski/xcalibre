# RMP-21b — Page Count & Reading Progress Sync (Green: Implementation)

> Prerequisite: rmp21a complete.
> TDD role: GREEN — implement page count storage, reading sessions, and progress UI.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R21b-T01 | `db/page_count.rs` + `db/reading_sessions.rs` | ⬜ |
| R21b-T02 | Auto-compute page count on ingest | ⬜ |
| R21b-T03 | Tauri commands for sessions and progress | ⬜ |
| R21b-T04 | `ReadingProgressBar.tsx` component | ⬜ |
| R21b-T05 | Wire progress into BookCard + BookDetailPanel | ⬜ |
| R21b-T06 | Milestone check + visual inspection | ⬜ |

---

## R21b-T01

Write `processing/src/db/page_count.rs`:
```rust
use sqlx::sqlite::SqlitePool;

pub async fn update_book_page_count(
    pool: &SqlitePool,
    book_id: &str,
    page_count: u32,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE local_books SET page_count=? WHERE id=?")
        .bind(page_count as i64)
        .bind(book_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_book_page_count(
    pool: &SqlitePool,
    book_id: &str,
) -> Result<Option<u32>, sqlx::Error> {
    let row: Option<(Option<i64>,)> = sqlx::query_as(
        "SELECT page_count FROM local_books WHERE id=?"
    )
    .bind(book_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.and_then(|(v,)| v).map(|v| v as u32))
}
```

Write `processing/src/db/reading_sessions.rs`:
```rust
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReadingSession {
    pub id:             String,
    pub book_id:        String,
    pub started_at:     String,
    pub ended_at:       Option<String>,
    pub duration_s:     i64,
    pub progress_start: f64,
    pub progress_end:   f64,
}

pub async fn start_reading_session(
    pool: &SqlitePool,
    book_id: &str,
    progress_start: f64,
) -> Result<String, sqlx::Error> {
    let row: (String,) = sqlx::query_as(
        "INSERT INTO reading_sessions (book_id, progress_start)
         VALUES (?, ?)
         RETURNING id"
    )
    .bind(book_id)
    .bind(progress_start)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

pub async fn end_reading_session(
    pool: &SqlitePool,
    session_id: &str,
    duration_s: i64,
    progress_start: f64,
    progress_end: f64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE reading_sessions
         SET ended_at=datetime('now'), duration_s=?, progress_start=?, progress_end=?
         WHERE id=?"
    )
    .bind(duration_s)
    .bind(progress_start)
    .bind(progress_end)
    .bind(session_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_reading_sessions(
    pool: &SqlitePool,
    book_id: &str,
) -> Result<Vec<ReadingSession>, sqlx::Error> {
    sqlx::query_as::<_, ReadingSession>(
        "SELECT * FROM reading_sessions WHERE book_id=? ORDER BY started_at DESC"
    )
    .bind(book_id)
    .fetch_all(pool)
    .await
}

pub async fn total_reading_time_seconds(
    pool: &SqlitePool,
    book_id: &str,
) -> Result<i64, sqlx::Error> {
    let row: (Option<i64>,) = sqlx::query_as(
        "SELECT COALESCE(SUM(duration_s), 0) FROM reading_sessions WHERE book_id=?"
    )
    .bind(book_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0.unwrap_or(0))
}
```

Add to `processing/src/db/mod.rs`:
```rust
pub mod page_count;
pub mod reading_sessions;
```

Then run:
```bash
cargo test --workspace -- test_page_count test_reading_sessions
git add processing/src/db/page_count.rs processing/src/db/reading_sessions.rs \
        processing/src/db/mod.rs
git commit -m "R21b-T01: page_count + reading_sessions queries — all tests green"
```

---

## R21b-T02

In the ingest pipeline (wherever `word_count` is computed after text extraction),
add a page count computation step:

In `src-tauri/src/commands/ingest.rs` (or wherever books are imported), after computing stats:
```rust
use xcalibre_processing::stats::compute_book_stats;
use xcalibre_processing::db::page_count::update_book_page_count;

// After inserting the book, compute and store page count:
if let Ok(stats) = compute_book_stats(&file_path) {
    let _ = update_book_page_count(&state.pool, &book_id, stats.page_count_estimate).await;
}
```

```bash
cargo build --workspace
git add src-tauri/src/commands/ingest.rs
git commit -m "R21b-T02: auto-compute page count on book ingest"
```

---

## R21b-T03

Write `src-tauri/src/commands/progress.rs`:
```rust
use tauri::State;
use xcalibre_processing::db::reading_sessions::{
    start_reading_session, end_reading_session, list_reading_sessions,
    total_reading_time_seconds, ReadingSession,
};
use xcalibre_processing::db::page_count::{get_book_page_count, update_book_page_count};
use crate::AppState;

#[tauri::command]
pub async fn start_reading_session_cmd(
    book_id:        String,
    progress_start: f64,
    state: State<'_, AppState>,
) -> Result<String, String> {
    start_reading_session(&state.pool, &book_id, progress_start)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn end_reading_session_cmd(
    session_id:     String,
    duration_s:     i64,
    progress_start: f64,
    progress_end:   f64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    end_reading_session(&state.pool, &session_id, duration_s, progress_start, progress_end)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_reading_stats_cmd(
    book_id: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let sessions = list_reading_sessions(&state.pool, &book_id)
        .await.map_err(|e| e.to_string())?;
    let total_s = total_reading_time_seconds(&state.pool, &book_id)
        .await.map_err(|e| e.to_string())?;
    let page_count = get_book_page_count(&state.pool, &book_id)
        .await.map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "session_count":      sessions.len(),
        "total_seconds":      total_s,
        "total_minutes":      total_s / 60,
        "page_count":         page_count,
    }))
}

#[tauri::command]
pub async fn update_reading_progress_cmd(
    book_id:          String,
    progress_percent: f64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE local_books SET progress_percent=?, last_opened_at=datetime('now') WHERE id=?"
    )
    .bind(progress_percent)
    .bind(&book_id)
    .execute(&state.pool)
    .await.map_err(|e| e.to_string())?;
    Ok(())
}
```

Register all commands in `src-tauri/src/main.rs`:
```rust
commands::progress::start_reading_session_cmd,
commands::progress::end_reading_session_cmd,
commands::progress::get_reading_stats_cmd,
commands::progress::update_reading_progress_cmd,
```

```bash
cargo build --workspace
git add src-tauri/src/commands/progress.rs src-tauri/src/main.rs
git commit -m "R21b-T03: Tauri commands for reading sessions and progress"
```

---

## R21b-T04

Write `ui/src/components/ReadingProgressBar.tsx`:
```tsx
interface Props {
  percent:   number
  showLabel?: boolean
  color?:    string
  height?:   number
}

export function ReadingProgressBar({
  percent,
  showLabel = false,
  color  = "var(--blue, #89b4fa)",
  height = 6,
}: Props) {
  const clamped = Math.min(100, Math.max(0, percent))

  return (
    <div style={{ display: "flex", alignItems: "center", gap: "0.5rem" }}>
      <div
        data-testid="reading-progress-bar"
        role="progressbar"
        aria-valuenow={clamped}
        aria-valuemin={0}
        aria-valuemax={100}
        style={{
          flex: 1, height: `${height}px`,
          background: "var(--bg-overlay, #313244)",
          borderRadius: `${height / 2}px`, overflow: "hidden",
        }}
      >
        <div
          data-testid="reading-progress-fill"
          style={{
            width:        `${clamped}%`,
            height:       "100%",
            background:   clamped === 100 ? "var(--green, #a6e3a1)" : color,
            borderRadius: `${height / 2}px`,
            transition:   "width 0.3s ease",
          }}
        />
      </div>
      {showLabel && (
        <span style={{ fontSize: "0.8rem", color: "var(--text-muted, #6c7086)", minWidth: "3ch" }}>
          {Math.round(clamped)}%
        </span>
      )}
    </div>
  )
}
```

Then run:
```bash
cd ui && npm test -- ReadingProgressBar && cd ..
git add ui/src/components/ReadingProgressBar.tsx
git commit -m "R21b-T04: ReadingProgressBar component — all tests green"
```

---

## R21b-T05

In `ui/src/components/BookCard.tsx`, add the progress bar below the cover:
```tsx
import { ReadingProgressBar } from "./ReadingProgressBar"

// In BookCard JSX, below book title:
{book.progress_percent > 0 && (
  <ReadingProgressBar percent={book.progress_percent} />
)}
```

In `ui/src/components/BookDetailPanel.tsx`, add reading stats section:
```tsx
// Add reading time stats after cover display:
// <section>
//   <h3>Reading History</h3>
//   <p>Total reading time: {Math.round(readingStats.total_minutes)} minutes</p>
//   <p>Sessions: {readingStats.session_count}</p>
// </section>
```

```bash
cd ui && npm test && cd ..
git add ui/src/components/BookCard.tsx ui/src/components/BookDetailPanel.tsx
git commit -m "R21b-T05: progress bar in BookCard + reading stats in detail panel"
```

---

## R21b-T06 — Milestone Check + Visual Inspection

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cd ui && npm test && cd ..
```

**Visual inspection:**
1. Launch the app: `cd src-tauri && cargo tauri dev`
2. Import a new EPUB — check console or DB: verify `page_count` is populated
3. Open a book's detail panel — verify "Book Statistics" shows page count
4. Open a book in the reader — start reading, then close
5. Reopen the book detail panel — verify reading session appears under "Reading History"
6. Verify `ReadingProgressBar` shows correct fill color (green at 100%, blue otherwise)
7. Verify progress percentage is persisted between app restarts

```bash
git add -A
git commit -m "R21b-T06: RMP-21 Page Count & Progress — all tests green, UI wired"
```
