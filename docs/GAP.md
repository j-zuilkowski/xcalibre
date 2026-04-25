# xCalibre — Gap Analysis vs. Calibre

This document compares the current xCalibre codebase against Calibre's feature
set. It is a living gap analysis, not a phase plan.

Last updated: 2026-04-24 · Phase 10 code complete

---

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented in xCalibre |
| ⚠️ | Partial / best-effort |
| 🟦 | In scope, planned but not yet implemented |
| ❌ | Intentionally out of scope |

---

## 1. Library Storage & Data Model

| Feature | Calibre | xCalibre | Gap Notes |
|---------|---------|----------|-----------|
| Single-library SQLite database | ✅ `metadata.db` | ✅ `jobs.db` + `local_books` | xCalibre keeps job state local and library rows in SQLite |
| Multiple simultaneous libraries | ✅ | 🟦 | In scope; the current app is still single-library only |
| `<Author>/<Title [ID]>/` filesystem layout | ✅ | ❌ | xCalibre processes files in place |
| OPF sidecar metadata per book | ✅ `metadata.opf` | ⚠️ | xCalibre reads Calibre `metadata.opf` on import, but does not write sidecars |
| Core normalised metadata model | ✅ | ✅ | Authors, tags, series, publisher, language, ratings, identifiers, sort fields are modeled |
| Comments / rich custom metadata | ✅ | ✅ | Bulk metadata editing and richer field editing are now implemented; Calibre's editor still has more affordances |
| Custom columns | ✅ | 🟦 | In scope; needs schema + UI support |
| Multiple format files per book | ✅ | ❌ | xCalibre tracks one primary file per ingest job |
| Virtual libraries / saved filter presets | ✅ | ❌ | Not planned |
| Library integrity check | ✅ | ✅ | Repair workflow now exists alongside the integrity check |
| Library repair | ✅ | ✅ | Common repair actions are implemented for missing paths, covers, and inconsistent metadata |
| Catalog generation | ✅ HTML / EPUB / CSV | ⚠️ | HTML and CSV export exist; Calibre's EPUB catalog pipeline is not implemented |

---

## 2. Format Support

### Read

| Format | Calibre | xCalibre | Gap Notes |
|--------|---------|----------|-----------|
| EPUB 2/3 | ✅ | ✅ | Full spine, OPF, cover, reader support |
| PDF | ✅ | ✅ | Metadata, text extraction, embedded cover extraction |
| MOBI / AZW3 | ✅ | ✅ | Metadata and text extraction; built-in viewer is not planned |
| CBZ / CBR | ✅ | ✅ | Comic metadata, page counts, archive handling, and a comic viewer are implemented |
| TXT | ✅ | ✅ | Plain text ingest |
| DOCX | ✅ | ✅ | Metadata + text extraction |
| ODT | ✅ | ✅ | Metadata + text extraction |
| FB2 | ✅ | ✅ | Metadata + text extraction |
| HTML / HTMLZ | ✅ | ✅ | Metadata + text extraction |
| RTF | ✅ | ✅ | Metadata + text extraction |
| CHM | ✅ | ✅ | Heuristic metadata and text recovery now produce usable ingest content |
| LIT | ✅ | ✅ | Heuristic metadata and text recovery now produce usable ingest content |
| LRF / LRX | ✅ | ✅ | Heuristic metadata and text recovery now produce usable ingest content |
| PDB / PML / RB | ✅ | ✅ | Heuristic decoding now produces usable ingest content |
| SNB | ✅ | ✅ | Text recovery now avoids the empty-stub fallback where possible |
| TCR | ✅ | ✅ | Heuristic text recovery now produces usable ingest content |
| AZW4 | ✅ | ✅ | Metadata and text recovery now produce usable ingest content |
| DJVU | ✅ | ✅ | Metadata and text recovery now produce usable ingest content |

### Conversion

| Feature | Calibre | xCalibre | Gap Notes |
|---------|---------|----------|-----------|
| Plumber conversion pipeline | ✅ | ✅ | A dedicated conversion pipeline now persists jobs and outputs EPUB artifacts |
| Input/output conversion plugins | ✅ | ⚠️ | Built-in conversion handlers exist; Calibre's full plugin model is not replicated |
| CSS transform rules | ✅ | ⚠️ | EPUB tweak support exists, but Calibre's full rule language is not replicated |
| Search-and-replace in ebook content | ✅ | ✅ | EPUB tweak workflow now supports content rewriting |
| EPUB in-place tweak | ✅ | ✅ | A dedicated tweak mode is implemented alongside conversion |

