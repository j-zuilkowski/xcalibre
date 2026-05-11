# RMP-21a — Page Count & Reading Progress Sync (Red: Failing Tests)

> Prerequisite: rmp16b complete (book stats module), rmp01b complete.
> TDD role: RED — define page count storage and progress sync API via failing tests.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R21a-T01 | Migration 0019 — page_count column + reading_sessions table | ⬜ |
| R21a-T02 | Failing tests: page count population on ingest | ⬜ |
| R21a-T03 | Failing tests: reading session recording | ⬜ |
| R21a-T04 | Failing tests: ReadingProgressBar component | ⬜ |

---

## R21a-T01

Write `processing/src/db/migrations/0019_reading_sessions.sql`:
```sql
-- Store computed page counts on local_books (column may already exist as NULL).
-- Use ALTER TABLE with IF NOT EXISTS guard pattern.
ALTER TABLE local_books ADD COLUMN IF NOT EXISTS page_count INTEGER;

-- Reading sessions: track time-spent reading per book.
CREATE TABLE IF NOT EXISTS reading_sessions (
    id          TEXT NOT NULL PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    book_id     TEXT NOT NULL REFERENCES local_books(id) ON DELETE CASCADE,
    started_at  TEXT NOT NULL DEFAULT (datetime('now')),
    ended_at    TEXT,
    duration_s  INTEGER NOT NULL DEFAULT 0,
    progress_start REAL NOT NULL DEFAULT 0.0,
    progress_end   REAL NOT NULL DEFAULT 0.0
);
CREATE INDEX IF NOT EXISTS rs_book_id ON reading_sessions(book_id);
```

> **Note**: SQLite does not support `ADD COLUMN IF NOT EXISTS` before version 3.37.
> If the target SQLite version is older, use a try/catch in the migration runner
> or check `pragma table_info(local_books)` first. Alternatively, use:
> ```sql
> CREATE TABLE IF NOT EXISTS local_books_v2 AS SELECT *, NULL AS page_count FROM local_books;
> ```
> but the simplest approach is to add the column unconditionally (migration will fail
> on second run if the column already exists). Use `sqlx migrate run` which skips
> already-applied migrations by checksum.

```bash
cargo build --workspace
git add processing/src/db/migrations/0019_reading_sessions.sql
git commit -m "R21a-T01: migration 0019 — page_count + reading_sessions"
```

---

## R21a-T02

Write `processing/tests/test_page_count.rs`:
```rust
use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::db::page_count::{
    update_book_page_count, get_book_page_count,
};

async fn setup() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    sqlx::query(
        "INSERT INTO local_books (id, title, authors, format)
         VALUES ('b1', 'Dune', '[]', 'EPUB')"
    ).execute(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_update_and_get_page_count() {
    let pool = setup().await;
    update_book_page_count(&pool, "b1", 752).await.expect("update");
    let count = get_book_page_count(&pool, "b1").await.expect("get");
    assert_eq!(count, Some(752));
}

#[tokio::test]
async fn test_page_count_null_by_default() {
    let pool = setup().await;
    let count = get_book_page_count(&pool, "b1").await.expect("get");
    assert!(count.is_none(), "page count should be null before first computation");
}

#[tokio::test]
async fn test_update_page_count_overwrites() {
    let pool = setup().await;
    update_book_page_count(&pool, "b1", 100).await.unwrap();
    update_book_page_count(&pool, "b1", 200).await.unwrap();
    let count = get_book_page_count(&pool, "b1").await.unwrap();
    assert_eq!(count, Some(200));
}
```

```bash
cargo test --workspace 2>&1 | grep -E "^error" | head -5
```

RED confirmed.

```bash
git add processing/tests/test_page_count.rs
git commit -m "R21a-T02: failing tests for page count storage"
```

---

## R21a-T03

