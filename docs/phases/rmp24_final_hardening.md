# RMP-24 — Final Hardening (Security, Correctness, Performance)

> Prerequisite: rmp23 complete.
> Status: ⬜ not started
> This is the last remediation phase. After this phase xcalibre is production-ready.

## Status

| Task | Title | Status |
|------|-------|--------|
| R24-T01 | Enable `PRAGMA foreign_keys = ON` at connection time | ⬜ |
| R24-T02 | Sandbox `write_file` command to app data dir | ⬜ |
| R24-T03 | Fix zip-slip in `list_comic_pages` | ⬜ |
| R24-T04 | Fix `jobs.format` CHECK constraint (blocks DOCX/FB2/DJVU ingest) | ⬜ |
| R24-T05 | Fix `ai_config` upsert — broken `ON CONFLICT(id)` | ⬜ |
| R24-T06 | Exclude `api_key` from `get_ai_config_cmd` response | ⬜ |
| R24-T07 | Remove `eprintln!` debug output from `get_epub_chapter_html` | ⬜ |
| R24-T08 | Wrap `bulk_delete_books` in a single transaction | ⬜ |
| R24-T09 | Fix `bulk_export_metadata` CSV — unescaped newlines in description | ⬜ |
| R24-T10 | Fix FTS MATCH error handling — return empty results on bad syntax | ⬜ |
| R24-T11 | Fix `get_collection_books_cmd` N+1 | ⬜ |
| R24-T12 | Fix `get_ai_context_chunks` broken word scoring | ⬜ |
| R24-T13 | `OnceLock<Selector>` for scraper hot paths | ⬜ |
| R24-T14 | Final sweep | ⬜ |

---

## CRITICAL — Read first before any task

- All backend changes go in `src-tauri/src/commands.rs` (monolithic file, ~2155 lines).
- Do NOT edit `src-tauri/src/commands/convert.rs` — it is orphaned, never compiled.
- Read every file before editing it.
- `cargo check --workspace 2>&1 | grep "^error"` after every file edit.
- One commit per task. Do not batch.
- **DO NOT use spawn_agent under any circumstances.** There is no valid agent type. Use it and Merlin stops cold.

---

## R24-T01 — Enable `PRAGMA foreign_keys = ON`

**File:** `src-tauri/src/main.rs`

SQLite FK cascades (`ON DELETE CASCADE`) and FK constraints are silently ignored unless this
pragma is set per-connection. With `max_connections(1)` the pool has exactly one connection, so
one pragma call covers everything.

`SqliteConnectOptions` supports `.pragma(key, value)`. Read the file, then add the pragma to
the existing `SqliteConnectOptions` chain:

```rust
// Before:
SqliteConnectOptions::new()
    .filename(&db_path)
    .create_if_missing(true),

// After:
SqliteConnectOptions::new()
    .filename(&db_path)
    .create_if_missing(true)
    .pragma("foreign_keys", "ON"),
```

```bash
cd /Users/jonzuilkowski/Documents/localProject/xcalibre
cargo check --workspace 2>&1 | grep "^error"
git add src-tauri/src/main.rs
git commit -m "R24-T01: PRAGMA foreign_keys = ON — FK cascades and constraints now enforced"
```

---

## R24-T02 — Sandbox `write_file` command

**File:** `src-tauri/src/commands.rs` (~line 699)

`write_file` accepts an arbitrary path from the frontend and writes to it with no validation.
A malicious EPUB-embedded script that escapes the iframe sandboxing can call this command and
write anywhere the process can reach (shell configs, cron files, app binaries).

Read the function, then add a path check:

```rust
#[tauri::command]
pub async fn write_file(
    path: String,
    contents: String,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let target = std::path::Path::new(&path)
        .canonicalize()
        .or_else(|_| {
            // File doesn't exist yet — canonicalize the parent instead
            std::path::Path::new(&path)
                .parent()
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "no parent"))
                .and_then(|p| p.canonicalize())
        })
        .map_err(|e| format!("invalid path: {e}"))?;

    let allowed = app.path().app_data_dir().map_err(|e| e.to_string())?;
    if !target.starts_with(&allowed) {
        return Err("write_file: path is outside the app data directory".to_string());
    }

    if let Some(parent) = std::path::Path::new(&path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }
    std::fs::write(path, contents).map_err(|e| e.to_string())
}
```

