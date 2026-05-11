# xCalibre — Feature Inventory

Generated: 2026-05-11 from source folder analysis.

This document enumerates every discrete feature, module, and major functional
area in the xCalibre codebase. Each item records its location and purpose, with
no assessment of completeness — that lives in `GAP.md`.

---

## 1. Crate Architecture

| Crate | Location | Purpose |
|-------|----------|---------|
| `xcalibre-processing` | `processing/` | Core library + CLI binary. All business logic. |
| `xcalibre-api` | `api/` | HTTP client for xcalibre-server remote API |
| `app` (src-tauri) | `src-tauri/` | Tauri v2 desktop shell. UI bridging. |
| `ui` | `ui/` | React 18 + TypeScript + Tailwind frontend |

---

## 2. CLI Entry Point

**File:** `processing/src/main.rs`
- Subcommand: `ingest <FILE>` — run full local import pipeline
- Subcommand: `auth set-token|status|remove-token` — manage xcalibre-server keychain token
- Subcommand: `sync` — pull remote library, backpropagate status, retry failed pushes

---

## 3. Config Module

**File:** `processing/src/config.rs`
- `Config` struct: `xs_url`, `db_path`, `cover_dir`
- Loads from `~/Library/Application Support/xcalibre/config.toml` (macOS)
- Environment var overrides: `XCALIBRE_SERVER_URL`, `XCALIBRE_DB_PATH`
- Auto-creates default config on first load

---

## 4. Error Types

**File:** `processing/src/error.rs`
- `ProcessingError` enum (thiserror):
  - `IoError` — file I/O failures
  - `DbError` — SQLite query failures
  - `UnsupportedFormat` — unknown or unreadable file
  - `IntegrityError` — format structural validation failure
  - `MetadataError` — metadata extraction failure
  - `TextError` — text extraction failure
  - `CoverError` — cover extraction failure
  - `Duplicate` — SHA-256 collision (already imported)

---

## 5. Processing Pipeline

**Location:** `processing/src/pipeline/`

### 5.1 Stage 1 — Ingest
**File:** `pipeline/ingest.rs`
- `detect_format()` — magic-byte-first format detection (reads up to 4 KiB)
- `validate_integrity()` — per-format structural check (ZIP, PDF header/EOF, MOBI header, etc.)
- `run_ingest()` — full stage: validate → detect → SHA-256 → dedup check → insert job row

### 5.2 Stage 2 — Metadata Extraction
**File:** `pipeline/metadata.rs` + `metadata/`
- `run_metadata()` — dispatches to format-specific extractor, serializes `BookMetadata` as JSON, upserts `job_metadata`

### 5.3 Stage 3 — Enrichment
**File:** `pipeline/enrichment.rs`
- `run_enrichment()` — ISBN lookup → Open Library + Google Books (concurrent, 10s timeout each)
- Result stored in `enrichment_cache` (30-day TTL)
- Suggestions built; user always confirms before application

### 5.4 Stage 4 — Text Extraction
**File:** `pipeline/text.rs` + `text/`
- `run_text()` — dispatches to format-specific text extractor, stores in `job_text`

### 5.5 Stage 5 — Cover Extraction
**File:** `pipeline/cover.rs` + `cover/`
- `run_cover()` — extract cover image, resize to max 500×750 JPEG, write to disk

### 5.6 Stage 6 — Push to Server
**File:** `pipeline/push.rs`
- `run_push()` — push assembled payload to xcalibre-server
- On success: `COMPLETED` + `xs_book_id`
- On failure: exponential backoff (`5 × retry_count` minutes), status `RETRYING`
- After 5 failures: `FAILED`

### 5.7 Background Sync
**File:** `pipeline/sync.rs`
- `sync_pull()` — paginated pull of remote books into `local_books`
- `sync_status_backprop()` — detects remotely deleted books and re-queues local jobs
- `sync_annotations()` — pushes unsynced annotations to xcalibre-server

