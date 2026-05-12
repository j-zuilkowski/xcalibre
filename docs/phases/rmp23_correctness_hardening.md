# RMP-23 — Correctness & Performance Hardening

> Prerequisite: rmp22b complete (xcalibre roadmap finished).
> TDD role: GREEN — fix 9 correctness/performance issues surfaced by code review.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R23-T01 | `sanitize_sort_field` — return `Result` instead of silent fallback | ⬜ |
| R23-T02 | `sessions.lock().unwrap()` — propagate poisoned-lock errors (7 sites) | ⬜ |
| R23-T03 | `backup/mod.rs` — replace `.unwrap()` on `read_to_string` with `?` | ⬜ |
| R23-T04 | `update_book_details` — wrap 4 writes in a single transaction | ⬜ |
| R23-T05 | `similar_queries.rs` — eliminate N+1 tag queries with GROUP_CONCAT JOIN | ⬜ |
| R23-T06 | Regex hot paths — `OnceLock<Regex>` at 7 compile-per-call sites | ⬜ |
| R23-T07 | `book_copy.rs` — `unique_id()` use full `as_nanos()`, not `subsec_nanos()` | ⬜ |
| R23-T08 | `main.rs` — `SqlitePoolOptions` set `.max_connections(1)` | ⬜ |
| R23-T09 | Migration — add `idx_book_tags_tag_id` index | ⬜ |

---

## R23-T01 — `sanitize_sort_field` returns `Result`

**File:** `processing/src/db/vlib_execute.rs`

The current implementation silently maps unknown sort fields to `"title"`. Callers receive no signal
that the requested sort field was invalid, hiding bugs in the frontend/command layer.

Replace the existing `sanitize_sort_field` function and its one call-site:

```rust
// Replace the function:
fn sanitize_sort_field(field: &str) -> Result<&'static str, crate::error::ProcessingError> {
    match field {
        "title" | "authors" | "format" | "last_opened_at" | "progress_percent" => Ok(field),
        _ => Err(crate::error::ProcessingError::InvalidInput(
            format!("unknown sort field: {field}"),
        )),
    }
}
```

Update the call-site in the same file (the query-building block) to propagate the error:

```rust
// Before:
let sort_col = sanitize_sort_field(&params.sort_field);

// After:
let sort_col = sanitize_sort_field(&params.sort_field)?;
```

Make sure `ProcessingError` has an `InvalidInput(String)` variant. If it does not, add one:

```rust
// In processing/src/error.rs, inside the ProcessingError enum:
#[error("invalid input: {0}")]
InvalidInput(String),
```

```bash
cargo check -p xcalibre-processing 2>&1 | grep "^error"
cargo test -p xcalibre-processing -- sanitize 2>&1 | tail -5
git add processing/src/db/vlib_execute.rs processing/src/error.rs
git commit -m "R23-T01: sanitize_sort_field returns Result — silent fallback removed"
```

---

## R23-T02 — `sessions.lock().unwrap()` → error propagation

**File:** `src-tauri/src/commands.rs` (approximately 7 call-sites around lines 1820–1870)

`Mutex::lock()` returns `Err` only if another thread panicked while holding the lock (poison). Using
`.unwrap()` here means a single panic anywhere propagates to every subsequent command call.

Search for all occurrences and replace:

```bash
grep -n "sessions\.lock()\.unwrap()" src-tauri/src/commands.rs
```

For each occurrence, replace:

```rust
// Before:
let mut sessions = sessions.lock().unwrap();

// After:
let mut sessions = sessions.lock().map_err(|_| "session lock poisoned".to_string())?;
```

The enclosing functions already return `Result<_, String>`, so `?` works without signature changes.

```bash
cargo check --workspace 2>&1 | grep "^error"
# Confirm zero remaining unwraps on sessions lock:
grep -c "sessions\.lock()\.unwrap()" src-tauri/src/commands.rs
git add src-tauri/src/commands.rs
git commit -m "R23-T02: sessions.lock().unwrap() → map_err + ? at all 7 sites"
```

---

## R23-T03 — `backup/mod.rs` read_to_string panic

**File:** `processing/src/backup/mod.rs` (~line 130)

A corrupted or truncated backup archive entry causes a panic instead of a recoverable error.

```rust
// Before:
std::io::Read::read_to_string(&mut entry, &mut json).unwrap();

// After:
std::io::Read::read_to_string(&mut entry, &mut json)
    .map_err(|e| crate::error::ProcessingError::IoError(e.to_string()))?;
```

If `ProcessingError::IoError` does not exist, use whichever IO/generic variant is present. Check
with `grep "IoError\|Io(" processing/src/error.rs`.