Note: the function signature gains `app: tauri::AppHandle`. Tauri injects this automatically —
no change needed in `main.rs` or the frontend caller.

```bash
cargo check --workspace 2>&1 | grep "^error"
git add src-tauri/src/commands.rs
git commit -m "R24-T02: write_file sandboxed to app data dir — path traversal blocked"
```

---

## R24-T03 — Fix zip-slip in `list_comic_pages`

**File:** `src-tauri/src/commands.rs` (~lines 1280–1295)

`entry.name()` in a CBZ can contain `../../` sequences. The code calls `.file_name()` to strip
them, but the `unwrap_or` fallback restores the raw untrusted name when `.file_name()` returns
`None` (which happens for entries like `../evil`).

Read the function, then replace the `out_path` construction:

```rust
// Before:
let out_path = temp_dir.join(
    std::path::Path::new(&name)
        .file_name()
        .unwrap_or(std::ffi::OsStr::new(&name)),
);

// After:
let safe_name = std::path::Path::new(&name)
    .file_name()
    .ok_or_else(|| format!("unsafe zip entry name: {name}"))?;
let out_path = temp_dir.join(safe_name);
```

The `?` propagates to the `Result<Vec<String>, String>` return type and skips dangerous entries.

```bash
cargo check --workspace 2>&1 | grep "^error"
git add src-tauri/src/commands.rs
git commit -m "R24-T03: zip-slip fixed in list_comic_pages — unsafe entry names rejected"
```

---

## R24-T04 — Fix `jobs.format` CHECK constraint

**File:** `processing/src/db/migrations/` — add a new file `0020_jobs_format_any.sql`

The `jobs` table was created with:
```sql
format TEXT NOT NULL CHECK (format IN ('EPUB','PDF','MOBI','AZW3','CBZ','CBR','TXT'))
```

The app now ingests DOCX, FB2, DJVU, CHM, LRF, PDB, SNB, ODT, HTML, HTMLZ, RTF, KFX, LIT,
and more. Every ingest of these formats fails silently or hard with a CHECK constraint violation.

SQLite cannot `ALTER TABLE ... DROP CONSTRAINT`. The fix is to recreate the table.

Create `processing/src/db/migrations/0020_jobs_format_any.sql`:

```sql
-- Recreate jobs table without the format CHECK constraint.
-- The original constraint only allowed 7 formats; the app now supports 20+.
PRAGMA foreign_keys = OFF;

CREATE TABLE jobs_new (
    id                TEXT PRIMARY KEY,
    file_path         TEXT NOT NULL,
    file_sha256       TEXT NOT NULL,
    format            TEXT NOT NULL,
    status            TEXT NOT NULL DEFAULT 'PENDING'
                          CHECK (status IN ('PENDING','READY_TO_PUSH','PUSHING','RETRYING','COMPLETED','FAILED')),
    retry_count       INTEGER NOT NULL DEFAULT 0,
    next_retry_at     TEXT,
    xs_book_id        TEXT,
    push_step         INTEGER NOT NULL DEFAULT 0,
    error_message     TEXT,
    created_at        TEXT NOT NULL,
    updated_at        TEXT NOT NULL
);

INSERT INTO jobs_new SELECT * FROM jobs;
DROP TABLE jobs;
ALTER TABLE jobs_new RENAME TO jobs;

CREATE INDEX IF NOT EXISTS idx_jobs_status ON jobs(status);
CREATE INDEX IF NOT EXISTS idx_jobs_sha256 ON jobs(file_sha256);

PRAGMA foreign_keys = ON;
```