Write `processing/tests/test_reading_sessions.rs`:
```rust
use sqlx::sqlite::SqlitePoolOptions;
use xcalibre_processing::db::reading_sessions::{
    start_reading_session, end_reading_session, list_reading_sessions,
    total_reading_time_seconds, ReadingSession,
};

async fn setup() -> sqlx::Pool<sqlx::Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    sqlx::query(
        "INSERT INTO local_books (id, title, authors, format)
         VALUES ('b1', 'Dune', '[]', 'EPUB')"
    ).execute(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_start_and_end_session() {
    let pool = setup().await;
    let session_id = start_reading_session(&pool, "b1", 0.0).await.expect("start");
    end_reading_session(&pool, &session_id, 60, 0.0, 5.0).await.expect("end");

    let sessions = list_reading_sessions(&pool, "b1").await.expect("list");
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].duration_s, 60);
    assert_eq!(sessions[0].progress_end, 5.0);
}

#[tokio::test]
async fn test_total_reading_time() {
    let pool = setup().await;
    for _ in 0..3 {
        let id = start_reading_session(&pool, "b1", 0.0).await.unwrap();
        end_reading_session(&pool, &id, 120, 0.0, 1.0).await.unwrap();
    }
    let total = total_reading_time_seconds(&pool, "b1").await.expect("total");
    assert_eq!(total, 360, "3 sessions of 120s = 360s total");
}

#[tokio::test]
async fn test_sessions_scoped_to_book() {
    let pool = setup().await;
    sqlx::query(
        "INSERT INTO local_books (id, title, authors, format) VALUES ('b2', 'Foundation', '[]', 'EPUB')"
    ).execute(&pool).await.unwrap();

    let id1 = start_reading_session(&pool, "b1", 0.0).await.unwrap();
    end_reading_session(&pool, &id1, 60, 0.0, 1.0).await.unwrap();
    let id2 = start_reading_session(&pool, "b2", 0.0).await.unwrap();
    end_reading_session(&pool, &id2, 90, 0.0, 1.0).await.unwrap();

    let b1_sessions = list_reading_sessions(&pool, "b1").await.unwrap();
    let b2_sessions = list_reading_sessions(&pool, "b2").await.unwrap();
    assert_eq!(b1_sessions.len(), 1);
    assert_eq!(b2_sessions.len(), 1);
}
```

```bash
cargo test --workspace 2>&1 | grep -E "^error" | head -5
```

RED confirmed.

```bash
git add processing/tests/test_reading_sessions.rs
git commit -m "R21a-T03: failing tests for reading session recording"
```

---

## R21a-T04

Write `ui/src/components/ReadingProgressBar.test.tsx`:
```tsx
import { render, screen } from "@testing-library/react"
import { describe, it, expect } from "vitest"
import { ReadingProgressBar } from "./ReadingProgressBar"

describe("ReadingProgressBar", () => {
  it("renders with 0% progress", () => {
    render(<ReadingProgressBar percent={0} />)
    const bar = screen.getByTestId("reading-progress-bar")
    expect(bar).toBeInTheDocument()
    expect(bar).toHaveAttribute("aria-valuenow", "0")
  })

  it("renders with 50% progress", () => {
    render(<ReadingProgressBar percent={50} />)
    const fill = screen.getByTestId("reading-progress-fill")
    expect(fill).toHaveStyle({ width: "50%" })
  })

  it("renders with 100% progress", () => {
    render(<ReadingProgressBar percent={100} />)
    const fill = screen.getByTestId("reading-progress-fill")
    expect(fill).toHaveStyle({ width: "100%" })
  })

  it("shows percentage label", () => {
    render(<ReadingProgressBar percent={75} showLabel />)
    expect(screen.getByText("75%")).toBeInTheDocument()
  })

  it("clamps values above 100", () => {
    render(<ReadingProgressBar percent={150} />)
    const fill = screen.getByTestId("reading-progress-fill")
    expect(fill).toHaveStyle({ width: "100%" })
  })

  it("clamps values below 0", () => {
    render(<ReadingProgressBar percent={-10} />)
    const fill = screen.getByTestId("reading-progress-fill")
    expect(fill).toHaveStyle({ width: "0%" })
  })
})
```

```bash
cd ui && npm test 2>&1 | grep "Cannot find\|FAIL" | head -5 && cd ..
git add ui/src/components/ReadingProgressBar.test.tsx
git commit -m "R21a-T04: failing tests for ReadingProgressBar component"
```

---

### ✅ RED Checkpoint

```bash
cargo test --workspace 2>&1 | grep -c "^error"
cd ui && npm test 2>&1 | grep -c "FAIL" && cd ..
```

Both should show failures. Proceed to **rmp21b**.