```bash
cargo check -p xcalibre-processing 2>&1 | grep "^error"
cargo test -p xcalibre-processing -- backup 2>&1 | tail -10
git add processing/src/backup/mod.rs
git commit -m "R23-T03: backup read_to_string unwrap → propagated error"
```

---

## R23-T04 — `update_book_details` atomic transaction

**File:** `src-tauri/src/commands.rs` (~line 192, function `update_book_details`)

The current implementation issues 4 separate SQL writes (UPDATE books, DELETE book_tags, INSERT
book_tags, UPDATE/INSERT metadata) outside a transaction. A failure midway leaves the database
in a partially-updated state.

Find the function and wrap all writes in a transaction:

```rust
pub async fn update_book_details(
    state: tauri::State<'_, AppState>,
    book_id: String,
    title: String,
    authors: Vec<String>,
    tags: Vec<String>,
) -> Result<(), String> {
    let pool = &state.pool;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    // UPDATE books
    sqlx::query("UPDATE local_books SET title = ?, authors_json = ? WHERE id = ?")
        .bind(&title)
        .bind(serde_json::to_string(&authors).unwrap_or_default())
        .bind(&book_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    // DELETE existing tags
    sqlx::query("DELETE FROM book_tags WHERE book_id = ?")
        .bind(&book_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    // INSERT new tags (upsert tag, then link)
    for tag_name in &tags {
        sqlx::query(
            "INSERT INTO tags (name) VALUES (?) ON CONFLICT(name) DO NOTHING",
        )
        .bind(tag_name)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            "INSERT INTO book_tags (book_id, tag_id) \
             SELECT ?, id FROM tags WHERE name = ?",
        )
        .bind(&book_id)
        .bind(tag_name)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}
```

Adapt the exact column names and extra writes to match what the current function actually does —
the principle is: open `pool.begin()`, replace every `.execute(pool)` with `.execute(&mut *tx)`,
commit at the end.

```bash
cargo check --workspace 2>&1 | grep "^error"
git add src-tauri/src/commands.rs
git commit -m "R23-T04: update_book_details wrapped in single transaction"
```

---

## R23-T05 — N+1 tag queries → single GROUP_CONCAT JOIN

**File:** `processing/src/db/similar_queries.rs` (~lines 61–75)

The current code fetches all candidate books, then fires one SELECT per book to fetch its tags.
On a 1 000-book library that is 1 001 queries where 1 would do.

Replace the loop with a single query using `GROUP_CONCAT`:

```rust
// Replace the candidate-fetch + per-book tag loop with:

#[derive(sqlx::FromRow)]
struct CandRow {
    id: String,
    title: String,
    authors_json: String,
    series_name: Option<String>,
    tags_csv: Option<String>,   // comma-separated tag names, may be NULL if no tags
}

let rows: Vec<CandRow> = sqlx::query_as(
    "SELECT b.id, b.title, b.authors_json, b.series_name,
            GROUP_CONCAT(t.name, ',') AS tags_csv
     FROM local_books b
     LEFT JOIN book_tags bt ON bt.book_id = b.id
     LEFT JOIN tags t       ON t.id = bt.tag_id
     WHERE b.id != ?
     GROUP BY b.id",
)
.bind(book_id)
.fetch_all(pool)
.await?;

// Parse tags_csv for each row:
for row in &rows {
    let tags: Vec<&str> = row
        .tags_csv
        .as_deref()
        .unwrap_or("")
        .split(',')
        .filter(|s| !s.is_empty())
        .collect();
    // ... rest of similarity logic using `tags`
}
```

Remove the old `for` loop and the inner `sqlx::query_as` call that fetched tags per book.

```bash
cargo check -p xcalibre-processing 2>&1 | grep "^error"
cargo test -p xcalibre-processing -- similar 2>&1 | tail -10
git add processing/src/db/similar_queries.rs
git commit -m "R23-T05: similar_queries N+1 eliminated — single GROUP_CONCAT JOIN"
```

---

## R23-T06 — Regex OnceLock at 7 hot-path sites

**Files:**
- `processing/src/convert/docx.rs` (2 regexes)
- `processing/src/text/kfx.rs` (1 regex)
- `processing/src/text/chm.rs` (1 regex)
- `processing/src/text/html.rs` (1 regex)
- `processing/src/metadata/html.rs` (2 regexes)

Each file currently calls `Regex::new(...)` inside a function that may be called thousands of
times per conversion. Move every `Regex::new` call to a `OnceLock<Regex>` static.

Pattern to apply in **each** affected file:

```rust
use std::sync::OnceLock;
use regex::Regex;

// Declare one static per regex, near the top of the file:
static RE_STRIP_TAGS: OnceLock<Regex> = OnceLock::new();

fn strip_tags_regex() -> &'static Regex {
    RE_STRIP_TAGS.get_or_init(|| Regex::new(r"<[^>]+>").expect("regex"))
}

// Then replace inline Regex::new(…) calls with the accessor:
// Before:  let re = Regex::new(r"<[^>]+>").unwrap();  re.replace_all(...)
// After:   strip_tags_regex().replace_all(...)
```

Apply this pattern to every `Regex::new` call found in the 7 files listed above. Use a
descriptive name for each static (e.g. `RE_HTML_TAG`, `RE_ENTITY`, `RE_HEADING`).

```bash
# Confirm no remaining Regex::new inside functions (only in OnceLock init closures remain):
grep -rn "Regex::new" processing/src/convert/docx.rs \
     processing/src/text/kfx.rs \
     processing/src/text/chm.rs \
     processing/src/text/html.rs \
     processing/src/metadata/html.rs
cargo check -p xcalibre-processing 2>&1 | grep "^error"
cargo test -p xcalibre-processing 2>&1 | tail -10
git add processing/src/convert/docx.rs \
        processing/src/text/kfx.rs \
        processing/src/text/chm.rs \
        processing/src/text/html.rs \
        processing/src/metadata/html.rs
git commit -m "R23-T06: OnceLock<Regex> at all 7 hot-path sites — regex compiled once"
```

---

## R23-T07 — `unique_id()` use full duration nanos

**File:** `processing/src/db/book_copy.rs` (~line 71)

`subsec_nanos()` returns only the nanosecond sub-second component (0–999_999_999). Two copies
created in the same second get the same ID. Use the full monotonic duration.

```rust
// Before:
fn unique_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    format!("copy-{nanos:08x}")
}

// After:
fn unique_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("copy-{nanos:016x}")
}
```

The format width changes from 8 to 16 hex digits to accommodate a u128. The column is `TEXT` in
SQLite so no schema change is needed.

```bash
cargo check -p xcalibre-processing 2>&1 | grep "^error"
git add processing/src/db/book_copy.rs
git commit -m "R23-T07: unique_id uses as_nanos() — subsec collision eliminated"
```

---

## R23-T08 — `SqlitePoolOptions` add `max_connections(1)`

**File:** `src-tauri/src/main.rs` (~line 31)

SQLite in WAL mode is safe for concurrent reads, but concurrent writers serialize and can produce
`SQLITE_BUSY`. Setting `max_connections(1)` ensures all commands share one connection and
eliminates busy-timeout races without needing retry logic.

```rust
// Before:
let pool = SqlitePoolOptions::new()
    .connect(&db_url)
    .await
    .expect("failed to open database");

// After:
let pool = SqlitePoolOptions::new()
    .max_connections(1)
    .connect(&db_url)
    .await
    .expect("failed to open database");
```

```bash
cargo check --workspace 2>&1 | grep "^error"
git add src-tauri/src/main.rs
git commit -m "R23-T08: SqlitePoolOptions max_connections(1) — eliminates SQLITE_BUSY races"
```

---

## R23-T09 — Migration: index on `book_tags(tag_id)`

**File:** the most recent migration file under `processing/src/db/migrations/` (or wherever
the CREATE TABLE migrations live — check with `ls processing/src/db/migrations/`).

The existing `book_tags` table likely has an index on `book_id` (used in DELETE/INSERT paths)
but not on `tag_id`. The `similar_queries` JOIN added in T05, and any query filtering by tag,
does a full scan of `book_tags` on the `tag_id` column.

Append to the latest migration file (or create a new one if the project uses numbered files):

```sql
CREATE INDEX IF NOT EXISTS idx_book_tags_tag_id ON book_tags(tag_id);
```

If the project applies migrations at startup via `sqlx::migrate!`, this will run automatically
on next launch. If migrations are embedded strings, add the `CREATE INDEX` statement to the
appropriate location.

```bash
# Verify the index appears in the schema after a test run:
cargo test -p xcalibre-processing -- migration 2>&1 | tail -10
# Or open the test DB and inspect:
# sqlite3 /tmp/test.db ".indexes book_tags"
git add processing/src/db/migrations/
git commit -m "R23-T09: idx_book_tags_tag_id — covers JOIN in similar_queries and tag filters"
```

---

## R23-T10 — Final check

```bash
cargo test --workspace 2>&1 | tail -15
cargo clippy --workspace -- -D warnings 2>&1 | grep "^error"
cd ui && npm test 2>&1 | tail -10 && cd ..
```

All tests must remain green. Fix any clippy warnings before committing.

```bash
git add -A
git commit -m "R23-T10: RMP-23 Correctness & Performance Hardening — all checks green"
```