### 5.8 Retry Worker
**File:** `pipeline/retry.rs`
- `start_retry_worker()` — 60-second interval background retry loop
- `run_retries()` — picks up `RETRYING` jobs with elapsed `next_retry_at`
- Exponential backoff: `2^retry_count` minutes, capped at 60 min, max 5 retries

### 5.9 Full Pipeline Runner
**File:** `pipeline/local.rs`
- `import_local_book()` — runs stages 1–5 + FTS index refresh sequentially
- Best-effort: each stage can fail independently, pipeline continues
- Produces SVG placeholder cover when no image cover is found

### 5.10 Conversion Pipeline
**File:** `pipeline/conversion.rs`
- `convert_book_to_epub()` — repackages any supported format into EPUB
- Two modes: `CONVERT` (full repackage) and `TWEAK` (in-place EPUB tweak)
- Extracts text from any supported format, builds valid EPUB 3 package (mimetype, container.xml, OPF, XHTML)
- Persists conversion job in `conversion_jobs` table

---

## 6. Format Support

**Location:** `processing/src/plugins/mod.rs`
- `DetectedFormat` enum: 25 formats
  - Primary: `Epub`, `Pdf`, `Mobi`, `Azw3`, `Azw4`, `Cbz`, `Cbr`, `Txt`
  - Extended: `Fb2`, `Html`, `Htmlz`, `Rtf`, `Docx`, `Odt`, `Chm`
  - Legacy: `Lrf`, `Lrx`, `Pdb`, `Pml`, `Rb`, `Snb`, `Tcr`
  - Special: `Djvu`, `Lit`
- Each format has: `extension()`, `display_name()`, `is_image_only()`

---

## 7. Format-Specific Metadata Extractors

**Location:** `processing/src/metadata/`

| File | Format(s) | Capability |
|------|-----------|------------|
| `epub.rs` | EPUB | OPF-based metadata |
| `pdf.rs` | PDF | Info dict + XMP metadata |
| `mobi.rs` | MOBI/AZW3 | PalmDOC header + EXTH records |
| `fb2.rs` | FB2 | FictionBook XML parsing |
| `html.rs` | HTML/HTMLZ | `<meta>` tag extraction |
| `rtf.rs` | RTF | Info dictionary |
| `docx.rs` | DOCX | OOXML core/application XML |
| `odt.rs` | ODT | OpenDocument meta.xml |
| `chm.rs` | CHM | Heuristic recovery |
| `lrf.rs` | LRF/LRX | Heuristic recovery |
| `pdb.rs` | PDB/PML/RB | Heuristic recovery |
| `snb.rs` | SNB | Heuristic recovery |
| `tcr.rs` | TCR | Heuristic recovery |
| `azw4.rs` | AZW4 | Heuristic recovery |
| `djvu.rs` | DJVU | Heuristic recovery |
| `lit.rs` | LIT | Heuristic recovery |
| `cbz.rs` | CBZ/CBR | Archive page count |
| `isbn.rs` | N/A (shared) | EPUB + PDF ISBN detection |
| `enrichment.rs` | N/A (shared) | `EnrichmentSuggestion` struct + `build_suggestions()` merge logic |

---

## 8. Format-Specific Text Extractors

**Location:** `processing/src/text/`

| File | Format(s) |
|------|-----------|
| `epub.rs` | EPUB (spine → HTML strip → plain text) |
| `pdf.rs` | PDF |
| `mobi.rs` | MOBI/AZW3 |
| `fb2.rs` | FB2 |
| `html.rs` | HTML/HTMLZ |
| `rtf.rs` | RTF |
| `docx.rs` | DOCX |
| `odt.rs` | ODT |
| `chm.rs` | CHM |
| `lrf.rs` | LRF/LRX |
| `pdb.rs` | PDB/PML/RB |
| `snb.rs` | SNB |
| `tcr.rs` | TCR |
| `azw4.rs` | AZW4 |
| `djvu.rs` | DJVU |
| `lit.rs` | LIT |
| `txt.rs` | TXT (plain read) |

---

## 9. Cover Extractors

**Location:** `processing/src/cover/`

