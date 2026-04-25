# xCalibre — Rust Architecture

This document describes the architecture of the xCalibre Rust implementation.
For the language-agnostic design (data model, pipeline stages, protocols), see AGNOSTIC.md.
For phase-by-phase build instructions, see phase1_commands.md through phase10_commands.md.

---

## Workspace Layout

```
xcalibre/
├── Cargo.toml                  ← workspace root (resolver = "2")
├── processing/                 ← core processing engine (binary + library)
│   ├── src/
│   │   ├── lib.rs              ← public module exports
│   │   ├── main.rs             ← CLI entry point (clap)
│   │   ├── error.rs            ← ProcessingError (thiserror)
│   │   ├── config.rs           ← Config struct (toml + env vars)
│   │   ├── commands.rs         ← shared EPUB spine helper for Tauri
│   │   ├── enrichment_prompt.rs ← CLI confirmation prompt for suggestions
│   │   ├── import/             ← Calibre import helpers
│   │   │   └── calibre.rs      ← walk a Calibre library and queue ingest jobs
│   │   ├── pipeline/           ← processing pipeline + background jobs
│   │   │   ├── ingest.rs       ← stage 1: detect, validate, hash, record
│   │   │   ├── metadata.rs     ← stage 2: extract and persist metadata
│   │   │   ├── enrichment.rs   ← stage 3: ISBN lookup + suggestions
│   │   │   ├── text.rs         ← stage 4: extract full text
│   │   │   ├── cover.rs        ← stage 5: extract and resize cover
│   │   │   ├── push.rs         ← stage 6: push to xcalibre-server + retry
│   │   │   ├── sync.rs         ← background pull/back-propagate jobs
│   │   │   └── retry.rs        ← background retry worker for failed push jobs
│   │   ├── plugins/            ← format definitions
│   │   │   └── mod.rs          ← DetectedFormat enum
│   │   ├── metadata/           ← format-specific metadata extractors
│   │   │   ├── mod.rs          ← BookMetadata struct
│   │   │   ├── epub.rs
│   │   │   ├── pdf.rs          ← stub (v1)
│   │   │   ├── mobi.rs         ← stub (v1)
│   │   │   ├── isbn.rs         ← EPUB/PDF ISBN detection
│   │   │   └── enrichment.rs   ← enrichment suggestion merge logic
│   │   ├── text/               ← format-specific text extractors
│   │   │   ├── mod.rs          ← ExtractedText struct
│   │   │   └── epub.rs
│   │   ├── cover/              ← format-specific cover extractors
│   │   │   ├── mod.rs          ← CoverResult struct
│   │   │   ├── epub.rs
│   │   │   └── resize.rs       ← image resize to 500×750 JPEG
│   │   ├── db/                 ← database layer
│   │   │   ├── mod.rs
│   │   │   ├── queries.rs      ← typed query functions (non-macro sqlx)
│   │   │   ├── enrichment_cache.rs
│   │   │   └── migrations/     ← SQL migration files (sqlx migrate!)
│   │   └── utils/
│   │       ├── mod.rs
│   │       ├── hash.rs         ← sha256_file
│   │       └── normalise.rs    ← text normalisation
│   └── tests/                  ← integration tests (in-memory SQLite)
├── api/                        ← xcalibre-server API client
│   └── src/
│       ├── lib.rs
│       ├── client.rs           ← ApiClient (reqwest + keyring)
│       ├── push.rs             ← push_book endpoint
│       ├── read.rs             ← get_book / list_books read endpoints
│       └── enrichment/         ← Open Library + Google Books lookups
├── src-tauri/                  ← Tauri v2 desktop shell
│   └── src/
│       ├── main.rs             ← Tauri builder, DB init, manage state
│       ├── commands.rs         ← #[tauri::command] handlers
│       └── epub_protocol.rs    ← epub:// custom URI scheme handler
└── ui/                         ← React frontend (Vite + TypeScript + Tailwind)
    └── src/
        ├── App.tsx
        ├── main.tsx
        ├── store/
        │   ├── libraryStore.ts  ← Zustand: book list + fetch
        │   └── settingsStore.ts ← Zustand persist: font, theme
        ├── components/
        │   ├── LibraryView.tsx
        │   ├── BookDetail.tsx
        │   ├── SearchBar.tsx
        │   ├── FilterBar.tsx
        │   ├── CollectionsSidebar.tsx
        │   └── BulkActionBar.tsx
        └── reader/
            └── cfi.ts          ← CFI position tracking + progress calc
```