```bash
cargo check -p xcalibre-processing 2>&1 | grep "^error"
cargo test -p xcalibre-processing 2>&1 | tail -10
git add processing/src/db/migrations/0020_jobs_format_any.sql
git commit -m "R24-T04: remove jobs.format CHECK constraint — all 20+ formats can now ingest"
```

---

## R24-T05 — Fix `ai_config` upsert

**File:** `processing/src/db/ai_queries.rs` (~line 27) and migration

The upsert uses `ON CONFLICT(id)` but `id` has a `DEFAULT (lower(hex(randomblob(16))))` — it
is always new, so the conflict never fires. Every `save_ai_config_cmd` call inserts a fresh row
instead of updating the existing config. The user's saved AI settings silently accumulate as
duplicate rows; only the first row (lowest rowid) is ever read.

**Step 1** — add a UNIQUE constraint on `library_id` so the conflict target is meaningful.
Create `processing/src/db/migrations/0021_ai_config_unique_library.sql`:

```sql
CREATE UNIQUE INDEX IF NOT EXISTS ai_config_library_id ON ai_config(library_id);
```

**Step 2** — update the upsert in `processing/src/db/ai_queries.rs` to conflict on the index:

```rust
// Replace the existing upsert query:
sqlx::query(
    "INSERT INTO ai_config
     (library_id, provider, model, embed_model, base_url, api_key,
      reasoning_strategy, include_fields, updated_at)
     VALUES (?,?,?,?,?,?,?,?,?)
     ON CONFLICT(library_id) DO UPDATE SET
      provider=excluded.provider, model=excluded.model,
      embed_model=excluded.embed_model, base_url=excluded.base_url,
      api_key=COALESCE(excluded.api_key, ai_config.api_key),
      reasoning_strategy=excluded.reasoning_strategy,
      include_fields=excluded.include_fields, updated_at=excluded.updated_at",
)
```

Note the `api_key=COALESCE(excluded.api_key, ai_config.api_key)` — this preserves an existing
key when the update omits it (since `save_ai_config_cmd` passes `api_key: None`).

```bash
cargo check -p xcalibre-processing 2>&1 | grep "^error"
cargo test -p xcalibre-processing 2>&1 | tail -5
git add processing/src/db/ai_queries.rs \
        processing/src/db/migrations/0021_ai_config_unique_library.sql
git commit -m "R24-T05: ai_config upsert fixed — ON CONFLICT(library_id) + key preservation"
```

---

## R24-T06 — Exclude `api_key` from `get_ai_config_cmd`

**File:** `src-tauri/src/commands.rs` (~line 1503)

`get_ai_config_cmd` returns the full `AiConfig` struct including `api_key` as plaintext JSON
to the WebView. Any script running in the app (including EPUB-embedded JS) can call
`invoke("get_ai_config_cmd")` and exfiltrate it.

Add a separate DTO that omits the key, or redact it before returning:

```rust
#[derive(Debug, serde::Serialize)]
pub struct AiConfigPublic {
    pub provider:           String,
    pub model:              String,
    pub embed_model:        String,
    pub base_url:           String,
    pub has_api_key:        bool,   // frontend uses this to show a "key saved" indicator
    pub reasoning_strategy: String,
    pub include_fields:     String,
}

#[tauri::command]
pub async fn get_ai_config_cmd(
    pool: tauri::State<'_, std::sync::Arc<sqlx::SqlitePool>>,
) -> Result<Option<AiConfigPublic>, String> {
    let cfg = get_ai_config(pool.inner().as_ref(), None)
        .await.map_err(|e| e.to_string())?;
    Ok(cfg.map(|c| AiConfigPublic {
        provider:           c.provider,
        model:              c.model,
        embed_model:        c.embed_model,
        base_url:           c.base_url,
        has_api_key:        c.api_key.is_some(),
        reasoning_strategy: c.reasoning_strategy,
        include_fields:     c.include_fields,
    }))
}
```

Search the UI for any TypeScript code reading `cfg.api_key` and update it to use `cfg.has_api_key`:
```bash
grep -rn "api_key\|apiKey" ui/src/ --include="*.tsx" --include="*.ts"
```