---

## 3. Metadata

| Feature | Calibre | xCalibre | Gap Notes |
|---------|---------|----------|-----------|
| EPUB OPF metadata extraction | ✅ | ✅ | |
| PDF metadata extraction | ✅ | ✅ | |
| MOBI metadata extraction | ✅ | ✅ | |
| DOCX / ODT / FB2 / HTML / RTF metadata extraction | ✅ | ✅ | Covered by Phase 6/7 format handlers |
| ISBN detection from content | ✅ | ✅ | EPUB + PDF ISBN detection |
| Open Library enrichment | ✅ | ✅ | User-confirmed suggestions only |
| Google Books enrichment | ✅ | ✅ | User-confirmed suggestions only |
| Enrichment result cache | ✅ | ✅ | SQLite cache prevents repeated lookups |
| Title sort / author sort computation | ✅ | ✅ | Normalisation is now part of ingest/import |
| Bulk metadata edit | ✅ | ✅ | Bulk edit flows are implemented via the metadata editor |
| Author name disambiguation | ✅ | ❌ | Not planned |
| OPDS / third-party metadata sources | ✅ | ❌ | Not planned |
| Goodreads / Amazon metadata plugins | ✅ | ❌ | Not planned |
| Series detection from title | ✅ | ❌ | Not planned |
| Spell-check in metadata editor | ✅ | 🟦 | In scope as part of the richer metadata editor workflow |

---

## 4. Search & Discovery

| Feature | Calibre | xCalibre | Gap Notes |
|---------|---------|----------|-----------|
| Full-text search (SQLite FTS) | ✅ | ✅ | FTS5-backed search is implemented |
| Restriction / filter by format | ✅ | ✅ | Filter bar uses the indexed library data |
| Restriction / filter by tag, author, series | ✅ | ✅ | Covered by Phase 5 queries/UI |
| Collections / saved grouping | ✅ | ✅ | xCalibre models these as collections |
| Boolean search syntax | ✅ | ❌ | xCalibre uses simpler FTS query semantics |
| Saved searches / virtual libraries | ✅ | ❌ | Not planned |
| Search history | ✅ | ❌ | Not planned |

---

## 5. Collections & Organisation

| Feature | Calibre | xCalibre | Gap Notes |
|---------|---------|----------|-----------|
| Tags | ✅ | ✅ | Stored in SQLite and exposed in search/filtering |
| Collections | ✅ | ✅ | CRUD + sidebar support |
| Series with index | ✅ | ✅ | Stored in the DB and exported in catalog views |
| Ratings | ✅ | ⚠️ | Rating field exists; Calibre's full rating UX is not matched |
| Reading lists | ✅ / partial | ❌ | Not planned |
| Multiple formats per book | ✅ | ❌ | Still a structural gap |

---

## 6. Reading

| Feature | Calibre | xCalibre | Gap Notes |
|---------|---------|----------|-----------|
| Built-in EPUB viewer | ✅ | ✅ | Served through `epub://` in the WebView |
| CFI position tracking | ✅ | ✅ | Implemented in the reader |
| Reading progress percent | ✅ | ✅ | Derived from CFI |
| Last-opened bookmark restore | ✅ | ✅ | Stored in the library DB |
| Annotations / highlights / bookmarks | ✅ | ✅ | Implemented in Phase 8 |
| Comic viewer (CBZ / CBR) | ✅ | ✅ | Dedicated comic page viewer is present |
| PDF viewer | ✅ | ❌ | xCalibre opens PDF in the OS default app |
| MOBI / AZW3 viewer | ✅ | ❌ | xCalibre opens these externally by design |
| Text-to-speech | ✅ | ❌ | Not planned |
| Font embedding / font override | ✅ | ❌ | Not planned |

---

## 7. Device Sync

| Feature | Calibre | xCalibre | Gap Notes |
|---------|---------|----------|-----------|
| USB / MTP / USBMS detection | ✅ | ❌ | Intentionally out of scope |
| Kindle sync | ✅ | ❌ | Out of scope |
| Kobo sync | ✅ | ❌ | Out of scope |
| Send-to-device with conversion | ✅ | ❌ | Out of scope |
| Wireless device server | ✅ | ❌ | Out of scope |

xCalibre delegates delivery to xcalibre-server instead of talking to devices
directly.

---

## 8. News & Recipes

| Feature | Calibre | xCalibre | Gap Notes |
|---------|---------|----------|-----------|
| News download recipes | ✅ | ❌ | Out of scope |
| Built-in recipe scheduler | ✅ | ❌ | Out of scope |
| Recipe editor | ✅ | ❌ | Out of scope |