---

## Crate Responsibilities

### `xcalibre-processing` (library + binary)

The core crate. Exposes a library used by both the CLI (`main.rs`) and the
Tauri shell (`src-tauri`). Contains all business logic.

**Error handling:** `ProcessingError` (thiserror) is the canonical error type
for all library code. `anyhow::Result` is used in `main.rs` and tests only.

**Async runtime:** Tokio (`#[tokio::main]`). All pipeline stages are async.

**Database:** sqlx 0.8 with SQLite, runtime-tokio-rustls driver, migrate feature.
Queries use the non-macro form (`sqlx::query_as::<_, TupleType>()`) because
`DATABASE_URL` is not available at compile time. Migrations run via
`sqlx::migrate!("src/db/migrations")` at startup.

**No `unwrap()` in library code.** All fallible operations use `?` with
`ProcessingError`.

Additional helpers in the processing crate:
- `config.rs` loads `config.toml` and environment overrides.
- `commands.rs` provides shared EPUB spine parsing used by the desktop shell.
- `enrichment_prompt.rs` gates metadata enrichment on explicit user approval.
- `import/calibre.rs` walks a Calibre library folder, deduplicates files, and
  queues new ingest jobs.

### `xcalibre-api`

Thin HTTP client for the xcalibre-server remote API.

- `ApiClient::from_keyring()` reads the service token from the OS keychain
  via the `keyring` crate. Token is never stored in config files or logs.
- `ApiClient::new(base_url, token)` is available when the token has already
  been resolved by the caller.
- `push.rs` requests use a 10-second timeout and surface failures to the
  caller; the caller decides retry policy.
- `read.rs` requests use a 5-second timeout and return empty results or
  `None` for non-fatal read failures.
- `enrichment/` providers use a 10-second timeout and silent fallback, so
  enrichment never blocks processing.

### `app` (src-tauri)

Tauri v2 application shell. Owns:
- Database initialisation at startup (`sqlx migrate!` + `Arc<SqlitePool>` managed state)
- Tauri command registration (`invoke_handler`)
- Custom `epub://` URI scheme for serving EPUB assets to the WebView
- No business logic — all calls delegate to `xcalibre-processing`
- Command surface for listing/filtering/searching books, managing collections,
  bulk library actions, Calibre import, and reader state updates

---

## Processing Pipeline

Each stage is a standalone async function. Stages run sequentially per job.
A job can be re-entered at any failed stage.

```
run_ingest()     → IngestResult    (job_id, format, sha256, file_size)
run_metadata()   → BookMetadata    (title, authors, language, …)
run_enrichment() → Vec<EnrichmentSuggestion> (ISBN lookup + user-confirmed metadata)
run_text()       → ()              (writes to job_text table)
run_cover()      → Option<PathBuf> (writes JPEG to disk, updates local_books)
run_push()       → ()              (push to xcalibre-server or skip if no client)
```

### Stage 1 — Ingest (`pipeline/ingest.rs`)

1. Verify file exists and is ≤ 500 MB
2. Detect format by magic bytes (`detect_format`)
3. Validate structural integrity per format (`validate_integrity`)
4. Compute SHA-256 (`sha256_file`)
5. Check for duplicate: `SELECT id FROM jobs WHERE file_sha256 = ? AND status = 'COMPLETED'`
6. Insert `jobs` row with status `PENDING`, return `IngestResult`