Adjust any UI component that renders or uses `api_key` to use `has_api_key` as a boolean flag.

```bash
cargo check --workspace 2>&1 | grep "^error"
git add src-tauri/src/commands.rs
git commit -m "R24-T06: api_key excluded from get_ai_config_cmd response — key no longer exposed to WebView"
```

---

## R24-T07 — Remove debug `eprintln!` from `get_epub_chapter_html`

**File:** `src-tauri/src/commands.rs` (~lines 471–487)

Two `eprintln!` calls log `book_id`, `href`, and response byte counts to stderr on every chapter
load in the reader. In a production desktop app this leaks reading history to any process or log
aggregator that captures stderr.

```rust
// Delete these two eprintln! calls entirely:
eprintln!("get_epub_chapter_html:start book_id={} href={}", book_id, href);
// ...
eprintln!(
    "get_epub_chapter_html:done book_id={} href={} bytes={}",
    book_id,
    href,
    rendered.len()
);
```

```bash
cargo check --workspace 2>&1 | grep "^error"
git add src-tauri/src/commands.rs
git commit -m "R24-T07: remove eprintln debug output from get_epub_chapter_html"
```

---

## R24-T08 — Wrap `bulk_delete_books` in a single transaction

**File:** `src-tauri/src/commands.rs` (~lines 972–1036)

`bulk_delete_books` issues 7 separate SQL statements per book with no transaction. For 100 books
that is 700 individual round-trips to SQLite. A crash midway leaves the library in a partially
deleted state. Additionally, FTS and `book_formats` deletes use `let _ =` (fire-and-forget),
which discards errors silently.

Read the function in full, then rewrite it to use a transaction:

```rust
#[tauri::command]
pub async fn bulk_delete_books(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    book_ids: Vec<String>,
) -> Result<u64, String> {
    let pool = pool.inner().as_ref();
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    let mut count = 0u64;

    for id in &book_ids {
        // Collect cover path before deletion
        let cover: Option<(Option<String>,)> = sqlx::query_as(
            "SELECT cover_path FROM local_books WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        // Delete from all dependent tables
        for sql in &[
            "DELETE FROM books_fts WHERE book_id = ?",
            "DELETE FROM book_chunks WHERE book_id = ?",
            "DELETE FROM job_text WHERE job_id = ?",
            "DELETE FROM book_formats WHERE book_id = ?",
            "DELETE FROM book_tags WHERE book_id = ?",
            "DELETE FROM identifiers WHERE book_id = ?",
            "DELETE FROM collection_books WHERE book_id = ?",
            "DELETE FROM annotations WHERE book_id = ?",
            "DELETE FROM notes WHERE book_id = ?",
            "DELETE FROM reading_sessions WHERE book_id = ?",
            "DELETE FROM push_queue WHERE job_id = ?",
        ] {
            sqlx::query(sql)
                .bind(id)
                .execute(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;
        }

        let result = sqlx::query("DELETE FROM local_books WHERE id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
        count += result.rows_affected();

        sqlx::query("DELETE FROM jobs WHERE id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

        // Delete cover file after DB rows are gone (best-effort)
        if let Some((Some(path),)) = cover {
            if !path.is_empty() {
                if let Err(err) = std::fs::remove_file(&path) {
                    log::warn!("could not delete cover {}: {}", path, err);
                }
            }
        }
    }

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(count)
}
```

Note: this version also adds `annotations`, `notes`, `reading_sessions`, `book_chunks` to the
delete chain — tables the original missed.

```bash
cargo check --workspace 2>&1 | grep "^error"
git add src-tauri/src/commands.rs
git commit -m "R24-T08: bulk_delete_books in transaction — atomic, covers all dependent tables"
```

---

## R24-T09 — Fix `bulk_export_metadata` CSV escaping

**File:** `src-tauri/src/commands.rs` (~lines 1055–1105)