---

## 9. Content Server

| Feature | Calibre | xCalibre | Gap Notes |
|---------|---------|----------|-----------|
| Built-in HTTP server | ✅ | ❌ | xCalibre uses xcalibre-server instead |
| OPDS catalog endpoint | ✅ | ❌ | Out of scope |
| Remote LAN library access | ✅ | ❌ | Handled by xcalibre-server |
| User authentication for server | ✅ | ❌ | Out of scope |

---

## 10. Plugin System

| Feature | Calibre | xCalibre | Gap Notes |
|---------|---------|----------|-----------|
| ZIP-based plugin install | ✅ | 🟦 | In scope; not yet implemented |
| Plugin API / UI | ✅ | 🟦 | In scope; not yet implemented |
| Store plugins | ✅ | 🟦 | In scope; not yet implemented |
| Metadata source plugins | ✅ | 🟦 | In scope; would be user-installable via the planned plugin system |

---

## 11. xcalibre-server Integration

These features are unique to xCalibre.

| Feature | Status |
|---------|--------|
| Push library-ready assets to xcalibre-server | ✅ |
| Push retry with exponential back-off | ✅ |
| Pull remote library into local DB | ✅ |
| Service token stored in OS keychain | ✅ |
| Offline-first operation | ✅ |

---

## 12. Import from Calibre

| Feature | Calibre | xCalibre | Gap Notes |
|---------|---------|----------|-----------|
| Import a Calibre library folder | N/A | ✅ | Walks the library tree and queues ingest jobs |
| Preserve Calibre metadata on import | N/A | ⚠️ | Reads `metadata.opf` sidecars, but does not recreate Calibre's on-disk model |
| Incremental import / skip duplicates | N/A | ✅ | SHA-256 deduplication prevents re-ingest |

---

## 13. UI & UX

| Feature | Calibre | xCalibre | Gap Notes |
|---------|---------|----------|-----------|
| Native Qt desktop GUI | ✅ | ❌ | xCalibre uses Tauri + React |
| Library grid / list view | ✅ | ✅ | Implemented |
| Book detail panel | ✅ | ✅ | Implemented |
| Cover browser | ✅ | ⚠️ | Cover is shown in details; no dedicated cover-grid browser |
| Search bar | ✅ | ✅ | Wired to FTS |
| Filter bar | ✅ | ✅ | Wired to the library filters |
| Collections sidebar | ✅ | ✅ | Implemented |
| Bulk action bar | ✅ | ✅ | Implemented |
| Settings modal | ✅ | ✅ | Implemented |
| Keyboard shortcuts | ✅ | ✅ | Global and reader shortcuts are wired |
| Auto-updater | ✅ / system-dependent | ✅ | Tauri updater plugin is wired |
| macOS / Windows bundle targets | ✅ / system-dependent | ⚠️ | App bundle builds; full DMG packaging remains environment-dependent on this machine |

---

## 14. Performance & Correctness

| Feature | Calibre | xCalibre |
|---------|---------|----------|
| Magic-byte-first format detection | ✅ | ✅ |
| SHA-256 deduplication | ❌ | ✅ |
| Async pipeline | ❌ | ✅ |
| Re-entrant pipeline | ⚠️ partial | ✅ |
| Zero `unwrap()` in library code | N/A | ✅ enforced by CI |
| `cargo clippy --workspace -- -D warnings` clean | N/A | ✅ enforced by CI |

---

## Remaining In-Scope Gaps

These features are still missing or only partially matched versus Calibre:

1. Boolean search syntax.
2. Saved searches / virtual libraries.
3. Search history.
4. Author name disambiguation.
5. OPDS / third-party metadata sources.
6. Goodreads / Amazon metadata plugins.
7. Series detection from title.
8. Spell-check in the metadata editor.
9. Calibre's richer catalog EPUB pipeline.

## Intentionally Out of Scope

These remain excluded by product boundary:

1. USB / MTP device sync.
2. Built-in content server and OPDS endpoint.
3. News recipes and recipe editor.

---

## What xCalibre Does That Calibre Does Not

1. Async processing pipeline with restartable stages.
2. xcalibre-server push and pull integration.
3. User-confirmed enrichment suggestions, never auto-applied.
4. Enrichment caching in SQLite.
5. SHA-256 based deduplication.
6. CFI-based reading progress and bookmark restore.
7. Tauri updater integration.
8. OS keychain token storage.