Format detection reads up to 128 bytes with `file.read()` (not `read_exact` —
handles files smaller than 128 bytes). Magic byte table: see AGNOSTIC.md §4.

### Stage 2 — Metadata (`pipeline/metadata.rs`)

Dispatches to format-specific extractor, serialises `BookMetadata` as JSON,
upserts into `job_metadata(job_id, metadata_json, extracted_at)`.

### Stage 3 — Enrichment (`pipeline/enrichment.rs`)

If an ISBN is available, fetches Open Library and Google Books data with a
10-second timeout per provider, stores the payload in `enrichment_cache`, and
builds suggestion records. Suggestions are only applied after explicit user
confirmation.

### Stage 4 — Text (`pipeline/text.rs`)

EPUB only (v1). Parses spine, strips HTML tags via regex, counts words,
upserts into `job_text(job_id, full_text, word_count, user_edited, updated_at)`.

### Stage 5 — Cover (`pipeline/cover.rs`)

EPUB only (v1). Extracts cover image bytes, resizes to max 500×750 JPEG via
`image` crate, writes to `<original_path>.cover.jpg`, updates `local_books.cover_path`.

### Stage 6 — Push (`pipeline/push.rs`)

If no `ApiClient` is provided, logs and returns `Ok(())` — offline-first.
On success: sets `jobs.status = COMPLETED`, stores `xs_book_id`.
On failure: increments `retry_count`, sets `next_retry_at` with exponential
backoff (5 × retry_count minutes), sets status `RETRYING`. After 5 failures:
status `FAILED`.

### Background Jobs

- `sync_pull()` fetches remote books into `local_books` without blocking the
  normal ingest pipeline.
- `sync_status_backprop()` re-queues local jobs when a completed remote book
  has been deleted.
- `run_retries()` / `start_retry_worker()` retry failed push jobs on a timer.

---

## Database Schema

Single SQLite file. Location: `~/Library/Application Support/xcalibre/jobs.db`
(macOS), `~/.local/share/xcalibre/jobs.db` (Linux), `%APPDATA%\xcalibre\jobs.db` (Windows).

Managed by sqlx migrations in `processing/src/db/migrations/`:

| Migration | Contents |
|---|---|
| `0001_jobs.sql` | `jobs`, `job_metadata`, `job_text`, `local_books`, `push_queue` |
| `0002_enrichment_cache.sql` | `enrichment_cache` |
| `0003_reading_position.sql` | `ALTER TABLE local_books ADD COLUMN reading_cfi` |
| `0004_bookmarks.sql` | `bookmarks` |
| `0005_fts.sql` | `books_fts` |
| `0006_fts_triggers.sql` | `books_fts` population triggers |
| `0007_collections.sql` | `collections`, `collection_books` |

### Key Tables

**`jobs`** — one row per file ingested
```sql
id TEXT PK, file_path TEXT, file_sha256 TEXT,
format TEXT CHECK(format IN ('EPUB','PDF','MOBI','AZW3','CBZ','CBR','TXT')),
status TEXT CHECK(status IN ('PENDING','READY_TO_PUSH','PUSHING','RETRYING','COMPLETED','FAILED')),
retry_count INTEGER, next_retry_at TEXT,
xs_book_id TEXT, push_step INTEGER,
error_message TEXT, created_at TEXT, updated_at TEXT
```

**`local_books`** — library view (populated from push results or Calibre import)
```sql
id TEXT PK, title TEXT, authors_json TEXT, format TEXT,
local_path TEXT, cover_path TEXT,
reading_cfi TEXT, reading_position TEXT,
progress_percent REAL, last_opened_at TEXT,
created_at TEXT, updated_at TEXT
```

