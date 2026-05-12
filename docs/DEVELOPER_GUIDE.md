# xcalibre Developer Guide

## Table of Contents

1. [Project Structure](#1-project-structure)
2. [Architecture Overview](#2-architecture-overview)
3. [Building and Running](#3-building-and-running)
4. [Database Schema](#4-database-schema)
5. [Tauri Command Layer](#5-tauri-command-layer)
6. [Processing Pipeline](#6-processing-pipeline)
7. [Format Detection](#7-format-detection)
8. [Full-Text Search](#8-full-text-search)
9. [AI Subsystem](#9-ai-subsystem)
10. [Plugin Development](#10-plugin-development)
11. [Testing](#11-testing)
12. [Adding a New Format](#12-adding-a-new-format)
13. [Adding a New AI Provider](#13-adding-a-new-ai-provider)
14. [Security Model](#14-security-model)
15. [Error Handling Conventions](#15-error-handling-conventions)

---

## 1. Project Structure

```
xcalibre/
├── src-tauri/               # Tauri 2.x host process (Rust)
│   └── src/
│       ├── main.rs          # App setup, DB init, command registration
│       ├── commands.rs      # All #[tauri::command] handlers (IPC surface)
│       ├── epub_protocol.rs # Custom xcalibre:// protocol for EPUB serving
│       └── spellcheck.rs    # OS spell-check bridge
│
├── processing/              # Core library crate (no Tauri dependency)
│   └── src/
│       ├── pipeline/        # Ingest, conversion, sync pipelines
│       ├── db/              # All SQLite queries (one file per table group)
│       ├── plugins/         # Plugin loader + DetectedFormat enum
│       ├── metadata/        # Format-specific metadata extractors
│       ├── convert/         # Format converters (epub → txt/pdf/mobi/…)
│       ├── cover/           # Cover extraction and resizing
│       ├── editor/          # EPUB editor (in-place ZIP modification)
│       ├── backup/          # Library backup/restore
│       └── search/          # Virtual library query parser
│
├── xcalibre-ai/             # AI provider abstraction crate
│   └── src/
│       ├── provider.rs      # AiProvider trait
│       ├── factory.rs       # make_provider() constructor
│       ├── chunk.rs         # Text chunking for RAG
│       ├── citations.rs     # Citation extraction from model output
│       ├── reasoning.rs     # Reasoning budget application
│       └── {ollama,openai,gemini,lmstudio,openrouter}.rs
│
├── xcalibre-epub/           # EPUB-specific library crate
│   └── src/
│       ├── container.rs     # ZIP read/write abstraction
│       ├── opf.rs           # OPF/NCX metadata parsing
│       ├── css.rs           # CSS injection and theming
│       ├── font.rs          # Font embedding
│       ├── cover.rs         # Cover image operations
│       └── split.rs         # Chapter splitting
│
├── xcalibre-plugin-sdk/     # Plugin ABI crate (published separately)
│   └── src/lib.rs           # PLUGIN_API_VERSION, vtables, PluginMetadata
│
├── api/                     # xcalibre-server API client
│   └── src/
│       ├── client.rs        # HTTP client with keyring auth
│       ├── push.rs          # Book push to server
│       └── enrichment/      # Open Library + Google Books metadata fetchers
│
└── ui/                      # React + TypeScript frontend
    └── src/
        ├── components/      # React components
        ├── stores/          # Zustand state stores
        └── hooks/           # Custom React hooks
```

---

## 2. Architecture Overview

```
┌──────────────────────────────────────────────────────────────┐
│                    WebView (React/TypeScript)                 │
│  Zustand stores ← components → invoke("command", args)       │
└──────────────────────────┬───────────────────────────────────┘
                           │  Tauri IPC (JSON over message passing)
┌──────────────────────────▼───────────────────────────────────┐
│               src-tauri/src/commands.rs                       │
│  #[tauri::command] handlers — validate + delegate             │
└───┬──────────────┬───────────────┬────────────────────────────┘
    │              │               │
    ▼              ▼               ▼
processing/    xcalibre-ai/    xcalibre-epub/
(pipeline,     (providers,     (container,
 db queries,    chunking,       opf, css,
 formats)       citations)      fonts)
    │
    ▼
SQLite (sqlx, async)   +   Filesystem (ebook files, covers)
```

### Key invariants

- **No business logic in commands.rs.** Commands validate types, delegate to
  processing/xcalibre-ai, and map `ProcessingError → String`. Everything else
  lives in the crates.
- **The WebView never sees raw API keys.** `AiConfigPublic` replaces `api_key`
  with `has_api_key: bool`. See `commands.rs:AiConfigPublic`.
- **SQLite foreign keys are ON.** Set via `PRAGMA foreign_keys = ON` in
  `main.rs` using `SqliteConnectOptions::pragma`. Every connection inherits this.
- **All bulk mutations are transactional.** `bulk_delete_books` and
  `update_book_details` open `pool.begin()` / `tx.commit()` so partial failures
  leave no orphaned data.

---

## 3. Building and Running

### Prerequisites

- Rust 1.78+ (`rustup update`)
- Node.js 20+ and `npm`
- Tauri CLI 2.x: `cargo install tauri-cli --version "^2"`

### Development

```bash
cd xcalibre/ui && npm install
cd .. && cargo tauri dev
```

### Production build

```bash
cargo tauri build
```

### Run processing crate tests only

```bash
cd processing && cargo test
```

### Run AI crate tests

```bash
cd xcalibre-ai && cargo test
```

### Linting

```bash
cargo clippy --workspace -- -D warnings
```

---

## 4. Database Schema

The SQLite database lives at `{app_data}/xcalibre/library.db`. All migrations
are in `processing/src/db/migrations/` and run automatically on startup via
`sqlx::migrate!()` in `main.rs`.

### Core tables

| Table | Purpose |
|-------|---------|
| `jobs` | One row per imported file. Tracks ingest status and file path. |
| `local_books` | Enriched book record. Created after successful metadata extraction. |
| `book_tags` | Many-to-many join between `local_books` and `tags`. |
| `identifiers` | ISBN, ASIN, Goodreads IDs. Multiple per book. |
| `books_fts` | FTS5 virtual table for full-text search. |
| `book_chunks` | Text chunks for RAG AI queries. |
| `annotations` | Highlights and notes with CFI position. |
| `ai_config` | Per-library AI provider settings. |
| `installed_plugins` | Registered plugin dylibs. |
| `book_formats` | Multiple file formats for the same book (e.g. EPUB + PDF). |
| `collections` / `collection_books` | User-created book groupings. |
| `reading_sessions` | Reading session statistics. |
| `libraries` | Multi-library metadata. |
| `virtual_libraries` | Saved search queries as named virtual libraries. |

### Key relationships

```
jobs (id) ──── local_books (id)
                   │
                   ├── book_tags (book_id) → tags (id)
                   ├── identifiers (book_id)
                   ├── annotations (book_id)
                   ├── book_chunks (book_id)
                   └── collection_books (book_id) → collections (id)
```

### Adding a migration

Create `processing/src/db/migrations/XXXX_description.sql` where `XXXX` is the
next sequential number. `sqlx::migrate!()` runs all un-applied migrations on
startup in filename order.

---

## 5. Tauri Command Layer

All TypeScript-to-Rust calls go through `src-tauri/src/commands.rs`. Commands
are registered in `main.rs`:

```rust
.invoke_handler(tauri::generate_handler![
    list_books,
    ingest_file,
    // … all commands …
])
```

### TypeScript invocation

```typescript
import { invoke } from "@tauri-apps/api/core"

// Without arguments
const books = await invoke<BookRow[]>("list_books")

// With arguments (camelCase keys map to snake_case Rust fields via #[serde(rename_all = "camelCase")])
await invoke("update_book_details", { bookIds: ["abc"], details: { title: "Dune", ... } })
```

### Shared state

`tauri::State<'_, Arc<SqlitePool>>` is injected into every command that needs
the database. The pool is created once in `main.rs` with a connection-level
`PRAGMA foreign_keys = ON`.

### Error convention

Every command returns `Result<T, String>`. The `String` is the `.to_string()`
of the underlying `ProcessingError`. TypeScript receives it as a rejection reason
and displays it in an error toast.

### Adding a new command

1. Write the function in `commands.rs` with `#[tauri::command]`.
2. Add it to the `generate_handler![...]` list in `main.rs`.
3. Call it from TypeScript via `invoke("function_name", args)`.

---

## 6. Processing Pipeline

### Ingest (`processing/src/pipeline/ingest.rs`)

The ingest pipeline is triggered by `ingest_file` → `import_local_book` →
`run_ingest`. Steps:

1. **Format detection** — `detect_format()` reads magic bytes.
2. **Integrity validation** — `validate_integrity()` checks structural soundness.
3. **SHA-256 hashing** — computed on the blocking thread pool.
4. **Duplicate check** — queries `jobs` for matching `file_sha256`.
5. **Job creation** — inserts `status = 'PENDING'` row.

After `run_ingest`, the pipeline asynchronously:
- Extracts metadata → `job_metadata`
- Extracts full text → `job_text`
- Extracts cover → filesystem + `local_books.cover_path`
- Creates FTS index entry → `books_fts`
- Creates `local_books` row

### Conversion (`processing/src/convert/`)

Each converter is a standalone file (`epub_to_txt`, `epub_to_pdf`, etc.).
The `convert_book` Tauri command dispatches to the right module based on the
`OutputFormat` enum.

Conversions always start from EPUB. If the source is not EPUB, the pipeline
first converts to EPUB (`convert_book_to_epub`), then converts to the target.

### Sync (`processing/src/pipeline/sync.rs`)

`sync_annotations` pushes unsynced annotations to xcalibre-server using the
API client from the `api` crate. Requires a stored token in the OS keychain
(`xcalibre` / `xs_token`).

---

## 7. Format Detection

**Source:** `processing/src/pipeline/ingest.rs` → `detect_format()`

Detection reads the first 4096 bytes and applies checks in this order:

| Priority | Check | Formats |
|----------|-------|---------|
| 1 | `PK\x03\x04` ZIP magic | EPUB, DOCX, ODT, HTMLZ, CBZ |
| 2 | `%PDF-` header | PDF |
| 3 | `BOOKMOBI` at bytes 60–68 | MOBI, AZW3, AZW4 |
| 4 | `{\rtf` | RTF |
| 5 | XML + `FictionBook` string | FB2 |
| 6 | `<html`/`<!DOCTYPE` | HTML |
| 7 | `AT&TFORM` | DjVu |
| 8 | `ITSF` | CHM |
| 9 | `ITOLITLS` | LIT |
| 10 | SQLite magic + `fragments` table | KFX |
| 11 | `SNBP` | SNB |
| 12 | `L\0R\0F\0` (UTF-16LE) | LRF |
| 13 | RAR v4/v5 magic | CBR |
| 14 | Extension fallback | PDB, PML, RB, TCR, LRX, AZW3, AZW4 |
| 15 | ≥50% printable bytes | TXT |
| 16 | — | `UnsupportedFormat` error |

ZIP disambiguation probes open the archive a second time to check for specific
inner entries (`mimetype`, `word/document.xml`, `content.xml`, `index.html`).

---

## 8. Full-Text Search

**Source:** `processing/src/db/fts_queries.rs`

xcalibre uses SQLite FTS5 with four indexed columns:

```sql
CREATE VIRTUAL TABLE books_fts USING fts5(
    book_id UNINDEXED,
    title,
    authors,
    description,
    full_text
);
```

### Query syntax (forwarded directly to FTS5 MATCH)

| Syntax | Meaning |
|--------|---------|
| `dune` | Match "dune" in any column |
| `title:dune` | Match only in title |
| `authors:herbert` | Match only in authors |
| `dune herbert` | AND (both words must appear) |
| `dune OR tolkien` | OR |
| `"lord of the rings"` | Phrase search |
| `du*` | Prefix search |

### Malformed query handling

Invalid MATCH expressions (empty query, unclosed quote, bare `title:`) cause
SQLite to return a `Database` error. `fts_queries::search()` catches errors
whose message contains `"fts5:"` or `"malformed MATCH"` and returns an empty
result set instead of propagating the error. This prevents routine user typos
from showing error toasts.

See: `processing/src/db/fts_queries.rs:search()`

### Index maintenance

The FTS index is **not** maintained by triggers. It must be updated manually:
- After ingest: `upsert_fts()` is called from the ingest pipeline.
- After metadata edit: `refresh_book_index()` is called from `update_book_details`.

---

## 9. AI Subsystem

**Source:** `xcalibre-ai/`, `processing/src/db/ai_queries.rs`,
`src-tauri/src/commands.rs` (AI section)

### Provider architecture

```
AiProvider trait (xcalibre-ai/src/provider.rs)
    ├── OllamaBackend      (local, no key required)
    ├── LmStudioBackend    (local, no key required)
    ├── OpenAiBackend      (remote, API key required)
    ├── GeminiBackend      (remote, API key required)
    └── OpenRouterBackend  (remote, API key required)
```

Providers are constructed via `make_provider(cfg)` in `xcalibre-ai/src/factory.rs`.

### RAG pipeline

1. **Chunking** — `xcalibre_ai::chunk_text()` splits book text into overlapping
   ~512-token segments respecting sentence boundaries.
2. **Storage** — chunks stored in `book_chunks(book_id, chunk_index, chunk_text, embedding)`.
3. **Retrieval** — `get_ai_context_chunks` command scores chunks against the
   query using word-level exact matching (`HashSet<&str>`) and returns the top 5.
4. **Context injection** — the top chunks are concatenated and injected into the
   system prompt before the user message.

### Reasoning budgets

Defined in `xcalibre-ai/src/reasoning.rs`. The budget controls whether an
additional `<think>` system instruction is prepended and whether the model's
thinking token budget is set. Values: `None`, `Low`, `Medium`, `High`.

### API key security

- API keys are stored in `ai_config.api_key` (plaintext SQLite).
- The `get_ai_config_cmd` command returns `AiConfigPublic` with `has_api_key: bool` —
  the raw key is never sent to the WebView.
- The UI shows `"••••••••"` when `has_api_key` is true and sends `null` (not
  the masked string) when saving unchanged settings.
- The `COALESCE(excluded.api_key, ai_config.api_key)` in the upsert SQL
  preserves the stored key when `null` is received.

---

## 10. Plugin Development

This section explains how to create, package, and install a plugin for xcalibre.

### Plugin types

| Type | String | What it does |
|------|--------|-------------|
| `MetadataSource` | `"metadata_source"` | Search external databases for book metadata |
| `ConversionOutput` | `"conversion_output"` | Convert EPUB bytes to a target format |
| `Store` | `"store"` | (Reserved for future use) |

### ABI contract

Plugins are native shared libraries (`.dylib` on macOS, `.so` on Linux,
`.dll` on Windows). The ABI is defined in `xcalibre-plugin-sdk/src/lib.rs`.

**Every plugin must export these two symbols:**

```c
// Returns the PLUGIN_API_VERSION the plugin was compiled against.
// xcalibre loads this and rejects if it doesn't match.
uint32_t xcalibre_plugin_api_version(void);

// Returns a pointer to a static, null-terminated UTF-8 JSON string
// containing a serialised PluginMetadata struct.
const char *xcalibre_plugin_metadata(void);
```

**Additionally, each plugin type must export its vtable:**

```c
// MetadataSource plugins:
const MetadataSourceVtable *xcalibre_metadata_source_vtable(void);

// ConversionOutput plugins:
const ConversionOutputVtable *xcalibre_conversion_output_vtable(void);
```

The vtable structs are `#[repr(C)]` and defined in the SDK:

```rust
// xcalibre-plugin-sdk/src/lib.rs

#[repr(C)]
pub struct MetadataSourceVtable {
    pub search:   extern "C" fn(query: *const c_char) -> *mut c_char,
    pub free_str: extern "C" fn(ptr: *mut c_char),
}

#[repr(C)]
pub struct ConversionOutputVtable {
    pub convert:  extern "C" fn(
        epub_data: *const u8, epub_len: usize,
        out_buf: *mut *mut u8, out_len: *mut usize,
    ) -> i32,
    pub free_buf: extern "C" fn(ptr: *mut u8, len: usize),
}
```

### Writing a MetadataSource plugin in Rust

```rust
// In your plugin crate, add to Cargo.toml:
// xcalibre-plugin-sdk = { path = "../xcalibre-plugin-sdk" }

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use xcalibre_plugin_sdk::{PLUGIN_API_VERSION, MetadataSourceVtable};

// ── Required exports ────────────────────────────────────────

#[no_mangle]
pub extern "C" fn xcalibre_plugin_api_version() -> u32 {
    PLUGIN_API_VERSION
}

#[no_mangle]
pub extern "C" fn xcalibre_plugin_metadata() -> *const c_char {
    // Must be a static string — xcalibre does NOT call free on this pointer.
    static META: &[u8] = b"{\
        \"name\":\"open-library\",\
        \"version\":\"1.0.0\",\
        \"api_version\":1,\
        \"plugin_type\":\"MetadataSource\"\
    }\0";
    META.as_ptr() as *const c_char
}

// ── MetadataSource vtable ───────────────────────────────────

extern "C" fn search(query: *const c_char) -> *mut c_char {
    // SAFETY: xcalibre guarantees query is a valid null-terminated UTF-8 string.
    let q = unsafe { CStr::from_ptr(query) }.to_string_lossy();

    // Perform your metadata lookup here.
    let results = fetch_from_open_library(&q);

    // Serialise to JSON and return a heap-allocated C string.
    let json = serde_json::to_string(&results).unwrap_or_else(|_| "[]".into());
    // CString::into_raw gives xcalibre ownership. xcalibre calls free_str when done.
    CString::new(json).unwrap().into_raw()
}

extern "C" fn free_str(ptr: *mut c_char) {
    if ptr.is_null() { return; }
    // SAFETY: ptr was returned by CString::into_raw in the same process.
    unsafe { drop(CString::from_raw(ptr)) };
}

#[no_mangle]
pub extern "C" fn xcalibre_metadata_source_vtable() -> *const MetadataSourceVtable {
    static VTABLE: MetadataSourceVtable = MetadataSourceVtable {
        search,
        free_str,
    };
    &VTABLE
}
```

### The `plugin.json` manifest

Every plugin ZIP must contain a `plugin.json` at the root:

```json
{
  "name": "open-library",
  "version": "1.0.0",
  "api_version": 1,
  "plugin_type": "MetadataSource"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Human-readable name, also used as the extraction subdirectory |
| `version` | string | SemVer string shown in the Plugin Manager |
| `api_version` | integer | Must equal `PLUGIN_API_VERSION` (currently `1`) |
| `plugin_type` | string | `"MetadataSource"`, `"ConversionOutput"`, or `"Store"` |

### Building and packaging

```bash
# Build as a dynamic library
cargo build --release

# Package into a ZIP
zip open-library-1.0.zip \
    target/release/libopen_library.dylib \
    plugin.json
```

The ZIP may contain any directory structure; xcalibre extracts all files
preserving relative paths.

### Installing a plugin

1. Open xcalibre → menu bar → Plugins (or `Cmd+Shift+P`).
2. Click **"+ Install plugin from ZIP…"**.
3. Select your `.zip` file.

xcalibre will:
1. Open the ZIP and parse `plugin.json`.
2. Reject if `api_version != PLUGIN_API_VERSION` with an error toast.
3. Extract all files to `{app_data}/plugins/{name}/`.
4. Register the plugin in `installed_plugins`.

### Plugin installation flow (code path)

```
User selects ZIP in PluginManagerModal.tsx
  → invoke("install_plugin_from_zip", { zipPath })
    → src-tauri/src/commands.rs::install_plugin_from_zip()
      → processing::plugins::loader::install_plugin_zip()
          ↳ parse plugin.json → PluginMetadata
          ↳ version guard: manifest.api_version == PLUGIN_API_VERSION
          ↳ extract all ZIP entries to {app_data}/plugins/
      → processing::db::plugin_queries::install_plugin()
          ↳ INSERT INTO installed_plugins (…)
  → PluginManagerModal calls reload() to refresh the list
```

Source files involved:

| File | Role |
|------|------|
| `xcalibre-plugin-sdk/src/lib.rs` | ABI types, `PLUGIN_API_VERSION`, vtables |
| `processing/src/plugins/loader.rs` | ZIP extraction and version guard |
| `processing/src/db/plugin_queries.rs` | Database CRUD for `installed_plugins` |
| `src-tauri/src/commands.rs` | Tauri command wrappers (plugin section) |
| `ui/src/components/PluginManagerModal.tsx` | React UI |
| `processing/tests/test_plugin_loader.rs` | Loader tests |
| `processing/tests/test_plugin_sdk.rs` | SDK type tests |
| `processing/tests/test_plugin_db.rs` | Database query tests |

### Plugin runtime loading (vtable resolution)

> **Note:** Dynamic vtable loading at runtime is not yet wired into the main
> application flow. The infrastructure (DB table, install/uninstall commands,
> enable/disable toggle) is complete. The next step is implementing a
> `PluginRegistry` struct that:
>
> 1. Reads all enabled plugins from `installed_plugins`.
> 2. Calls `dlopen` (macOS/Linux) or `LoadLibrary` (Windows) on each `dylib_path`.
> 3. Resolves `xcalibre_plugin_api_version` and `xcalibre_plugin_metadata` symbols.
> 4. Resolves the type-specific vtable symbol.
> 5. Stores the vtable pointer for the duration of the app session.
>
> This will be implemented in `processing/src/plugins/runtime.rs`.

### Versioning policy

`PLUGIN_API_VERSION` is an integer in `xcalibre-plugin-sdk/src/lib.rs`. It must
be incremented whenever:

- A vtable field is added, removed, or reordered.
- A vtable function signature changes (parameter types or return type).
- The JSON schema of types exchanged across the boundary changes.

It does **not** need to be incremented for:
- Changes to xcalibre internals that don't touch the FFI boundary.
- Additions to `PluginType` (additive).

When the version increments, existing installed plugins will be rejected at
load time until they are recompiled against the new SDK.

---

## 11. Testing

### Unit / integration tests

```bash
# All crate tests
cargo test --workspace

# Just the plugin tests
cargo test -p xcalibre-processing test_plugin

# Specific test file
cargo test -p xcalibre-processing --test test_plugin_db
```

### Frontend tests (Vitest)

```bash
cd ui && npm test
```

### Test fixtures

Magic-byte fixtures are generated by `processing/src/bin/gen_fixtures.rs`:

```bash
cargo run -p xcalibre-processing --bin gen_fixtures
```

This creates files like `tests/fixtures/fixture_epub.epub`,
`fixture_pdf.pdf`, etc., used by the format detection tests.

### Plugin test fixtures

`tests/fixtures/plugin_bad_version.zip` is referenced by `test_plugin_loader.rs`.
It must be created manually (or by a fixture generator not yet written) — the
test skips if the file is absent.

---

## 12. Adding a New Format

1. **Add a variant** to `DetectedFormat` in `processing/src/plugins/mod.rs`
   with a doc comment explaining the magic bytes.

2. **Add magic-byte detection** in `processing/src/pipeline/ingest.rs::detect_format()`.
   Add a comment explaining the byte signature.

3. **Add integrity validation** in `validate_integrity()` in the same file, or
   fall through to the no-op arm if there is no useful structural check.

4. **Add a metadata extractor** in `processing/src/metadata/{format}.rs`.

5. **Add a text extractor** if the format supports it.

6. **Add a cover extractor** in `processing/src/cover/{format}.rs` if applicable.

7. **Add a converter** in `processing/src/convert/{format}.rs`:
   ```rust
   pub fn epub_to_{format}(epub: &Path, out: &Path) -> Result<(), ProcessingError> { … }
   ```

8. **Wire up the converter** in `src-tauri/src/commands.rs::convert_book()`:
   - Add a variant to `OutputFormat`.
   - Add an arm to the path/match blocks.

9. **Add to the UI** in `ui/src/components/ConversionDialog.tsx`.

10. **Write tests** in `processing/tests/`.

---

## 13. Adding a New AI Provider

1. **Create the backend** in `xcalibre-ai/src/{provider}.rs` implementing
   the `AiProvider` trait from `xcalibre-ai/src/provider.rs`.

2. **Register it** in `xcalibre-ai/src/lib.rs`:
   ```rust
   pub mod {provider};
   ```

3. **Add a match arm** in `xcalibre-ai/src/factory.rs::make_provider()`:
   ```rust
   "{provider}" => Ok(Box::new(YourBackend::new(&cfg.base_url, key, &cfg.model, &cfg.embed_model))),
   ```

4. **Add to the UI** in `ui/src/components/AIProviderSettingsPanel.tsx`:
   ```typescript
   const PROVIDERS = [
     // … existing …
     { value: "{provider}", label: "Your Provider", needsKey: true, defaultUrl: "https://…" },
   ]
   ```

5. **Write tests** in `xcalibre-ai/tests/test_{provider}_provider.rs`.

---

## 14. Security Model

### Path sandbox (`write_file`)

The `write_file` command in `commands.rs` enforces that all writes stay inside
`app_data_dir()`. It uses `Path::canonicalize()` to resolve symlinks and `..`
components before the check, so traversal attempts are caught.

**Source:** `src-tauri/src/commands.rs:write_file()`

### Zip-slip prevention

ZIP entry names in plugin installation and comic page extraction are sanitised
with `entry.name().file_name()` to strip directory components. An entry named
`../../evil` is reduced to `evil`.

**Source:** `processing/src/plugins/loader.rs:install_plugin_zip()`
and `src-tauri/src/commands.rs:list_comic_pages()`

### API key protection

The raw API key is never sent to the WebView. `AiConfigPublic.has_api_key: bool`
is the only thing the frontend sees.

**Source:** `src-tauri/src/commands.rs:get_ai_config_cmd()`

### Plugin version guard

Plugins whose `api_version != PLUGIN_API_VERSION` are rejected before any files
are extracted to disk.

**Source:** `processing/src/plugins/loader.rs:install_plugin_zip()`

### SQLite foreign keys

`PRAGMA foreign_keys = ON` is set at the connection level in `main.rs`. This
ensures cascade deletes work correctly and orphaned rows are never created.

---

## 15. Error Handling Conventions

### In `xcalibre-processing`

All public functions return `Result<T, ProcessingError>`. `ProcessingError`
variants are defined in `processing/src/error.rs`. New error conditions should
use existing variants where possible, or add a new variant with a doc comment.

### In Tauri commands

Commands return `Result<T, String>`. The `String` is produced by
`.map_err(|e| e.to_string())`. Never `unwrap()` in command handlers — always
propagate with `?` or `.map_err`.

### FTS query errors

FTS5 syntax errors are silently converted to empty results (see
`processing/src/db/fts_queries.rs:search()`). This is intentional — invalid
search queries are a user mistake, not a system error.

### Best-effort operations

Operations that clean up external resources (deleting cover files, deleting
dylib files after uninstall) use `let _ = std::fs::remove_file(...)` to
suppress errors. Failures are logged at `warn` level, not propagated.