| File | Format(s) |
|------|-----------|
| `epub.rs` | EPUB (OPF cover reference, embedded image) |
| `pdf.rs` | PDF (embedded cover, first-page render) |
| `cbz.rs` | CBZ/CBR (first image in archive) |
| `resize.rs` | N/A (shared) — resize to max 500×750 JPEG via `image` crate |

---

## 10. Database Layer

**Location:** `processing/src/db/`

### 10.1 Query Modules

| File | Purpose |
|------|---------|
| `queries.rs` | Core `jobs` + `local_books` CRUD, search, FTS integration |
| `extended_queries.rs` | Rich metadata: tags, identifiers, `BookDetails`, bulk replace |
| `enrichment_cache.rs` | ISBN → OL/GB JSON cache, 30-day TTL |
| `fts_queries.rs` | FTS5 search + per-book index refresh |
| `collection_queries.rs` | Collection CRUD + book membership |
| `annotation_queries.rs` | Annotation CRUD + sync tracking |
| `conversion_queries.rs` | `conversion_jobs` CRUD + status tracking |
| `format_queries.rs` | `book_formats` queries (multiple formats per book) |

### 10.2 Migrations (10 total)

| File | What it adds |
|------|-------------|
| `0001_jobs.sql` | `jobs`, `job_metadata`, `job_text`, `local_books`, `push_queue` |
| `0002_enrichment_cache.sql` | `enrichment_cache` |
| `0003_reading_position.sql` | `reading_cfi` + `reading_position` on `local_books` |
| `0004_bookmarks.sql` | `bookmarks` table |
| `0005_fts.sql` | `books_fts` FTS5 virtual table |
| `0006_fts_triggers.sql` | Auto-population triggers for `books_fts` |
| `0007_collections.sql` | `collections` + `collection_books` |
| `0008_library_management.sql` | `title_sort`, `author_sort`, `pubdate`, `description`, `publisher`, `series_name`, `series_index`, `rating`, `tags`, `book_tags`, `identifiers`, `book_formats` |
| `0009_annotations.sql` | `annotations` (highlights, notes, bookmarks with CFI + sync flag) |
| `0010_conversion_jobs.sql` | `conversion_jobs` (convert/tweak tracking) |

---

## 11. Utilities

**Location:** `processing/src/utils/`

| File | Purpose |
|------|---------|
| `hash.rs` | `sha256_file()` — streaming SHA-256 of file contents |
| `normalise.rs` | Text normalisation (unicode NFKD, whitespace collapse) |
| `sort.rs` | `title_sort()`, `author_sort()` — sort-key generation (articles, punctuation) |
| `recover.rs` | Heuristic text recovery for formats without proper parsers |

---

## 12. Enrichment Prompt

**File:** `processing/src/enrichment_prompt.rs`
- Interactive CLI prompt: accept all, pick individually, skip
- Used by CLI mode only; the Tauri app uses its own confirmation UI

---

## 13. EPUB Spine Command

**File:** `processing/src/commands.rs`
- `get_spine()` — parse EPUB container → OPF → spine itemrefs → resolved href list
- Shared library function used by both CLI and Tauri command layer

---

## 14. Calibre Import

**File:** `processing/src/import/calibre.rs` + `import/opf.rs`

| Feature | Module |
|---------|--------|
| Walk Calibre library folder | `calibre.rs` |
| Supported extensions filter | `calibre.rs` (7 formats) |
| SHA-256 dedup before ingest | `calibre.rs` |
| Parse `metadata.opf` sidecar | `opf.rs` |
| Full Calibre library import | `calibre.rs` (`import_calibre_library()`) |

---

## 15. Catalog Export

**File:** `processing/src/catalog.rs`

| Format | Function |
|--------|----------|
| CSV | `export_csv()` — full library: id, title, authors, format, publisher, series, series_index, pubdate, description |
| HTML | `export_html()` — styled HTML table: title, authors, format, series |

---

## 16. Integrity & Repair

**File:** `processing/src/integrity.rs` + `processing/src/repair.rs`