**`job_metadata`** — extracted metadata JSON, one row per job
**`job_text`** — extracted full text, one row per job
**`enrichment_cache`** — ISBN → metadata JSON, expires after 30 days
**`collections`** — named library collections
**`collection_books`** — many-to-many join table between collections and books

### Full-Text Search

`books_fts` is a normal SQLite FTS5 table populated from `local_books` via
triggers. It stores searchable title/author/text content and is joined back to
library rows when the UI performs search.

---

## Query Pattern

All queries use the non-macro `sqlx::query_as` form with explicit tuple types:

```rust
let row = sqlx::query_as::<_, (String, String, ...)>(
    "SELECT id, file_path, ... FROM jobs WHERE id = ?",
)
.bind(id)
.fetch_optional(pool)
.await
.map_err(ProcessingError::DbError)?
.map(|(id, file_path, ...)| Job { id, file_path, ... });
```

All `.map_err` calls use the function-pointer form (`ProcessingError::IoError`)
not the closure form (`|e| ProcessingError::IoError(e)`) — required by clippy.

---

## Tauri Command Layer

Commands live in `src-tauri/src/commands.rs`. Each command receives
`tauri::State<'_, Arc<SqlitePool>>` and delegates to processing library functions.

```rust
#[tauri::command]
pub async fn list_books(pool: State<'_, Arc<SqlitePool>>) -> Result<Vec<Value>, String>
#[tauri::command]
pub async fn filter_library(pool: State<'_, Arc<SqlitePool>>, format: Option<String>, status: Option<String>, author: Option<String>) -> Result<Vec<Value>, String>
#[tauri::command]
pub async fn search_library(pool: State<'_, Arc<SqlitePool>>, query: String) -> Result<Vec<Value>, String>
#[tauri::command]
pub async fn ingest_file(pool: State<'_, Arc<SqlitePool>>, path: String) -> Result<String, String>
#[tauri::command]
pub async fn create_collection_cmd(pool: State<'_, Arc<SqlitePool>>, name: String) -> Result<String, String>
#[tauri::command]
pub async fn list_collections_cmd(pool: State<'_, Arc<SqlitePool>>) -> Result<Vec<Value>, String>
#[tauri::command]
pub async fn add_book_to_collection_cmd(pool: State<'_, Arc<SqlitePool>>, collection_id: String, book_id: String) -> Result<(), String>
#[tauri::command]
pub async fn remove_book_from_collection_cmd(pool: State<'_, Arc<SqlitePool>>, collection_id: String, book_id: String) -> Result<(), String>
#[tauri::command]
pub async fn get_collection_books_cmd(pool: State<'_, Arc<SqlitePool>>, collection_id: String) -> Result<Vec<Value>, String>
#[tauri::command]
pub async fn bulk_reingest(pool: State<'_, Arc<SqlitePool>>, book_ids: Vec<String>) -> Result<(), String>
#[tauri::command]
pub async fn bulk_delete(pool: State<'_, Arc<SqlitePool>>, book_ids: Vec<String>) -> Result<(), String>
#[tauri::command]
pub async fn export_metadata(pool: State<'_, Arc<SqlitePool>>, book_ids: Vec<String>) -> Result<String, String>
#[tauri::command]
pub async fn write_file(path: String, contents: String) -> Result<(), String>
#[tauri::command]
pub async fn import_calibre(pool: State<'_, Arc<SqlitePool>>, library_path: String) -> Result<Value, String>
#[tauri::command]
pub async fn update_position(pool: State<'_, Arc<SqlitePool>>, book_id: String, position: String) -> Result<(), String>
```

The pool is initialised once in `main.rs` `setup()`, wrapped in `Arc`, and
managed via `app.manage()`. All commands access it via `pool.inner().as_ref()`.

---

## Frontend Architecture

**Framework:** React 18 + TypeScript, built with Vite 5, styled with Tailwind CSS 3.