`bulk_export_metadata` uses `title.replace(',', ";")` but does not handle newlines (`\n`, `\r`)
or double-quotes in `description`, `publisher`, or `series_name`. A book description with a
newline breaks the CSV structure entirely.

Read the function. Add a local `csv_escape` helper and apply it to every field:

```rust
fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
```

Then replace the unescaped format! call:

```rust
csv.push_str(&format!(
    "{},{},{},{},{},{},{},{},{}\n",
    csv_escape(&id),
    csv_escape(&title),
    csv_escape(&authors.join("; ")),
    csv_escape(&format),
    csv_escape(publisher.as_deref().unwrap_or("")),
    csv_escape(series_name.as_deref().unwrap_or("")),
    series_index.map(|f| f.to_string()).unwrap_or_default(),
    csv_escape(pubdate.as_deref().unwrap_or("")),
    csv_escape(description.as_deref().unwrap_or("")),
));
```

```bash
cargo check --workspace 2>&1 | grep "^error"
git add src-tauri/src/commands.rs
git commit -m "R24-T09: bulk_export_metadata CSV — proper quoting for newlines and commas"
```

---

## R24-T10 — FTS MATCH error handling

**Files:** `processing/src/db/fts_queries.rs`, `processing/src/db/queries.rs`,
`processing/src/db/notes_queries.rs`

SQLite FTS5 `MATCH` throws a syntax error for user queries containing unbalanced quotes,
leading `*`, `-"`, or bare `AND`/`OR`/`NOT` operators. Currently these propagate as raw sqlx
errors through the command layer to the UI, producing opaque error messages instead of empty
search results.

For each of the three FTS query functions, wrap the fetch in a match and return an empty Vec
when a FTS syntax error occurs:

```rust
// In fts_queries.rs, search():
pub async fn search(pool: &SqlitePool, query: &str) -> Result<Vec<String>, ProcessingError> {
    let result = sqlx::query_as::<_, (String,)>(
        "SELECT DISTINCT book_id FROM books_fts WHERE books_fts MATCH ? ORDER BY rank",
    )
    .bind(query)
    .fetch_all(pool)
    .await;

    match result {
        Ok(rows) => Ok(rows.into_iter().map(|(id,)| id).collect()),
        Err(sqlx::Error::Database(e)) if e.message().contains("fts5:") || e.message().contains("malformed MATCH") => {
            Ok(vec![])   // bad FTS syntax → return empty results, not an error
        }
        Err(e) => Err(ProcessingError::DbError(e)),
    }
}
```

Apply the same pattern to:
- `search_books` in `queries.rs` (the FTS JOIN query)
- `search_notes` in `notes_queries.rs`

```bash
cargo check -p xcalibre-processing 2>&1 | grep "^error"
git add processing/src/db/fts_queries.rs \
        processing/src/db/queries.rs \
        processing/src/db/notes_queries.rs
git commit -m "R24-T10: FTS MATCH syntax errors return empty results instead of propagating"
```

---

## R24-T11 — Fix `get_collection_books_cmd` N+1

**File:** `src-tauri/src/commands.rs` (~lines 636–654)

`get_collection_books_cmd` calls `get_collection_books` to get a list of job IDs, then calls
`fetch_book_by_id` once per book. For a collection of 200 books that is 201 SQL queries.

Replace the loop with a single JOIN query inlined in the command:

```rust
#[tauri::command]
pub async fn get_collection_books_cmd(
    pool: tauri::State<'_, Arc<SqlitePool>>,
    collection_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    let rows = sqlx::query_as::<_, BookRow>(
        "SELECT lb.id, lb.title, lb.authors_json, lb.format, lb.cover_path, lb.local_path,
                lb.progress_percent, lb.last_opened_at, lb.reading_cfi
         FROM local_books lb
         JOIN collection_books cb ON cb.book_id = lb.id
         WHERE cb.collection_id = ?
         ORDER BY cb.added_at ASC",
    )
    .bind(&collection_id)
    .fetch_all(pool.inner().as_ref())
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(book_row_to_value).collect())
}
```