| Feature | Function |
|---------|----------|
| Check library integrity | `check_integrity()` — verifies all `local_path` files exist on disk |
| Repair single book | `repair_single_book()` — fixes missing authors, title, sort fields, local_path, cover, rating |
| Batch repair | `repair_books()` — iterates over all or specified books |

---

## 17. Tauri Command Layer

**File:** `src-tauri/src/commands.rs`

### 17.1 Commands (44 total)

**Library:**
- `list_books` — all books, sorted by last_opened_at
- `get_library` — alias for list_books
- `get_book_details` — full BookDetails (title, authors, tags, identifiers, series, etc.)
- `update_book_details` — bulk metadata edit (supports multiple book_ids)

**Search & Filter:**
- `search_library` / `search_books` — FTS5 search
- `filter_library` / `filter_books` — filter by format, status, author, tag, series
- `list_tags` — all unique tags
- `list_series` — all unique series names
- `list_authors` — all unique authors

**Ingest & Import:**
- `ingest_file` — single file import pipeline
- `import_calibre` — batch import from Calibre library folder

**Collections:**
- `create_collection` / `create_collection_cmd`
- `list_collections` / `list_collections_cmd`
- `delete_collection`
- `add_book_to_collection` / `add_book_to_collection_cmd`
- `remove_book_from_collection` / `remove_book_from_collection_cmd`
- `get_collection_books_cmd` / `get_books_in_collection`

**Bulk Actions:**
- `bulk_reingest` / `bulk_reingest_books` — reset status to PENDING
- `bulk_delete` / `bulk_delete_books` — remove books + covers + all related rows
- `bulk_export_metadata` / `export_metadata` — JSON or CSV export

**Reader:**
- `get_spine` — EPUB spine itemrefs
- `get_epub_chapter_html` — decorated chapter with reader JS
- `update_position` — store CFI reading position
- `update_progress` — store progress percent

**Bookmarks:**
- `add_bookmark` — CFI + optional label
- `list_bookmarks` — all bookmarks for a book
- `delete_bookmark`

**Annotations:**
- `create_annotation` — highlight/note/bookmark with CFI + color
- `get_annotations` — all annotations for a book
- `delete_annotation`
- `update_annotation_note`
- `sync_annotations` — push unsynced to xcalibre-server

**Conversion:**
- `convert_book_to_epub` — convert or tweak

**Maintenance:**
- `repair_books` — fix missing paths/covers/metadata
- `check_library_integrity` — verify files exist on disk
- `export_library_csv` — full library CSV
- `export_library_html` — full library HTML

**Comics:**
- `list_comic_pages` — extract CBZ page paths to temp dir

**External:**
- `open_in_os` — open file in OS default app

**Config:**
- `save_config` — persist autolib URL
- `get_xs_url` — read current server URL
- `has_token` — check keychain for service token

---

## 18. EPUB Protocol Handler

**File:** `src-tauri/src/epub_protocol.rs`
- Custom `epub://` URI scheme registered in Tauri WebView
- `epub_handler()` — resolves `epub://{job_id}/{path}` to ZIP entries
- `render_html()` — extracted HTML with injected reader script
- Injected reader script: CFI tracking, progress calculation, scroll→position emission
- `decorate_html()` — injects `<base>` tag + reader JS + theme/style support
- MIME detection for: HTML, CSS, PNG, JPEG, GIF, SVG, XML

---

## 19. Tauri Application Shell

**File:** `src-tauri/src/main.rs`
- Database initialisation at startup (`sqlx migrate!`)
- `Arc<SqlitePool>` managed state
- `epub://` custom URI scheme registration
- Tauri updater plugin
- Panic hook → crash log
- 44 registered `#[tauri::command]` invocations

---

## 20. API Client Crate

**Location:** `api/src/`