**State:** Zustand stores
- `libraryStore` — book list and filtered/search results fetched via Tauri invocations; no local persistence
- `settingsStore` — font size, theme, and font family; persisted to `localStorage` via Zustand `persist` middleware

**Tauri integration:** `@tauri-apps/api/core` `invoke()` for all backend calls.
No direct DB access from the frontend.

**Library controls:** `SearchBar.tsx` and `FilterBar.tsx` drive search and
metadata filtering through Tauri commands. `CollectionsSidebar.tsx` switches
between all books and named collections, `BulkActionBar.tsx` handles
re-ingest/delete/export, and the top bar exposes Calibre import. Phase 6 adds
`SettingsModal.tsx`, `ErrorBoundary.tsx`, `Skeleton.tsx`, and the keyboard /
updater hooks used by the app shell and reader.

**EPUB rendering:** Custom `epub://` URI scheme registered in Tauri serves
ZIP entries from the book's format file directly to the WebView.
CFI-based position tracking is implemented in `ui/src/reader/cfi.ts`.

---

## Key Constraints (Non-Negotiable)

| Constraint | Enforcement |
|---|---|
| No `unwrap()` in library code | clippy `-D warnings` in CI |
| `cargo clippy --workspace -- -D warnings` must pass at zero warnings | CI gate |
| All `.map_err` use function-pointer form, not closure | clippy `redundant_closure` |
| Range checks use `.contains()` not `a >= x && a <= y` | clippy `manual_range_contains` |
| No `read_exact` on potentially-short files | use `read()`, check returned byte count |
| Service token never in config files or logs | keyring crate only |
| No secrets hardcoded | config.toml + env var overrides only |
| Magic-byte-first format detection | never rely on file extension alone |
| All enrichment calls: 10s timeout, silent fallback | never block processing |
| All xcalibre-server read calls: 5s timeout, empty/None on failure | never surface read errors |
| Offline-first: fully functional with no xcalibre-server connection | push stage skips if no client |
| LLM suggestions never auto-applied | user always confirms |

---

## Dependency Map

| Crate | Version | Purpose |
|---|---|---|
| tokio | 1 | Async runtime |
| sqlx | 0.8 | SQLite async queries + migrations |
| clap | 4 (derive) | CLI argument parsing |
| thiserror | 1 | ProcessingError derive |
| anyhow | 1 | Error type for main() and tests |
| serde / serde_json | 1 | Serialisation |
| uuid | 1 (v4) | Job ID generation |
| chrono | 0.4 | ISO 8601 datetime handling |
| sha2 | 0.10 | SHA-256 hashing |
| zip | 0.6 | EPUB/CBZ ZIP parsing |
| roxmltree | 0.20 | XML parsing (OPF, container.xml) |
| regex | 1 | HTML tag stripping for text extraction |
| image | 0.25 | Cover image resize (JPEG/PNG) |
| reqwest | 0.12 (rustls-tls) | HTTP client for API calls |
| keyring | 2 | OS keychain token storage |
| tracing / tracing-subscriber | 0.1 / 0.3 | Structured logging |
| tauri | 2 | Desktop shell |
| dirs | 5 | Platform app data paths |
| toml | 0.8 | Config file parsing |
| tempfile | 3 (dev) | Temporary files in tests |

---

## Testing Strategy

Integration tests live in `processing/tests/`. Each test file gets its own
in-memory SQLite pool with fresh migrations:

```rust
async fn setup_db() -> Pool<Sqlite> {
    let pool = SqlitePoolOptions::new().connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
    pool
}
```

Test fixtures in `processing/tests/fixtures/` are generated by
`processing/src/bin/gen_fixtures.rs`. The EPUB fixture is a structurally
complete ZIP (mimetype + container.xml + OPF + chapter XHTML) so that
metadata and text extraction tests have real content to parse.

Run all tests: `cargo test --workspace`
Run with filter: `cargo test --workspace -- <name>`