`BookRow` and `book_row_to_value` already exist at the top of `commands.rs` — reuse them directly.

```bash
cargo check --workspace 2>&1 | grep "^error"
git add src-tauri/src/commands.rs
git commit -m "R24-T11: get_collection_books_cmd N+1 eliminated — single JOIN query"
```

---

## R24-T12 — Fix `get_ai_context_chunks` word scoring

**File:** `src-tauri/src/commands.rs` (~lines 1528–1537)

The scoring loop uses `query_lower.contains(*w)` where `w` is a word from the text chunk. This
is substring matching: for query `"dragon"`, text words `"drag"`, `"rag"`, and `"on"` all score
positive because `"dragon".contains("drag")` is true. The ranking is meaningless.

The intent is to count how many chunk words appear in the query. Use word-level matching:

```rust
#[tauri::command]
pub async fn get_ai_context_chunks(
    pool: tauri::State<'_, std::sync::Arc<sqlx::SqlitePool>>,
    book_id: String,
    query: String,
) -> Result<Vec<String>, String> {
    let chunks = get_book_chunks(pool.inner().as_ref(), &book_id)
        .await.map_err(|e| e.to_string())?;

    let query_words: std::collections::HashSet<&str> =
        query.split_whitespace().collect();

    let mut scored: Vec<(usize, &str)> = chunks.iter()
        .map(|(_, text, _)| {
            let score = text
                .split_whitespace()
                .filter(|w| query_words.contains(*w))
                .count();
            (score, text.as_str())
        })
        .collect();

    scored.sort_by(|a, b| b.0.cmp(&a.0));
    Ok(scored.into_iter().take(5).map(|(_, t)| t.to_string()).collect())
}
```

```bash
cargo check --workspace 2>&1 | grep "^error"
git add src-tauri/src/commands.rs
git commit -m "R24-T12: get_ai_context_chunks word scoring — exact word match replaces substring"
```

---

## R24-T13 — `OnceLock<Selector>` for scraper hot paths

**Files:**
- `processing/src/metadata/html.rs` — `Selector::parse("title")`, `Selector::parse("meta")`
- `processing/src/text/html.rs` — `Selector::parse("p, h1, h2, h3, h4, li, td, th")`
- `processing/src/metadata/chm.rs` — `Selector::parse("title")`, `Selector::parse("meta")`

These selectors are re-parsed on every metadata extraction call. Apply the same `OnceLock`
pattern used in RMP-23-T06 for regexes:

```rust
use std::sync::OnceLock;

static SEL_TITLE: OnceLock<scraper::Selector> = OnceLock::new();
fn sel_title() -> &'static scraper::Selector {
    SEL_TITLE.get_or_init(|| scraper::Selector::parse("title").expect("selector"))
}

static SEL_META: OnceLock<scraper::Selector> = OnceLock::new();
fn sel_meta() -> &'static scraper::Selector {
    SEL_META.get_or_init(|| scraper::Selector::parse("meta").expect("selector"))
}
```

Then replace all `.select(&scraper::Selector::parse(...).unwrap())` call-sites with
`.select(sel_title())` etc.

Apply one file at a time. `cargo check` after each file.

```bash
cargo check -p xcalibre-processing 2>&1 | grep "^error"
git add processing/src/metadata/html.rs \
        processing/src/text/html.rs \
        processing/src/metadata/chm.rs
git commit -m "R24-T13: OnceLock<Selector> for scraper hot paths — selectors parsed once"
```

---

## R24-T14 — Final sweep

```bash
cd /Users/jonzuilkowski/Documents/localProject/xcalibre
cargo test --workspace 2>&1 | tail -15
cargo clippy --workspace -- -D warnings 2>&1 | grep "^error"
cd ui && npm test 2>&1 | tail -15 && cd ..
git log --oneline | head -16
```

All tests green. No clippy errors. Git log shows R24-T01 through R24-T13.

```bash
git add -A
git commit -m "R24-T14: RMP-24 Final Hardening complete — xcalibre production-ready"
```