| File | Purpose |
|------|---------|
| `client.rs` | `ApiClient` — reqwest HTTP client with keyring token storage |
| `push.rs` | `PushPayload` struct + `push_book()` — upload file + metadata to server |
| `read.rs` | `get_book()`, `list_books()` — remote library reads |
| `enrichment/google_books.rs` | Google Books API lookup by ISBN |
| `enrichment/open_library.rs` | Open Library API lookup by ISBN |

---

## 21. Frontend (React)

**Location:** `ui/src/`

### 21.1 Stores (Zustand)

| File | State |
|------|-------|
| `store/libraryStore.ts` | Book list, filtered/search results, fetched via Tauri invoke |
| `store/settingsStore.ts` | Font size, theme, font family; persisted to localStorage |

### 21.2 Components

| File | Purpose |
|------|---------|
| `components/LibraryView.tsx` | Main library grid/list view |
| `components/BookDetail.tsx` | Full book detail panel |
| `components/SearchBar.tsx` | FTS-driven search input |
| `components/FilterBar.tsx` | Filter by format, author, tag, series |
| `components/CollectionsSidebar.tsx` | Collection CRUD + selection |
| `components/BulkActionBar.tsx` | Re-ingest, delete, export |
| `components/SettingsModal.tsx` | Reader/catalog settings |
| `components/MetadataEditorModal.tsx` | Full metadata editor (title, authors, tags, identifiers, series, etc.) |
| `components/ReaderView.tsx` | EPUB reader with position tracking |
| `components/ReaderToolbar.tsx` | Reader controls |
| `components/ComicViewer.tsx` | CBZ comic viewer |
| `components/BookmarkPanel.tsx` | Bookmark list in reader |
| `components/AnnotationsSidebar.tsx` | Annotations panel |
| `components/FormatOpener.tsx` | Open book in reader or external app |
| `components/ErrorBoundary.tsx` | React error boundary |
| `components/Skeleton.tsx` | Loading skeleton |
| `components/UpdateBanner.tsx` | Tauri updater notification |

### 21.3 Hooks

| File | Purpose |
|------|---------|
| `hooks/useKeyboard.ts` | Global keyboard shortcuts |
| `hooks/useReaderKeyboard.ts` | Reader-specific shortcuts |
| `hooks/useUpdater.ts` | Tauri auto-update check |

### 21.4 Reader

| File | Purpose |
|------|---------|
| `reader/cfi.ts` | CFI parsing + position-to-percent calculation |
| `reader/highlights.ts` | Text selection + highlight management |
| `reader.css` | EPUB content styling (themes: light, dark, sepia) |

### 21.5 E2E Tests

| File | Coverage |
|------|----------|
| `e2e/01-layout.spec.ts` | Basic layout presence |
| `e2e/02-library.spec.ts` | Library grid interactions |
| `e2e/03-bulk-actions.spec.ts` | Bulk action bar behavior |
| `e2e/04-settings.spec.ts` | Settings modal |
| `e2e/05-dark-mode.spec.ts` | Dark/light theme toggle |
| `e2e/06-keyboard.spec.ts` | Keyboard shortcuts |

### 21.6 Unit Tests

| File | Coverage |
|------|----------|
| `store/libraryStore.test.ts` | Store logic |
| `store/settingsStore.test.ts` | Persisted store |
| `components/BulkActionBar.test.tsx` | Component behavior |
| `components/CollectionsSidebar.test.tsx` | Component behavior |
| `components/FilterBar.test.tsx` | Component behavior |
| `components/LibraryView.test.tsx` | Component behavior |
| `components/MetadataEditorModal.test.tsx` | Component behavior |
| `components/SearchBar.test.tsx` | Component behavior |
| `components/SettingsModal.test.tsx` | Component behavior |
| `hooks/useKeyboard.test.ts` | Hook logic |
| `reader/cfi.test.ts` | CFI algorithm |

---

## 22. Integration Tests (Rust)

**Location:** `processing/tests/`

| File | Coverage |
|------|----------|
| `test_db.rs` | Database setup + basic CRUD |
| `test_metadata.rs` | Metadata extraction across formats |
| `test_text.rs` | Text extraction pipeline |
| `test_cover.rs` | Cover extraction + placeholder |
| `test_detect.rs` | Format detection + integrity |
| `test_formats_primary.rs` | Primary format ingest |
| `test_formats_extended.rs` | Extended format ingest |
| `test_formats_phase10.rs` | Phase 10 format coverage |
| `test_annotations.rs` | Annotation CRUD |
| `test_bookmarks.rs` | Bookmark CRUD |
| `test_library.rs` | Library metadata operations |
| `test_enrichment.rs` | Enrichment pipeline |
| `test_push.rs` | Push pipeline (mock) |
| `test_sync.rs` | Sync pipeline (mock) |
| `cbz_cover_tests.rs` | CBZ cover extraction |
| `chm_metadata_tests.rs` | CHM metadata recovery |
| `djvu_tests.rs` | DJVU handling |
| `lit_metadata_tests.rs` | LIT metadata recovery |
| `lit_text_tests.rs` | LIT text recovery |
| `lrf_tests.rs` | LRF/LRX handling |
| `pdb_ereader_tests.rs` | PDB eReader format |
| `snb_metadata_tests.rs` | SNB metadata |
| `snb_text_tests.rs` | SNB text |
| `tcr_text_tests.rs` | TCR text recovery |
| `txt_pipeline_tests.rs` | TXT ingest pipeline |
| `azw4_text_tests.rs` | AZW4 text extraction |

---

## 23. CI Pipelines

**Location:** `.github/workflows/`

| File | Purpose |
|------|---------|
| `release-macos.yml` | macOS universal binary build + DMG |
| `release-windows.yml` | Windows MSI build |

---

## 24. Build Infrastructure

| File | Purpose |
|------|---------|
| `Cargo.toml` (root) | Workspace definition (processing, api, src-tauri) |
| `processing/Cargo.toml` | Core crate dependencies |
| `api/Cargo.toml` | API client dependencies |
| `src-tauri/Cargo.toml` | Tauri app dependencies + version |
| `src-tauri/tauri.conf.json` | Tauri app config + version |
| `src-tauri/build.rs` | Tauri build script |
| `ui/package.json` | Node dependencies (React, Vite, Tailwind, Vitest) |
| `ui/vite.config.ts` | Vite build config |
| `ui/playwright.config.ts` | E2E test config |
| `ui/vitest.config.ts` | Unit test config |
| `ui/tailwind.config.js` | Tailwind theme configuration |
| `ui/postcss.config.cjs` | PostCSS pipeline |

---

## 25. Test Fixtures

**Location:** `processing/tests/fixtures/` + `tests/fixtures/`

```
fixture_cbr.cbr   fixture_cbz.cbz   fixture_djvu.djvu   fixture_docx.docx
fixture_epub.epub fixture_fb2.fb2   fixture_html.html   fixture_lit.lit
fixture_mobi.mobi fixture_odt.odt   fixture_pdb.pdb     fixture_pdf.pdf
fixture_rtf.rtf   fixture_tcr.tcr   fixture_txt.txt     fixture_zero.bin
```

Fixture generator: `processing/src/bin/gen_fixtures.rs`

---

## 26. Key Dependencies (Rust)

| Crate | Purpose |
|-------|---------|
| tokio | Async runtime |
| sqlx | SQLite async queries + migrations |
| clap | CLI argument parsing |
| thiserror | Error type derive |
| anyhow | Error handling in main() and tests |
| serde / serde_json | Serialisation |
| uuid | Job/entity ID generation |
| chrono | ISO 8601 datetime handling |
| sha2 | SHA-256 hashing |
| zip | EPUB/CBZ ZIP parsing |
| roxmltree | XML parsing (OPF, container.xml) |
| regex | HTML tag stripping for text extraction |
| image | Cover image resize (JPEG/PNG) |
| reqwest | HTTP client for API calls |
| keyring | OS keychain token storage |
| tracing / tracing-subscriber | Structured logging |
| tauri | Desktop shell |
| dirs | Platform app data paths |
| toml | Config file parsing |
| walkdir | Calibre library directory traversal |
