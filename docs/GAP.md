# xCalibre — Gap Analysis vs. Calibre

This document compares the current xCalibre codebase against Calibre's feature
set. It is a living gap analysis, not a phase plan.

**Last updated:** 2026-05-11 · Phase 14-ish (annotations, conversion, extended metadata)

**Calibre codebase:** 1,452 Python source files across 150+ modules.
**xCalibre codebase:** ~120 Rust source files, ~25 TypeScript/React files.

---

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented in xCalibre |
| ⚠️ | Partial / best-effort |
| 🟦 | In scope, planned but not yet implemented |
| ❌ | Intentionally out of scope |
| 🔶 | Missing / not yet planned |
| 🔴 | Major gap — significant subsystem |

---

## 1. Library Storage & Data Model

| Feature | Calibre | xCalibre | Gap Notes |
|---------|---------|----------|-----------|
| Single-library SQLite database | ✅ `metadata.db` | ✅ `jobs.db` + `local_books` | xCalibre keeps job state local and library rows in SQLite |
| Multiple simultaneous libraries | ✅ | 🟦 | In scope; the current app is still single-library only |
| `<Author>/<Title [ID]>/` filesystem layout | ✅ | ❌ | xCalibre processes files in place |
| OPF sidecar metadata per book | ✅ `metadata.opf` | ⚠️ | xCalibre reads Calibre `metadata.opf` on import, but does not write sidecars |
| Core normalised metadata model | ✅ | ✅ | Authors, tags, series, publisher, language, ratings, identifiers, sort fields are modeled |
| Comments / rich notes on books | ✅ | ❌ | Calibre has rich-text notes with `notes/` subsystem (export/import, schema) |
| Custom columns (user-defined fields) | ✅ | 🟦 | In scope; needs schema + UI support |
| Multiple format files per book | ✅ | ✅ | `book_formats` table tracks multiple formats per book |
| Virtual libraries / saved filter presets | ✅ | ❌ | Not planned |
| Library integrity check | ✅ | ✅ | Repair workflow now exists alongside the integrity check |
| Library repair | ✅ | ✅ | Common repair actions are implemented for missing paths, covers, and inconsistent metadata |
| Catalog generation | ✅ HTML / EPUB / CSV | ⚠️ | HTML and CSV export exist; Calibre's EPUB catalog pipeline is not implemented |
| **Library backup & restore** | ✅ | 🔶 | Calibre has `db/backup.py` + `db/restore.py` |
| **Cross-library copy** | ✅ | 🔶 | Calibre `db/copy_to_library.py` |
| **Page count estimation** | ✅ | 🔶 | Calibre `db/page_count.py` based on format heuristics |
| **DB event listeners** | ✅ | ❌ | Calibre's `db/listeners.py` notifies GUI on changes |

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
| DOCX | ✅ | ✅ | Metadata and text extraction |
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
| KFX (Kindle KFX) | ✅ | 🔶 | Calibre reads KFX; xCalibre does not |
| Topaz (TPZ) | ✅ | ❌ | Legacy Amazon format |
| KDL | ✅ | ❌ | Not planned |
| Haodoo | ✅ | ❌ | Chinese PDB variant |

### Conversion — MAJOR GAP

Calibre has **40 conversion plugins** (20 input + 20 output) covering all
major formats. xCalibre has a single `convert_book_to_epub()` function.

| Feature | Calibre | xCalibre | Gap Notes |
|---------|---------|----------|-----------|
| Plumber conversion pipeline | ✅ | ✅ | A dedicated conversion pipeline now persists jobs and outputs EPUB artifacts |
| Input conversion plugins | ✅ (20) | ⚠️ | xCalibre reads all formats via its format handlers but has only one output path |
| Output conversion plugins | ✅ (20) | 🔶 | xCalibre outputs **only EPUB** — no MOBI, PDF, DOCX, FB2, HTMLZ, LIT, LRF, PDB, PML, RB, RTF, SNB, TCR, TXT |
| CSS transform rules | ✅ | ⚠️ | EPUB tweak support exists, but Calibre's full rule language is not replicated |
| Search-and-replace in ebook content | ✅ | ✅ | EPUB tweak workflow now supports content rewriting |
| EPUB in-place tweak | ✅ | ✅ | A dedicated tweak mode is implemented alongside conversion |
| Multi-format output (MOBI, PDF, DOCX, etc.) | ✅ | 🔶 | **Currently only EPUB output is supported** |
| Conversion option configuration UI | ✅ | 🔶 | Calibre has 20+ format-specific config panels |

**Calibre output plugins xCalibre lacks:**
`docx_output`, `epub_output`, `fb2_output`, `html_output`, `htmlz_output`,
`lit_output`, `lrf_output`, `mobi_output`, `oeb_output`, `pdb_output`,
`pdf_output`, `pml_output`, `rb_output`, `rtf_output`, `snb_output`,
`tcr_output`, `txt_output`

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
| **AI/LLM-powered metadata** | ✅ | 🔶 | Calibre 7+ has built-in AI integration (OpenAI, Ollama, Google, etc.) for metadata — see §15 |
| Metadata download from Amazon | ✅ | ❌ | Not planned |
| Cover download from metadata sources | ✅ | ❌ | xCalibre extracts covers from files only |
| Source priority configuration | ✅ | ❌ | |
| Automatic unattended download | ✅ | ❌ | xCalibre requires user confirmation |
| **Metadata OPF write-back** | ✅ | 🔶 | Calibre writes `metadata.opf` sidecars; xCalibre does not |

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
| **Advanced search query language** | ✅ | 🔶 | Calibre has a custom search query parser (`search_query_parser.py`) with field-scoped boolean operators |

---

## 5. Collections & Organisation

| Feature | Calibre | xCalibre | Gap Notes |
|---------|---------|----------|-----------|
| Tags | ✅ | ✅ | Stored in `tags` + `book_tags`, exposed in search/filtering |
| Collections | ✅ | ✅ | CRUD + sidebar support |
| Series with index | ✅ | ✅ | Stored in `local_books`, exported in catalog views |
| Ratings | ✅ | ⚠️ | Rating field exists (0–5); Calibre's full rating UX is not matched |
| Reading lists | ✅ / partial | ❌ | Not planned |
| Multiple formats per book | ✅ | ✅ | `book_formats` table now supports this |
| Hierarchical tag browser | ✅ | 🔶 | Calibre has a dedicated tag browser sidebar (`gui2/tag_browser/`) |
| Similar books | ✅ | ❌ | Not planned |
| Mark books (colored flags) | ✅ | ❌ | Not planned |

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
| **Dictionary lookup** | ✅ | 🔶 | Calibre viewer has built-in dictionary lookup |
| **Print from viewer** | ✅ | 🔶 | Not implemented |
| **Reference mode** | ✅ | 🔶 | Calibre viewer has a reference mode for academic PDFs |
| **Auto-scroll** | ✅ | 🔶 | Not implemented |
| **LLM/AI chat in viewer** | ✅ | 🔶 | Calibre 7+ has AI chat panel within the viewer |
| **TOC panel in viewer** | ✅ | 🔶 | Calibre viewer has a navigable TOC panel |
| **Search within book in viewer** | ✅ | 🔶 | Calibre viewer has in-book search |
| **Reading themes** | ✅ | ⚠️ | xCalibre has light/dark/sepia; Calibre has many more |
| **Configurable line spacing, margins** | ✅ | 🔶 | Not implemented |

---

## 7. Device Sync

| Feature | Calibre | xCalibre | Gap Notes |
|---------|---------|----------|-----------|
| USB / MTP / USBMS detection | ✅ (30+ drivers) | ❌ | Intentionally out of scope |
| Kindle sync (with APNX page numbers) | ✅ | ❌ | Out of scope |
| Kobo sync | ✅ | ❌ | Out of scope |
| Send-to-device with conversion | ✅ | ❌ | Out of scope |
| Wireless device server | ✅ | ❌ | Out of scope |
| Android device support | ✅ | ❌ | Out of scope |
| Sony / Nook / PocketBook / many others | ✅ | ❌ | Out of scope |

xCalibre delegates delivery to xcalibre-server instead of talking to devices directly.

---

## 8. News & Recipes

| Feature | Calibre | xCalibre | Gap Notes |
|---------|---------|----------|-----------|
| News download recipes | ✅ | ❌ | Out of scope |
| Built-in recipe scheduler | ✅ | ❌ | Out of scope |
| Recipe editor | ✅ | ❌ | Out of scope |
| Recipe collection (100+ sources) | ✅ | ❌ | Out of scope |

---

## 9. Content Server

| Feature | Calibre | xCalibre | Gap Notes |
|---------|---------|----------|-----------|
| Built-in HTTP server | ✅ (50+ files) | ❌ | xCalibre uses xcalibre-server instead |
| OPDS catalog endpoint | ✅ | ❌ | Out of scope |
| Remote LAN library access | ✅ | ❌ | Handled by xcalibre-server |
| User authentication for server | ✅ | ❌ | Out of scope |
| Bonjour/mDNS discovery | ✅ | ❌ | Out of scope |
| WebSocket support | ✅ | ❌ | Out of scope |
| Embedded book rendering in browser | ✅ | ❌ | Out of scope |
| Server-side conversion | ✅ | ❌ | Out of scope |
| Server-side FTS | ✅ | ❌ | Out of scope |
| Multi-library management | ✅ | ❌ | Out of scope |

---

## 10. Plugin System

| Feature | Calibre | xCalibre | Gap Notes |
|---------|---------|----------|-----------|
| ZIP-based plugin install | ✅ | 🟦 | In scope; not yet implemented |
| Plugin API / UI | ✅ | 🟦 | In scope; not yet implemented |
| Store plugins | ✅ | 🟦 | In scope; would be user-installable via the planned plugin system |
| Metadata source plugins | ✅ | 🟦 | In scope; would be user-installable via the planned plugin system |
| **Device plugins** | ✅ | ❌ | Out of scope |
| **Input/output conversion plugins** | ✅ | 🟦 | Loosely conceptually planned |
| **Built-in plugin registry** | ✅ (`builtins.py`) | 🔶 | Calibre registers all built-in plugins centrally |

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
| Annotation sync with xcalibre-server | ✅ |

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
| Cover browser / Cover flow | ✅ | ⚠️ | Cover is shown in details; no dedicated cover-grid browser |
| Search bar | ✅ | ✅ | Wired to FTS |
| Filter bar | ✅ | ✅ | Wired to the library filters |
| Collections sidebar | ✅ | ✅ | Implemented |
| Bulk action bar | ✅ | ✅ | Implemented |
| Settings modal | ✅ | ✅ | Implemented |
| Keyboard shortcuts | ✅ | ✅ | Global and reader shortcuts are wired |
| Auto-updater | ✅ / system-dependent | ✅ | Tauri updater plugin is wired |
| macOS / Windows bundle targets | ✅ / system-dependent | ⚠️ | App bundle builds; full DMG packaging remains environment-dependent |
| **Built-in ebook editor** (edit book) | ✅ | 🔶 | Calibre has a full HTML/CSS IDE with syntax highlighting, spell check, diff viewer, font management, TOC editor — see §16 |
| **Store/buy books browser** | ✅ | ❌ | Calibre has an integrated ebook store browser |
| **Dark/light theme** | ✅ | ✅ | Implemented with Tailwind dark mode |
| **Ebook reader theming** | ✅ | ⚠️ | Basic font/size settings only |
| **Preferences GUI** (20+ panels) | ✅ | 🔶 | Calibre has comprehensive preference panels for conversion, metadata, plugins, keyboard, look & feel |
| **Drag and drop** | ✅ | 🔶 | Calibre has full DnD for book imports and reordering |
| **Quickview panel** | ✅ | 🔶 | Calibre's quickview drills into all books by the author/tag/series under the cursor without leaving the library view (`gui2/actions/show_quickview.py`) |
| **Unpack book** | ✅ | 🔶 | Calibre decodes DRM-free books back to source HTML/OPF (`gui2/actions/unpack_book.py`) |
| **Device context menu** | ✅ | ❌ | Out of scope |

---

## 14. Performance & Correctness

| Feature | Calibre | xCalibre |
|---------|---------|----------|
| Magic-byte-first format detection | ✅ | ✅ |
| SHA-256 deduplication | ❌ | ✅ |
| Async processing pipeline | ❌ | ✅ |
| Re-entrant pipeline | ⚠️ partial | ✅ |
| Zero `unwrap()` in library code | N/A | ✅ enforced by CI |
| `cargo clippy --workspace -- -D warnings` clean | N/A | ✅ enforced by CI |
| **Concurrent conversion jobs** | ⚠️ | ✅ (async pipeline) |
| **Memory-safe language** | ❌ (Python) | ✅ (Rust) |

---

## 15. 🔴 AI / LLM Integration — MAJOR GAP

Calibre 7.x has a comprehensive built-in AI/LLM subsystem that is entirely
absent from xCalibre. This is the **single biggest functional gap**.

### 15.1 Provider Backends

| Feature | Calibre | xCalibre | Notes |
|---------|---------|----------|-------|
| Provider plugin architecture | ✅ | 🔶 | `AIProviderPlugin` base class; each provider is a discoverable plugin |
| OpenAI backend | ✅ | 🔶 | |
| Google AI (Gemini) backend | ✅ | 🔶 | Default for text-to-text; includes 1,500 free web searches/day via Google Search grounding |
| GitHub Copilot backend | ✅ | 🔶 | |
| Ollama (local) backend | ✅ | 🔶 | Auto-detects per-model thinking capability (`can_think` from model metadata) |
| LM Studio backend | ✅ | 🔶 | |
| OpenRouter backend | ✅ | 🔶 | |
| OpenAI-compatible (generic) backend | ✅ | 🔶 | |
| Live backend updates without restart | ✅ | 🔶 | `module_version` mechanism allows hot-updating backend code |

### 15.2 Capability Model

Calibre's `AICapabilities` flag enum defines discrete provider capability classes:

| Capability | Calibre | xCalibre | Notes |
|------------|---------|----------|-------|
| Text-to-text (chat) | ✅ | 🔶 | |
| Text-to-image | ✅ | 🔶 | Gemini image generation + Imagen 3 via Google backend |
| Text+image-to-image | ✅ | 🔶 | Image editing capability |
| Text-to-speech (TTS) | ✅ | 🔶 | |
| Embeddings | ✅ | 🔶 | |
| Tool use / function calling | ✅ | 🔶 | `ChatMessageType.tool` and `developer`; `ResultBlockReason` includes `malformed_function_call`, `unexpected_tool_call`, `too_many_tool_calls` |

### 15.3 Chat & Conversation

| Feature | Calibre | xCalibre | Notes |
|---------|---------|----------|-------|
| Streaming responses | ✅ | 🔶 | SSE-based streaming with throttled rendering (50 ms) |
| Multi-turn conversation history | ✅ | 🔶 | `ConversationHistory` maintains full exchange with cumulative cost |
| Markdown detection & rendering | ✅ | 🔶 | Responses heuristically detected as markdown and rendered to HTML |
| Show reasoning dialog | ✅ | 🔶 | Reasoning trace from thinking models shown in a dedicated dialog |
| Save discussion as a note | ✅ | 🔶 | AI discussions can be saved to the library's rich notes system |
| Localized results | ✅ | 🔶 | `llm_localized_results` pref — AI can respond in the user's UI language |

### 15.4 Advanced AI Capabilities

| Feature | Calibre | xCalibre | Notes |
|---------|---------|----------|-------|
| Web-grounded answers with citations | ✅ | 🔶 | `ChatResponse.citations` + `web_links` — inline hyperlinks to source URLs injected into markdown output |
| Thinking / reasoning model support | ✅ | 🔶 | `reasoning` + `reasoning_details` fields on both `ChatMessage` and `ChatResponse`; supports o1, R1, Gemini Thinking |
| Configurable reasoning budget | ✅ | 🔶 | User-selectable: auto / low / medium / high / none via `reasoning_strategy_config_widget` |
| Model quality tier selection | ✅ | 🔶 | cheap-fast / medium / high-quality-slow via `model_choice_strategy_config_widget` |
| Content safety filtering | ✅ | 🔶 | `PromptBlockReason` (5 types) + `ResultBlockReason` (12 types) with human-readable error messages |
| API secrets stored with restricted permissions | ✅ | 🔶 | Config file written with `chmod 600`; keys hex-encoded before storage |

### 15.5 Book Discussion Feature

| Feature | Calibre | xCalibre | Notes |
|---------|---------|----------|-------|
| "Discuss book with AI" GUI action | ✅ | 🔶 | Keyboard shortcut `Ctrl+Alt+A` |
| AI panel in ebook viewer | ✅ | 🔶 | Integrated into the reader UI |
| Built-in quick actions | ✅ | 🔶 | Summarize, Chapters, Read Next, Universe, Series — all prompt templates |
| User-defined custom actions | ✅ | 🔶 | Template variables: `{books_word}`, `{is_are}`, `{title}`, `{authors}`, `{series}` |
| Configurable metadata context | ✅ | 🔶 | Users select which fields (including custom columns) are sent to the AI |
| Multi-book discussion | ✅ | 🔶 | Can discuss multiple selected books in a single session |
| Provider preference UI | ✅ | 🔶 | Full settings with API keys, model selection, reasoning strategy |

**Calibre AI architecture** (`src/calibre/ai/` — 27 files):
- `openai/`, `google/`, `ollama/`, `github/`, `lm_studio/`, `open_router/`, `openai_compatible/` — each with `backend.py` + `config.py`
- `config.py` — `ConfigureAI` widget with per-purpose provider selection
- `prefs.py` — `JSONConfig('ai', permissions=0o600)`, purpose-to-provider mapping
- `utils.py` — streaming response parser, citation injector, `StreamedResponseAccumulator`, markdown detection, reasoning strategy/model tier widgets

**Current xCalibre status:** Zero AI integration exists. No providers, no chat widget, no discussion dialog, no tool use, no streaming.

**Estimated effort:** Ollama single-provider + basic chat panel is ~3-4 weeks. Full multi-provider system with citations, reasoning, tool use, and book discussion feature is more like 12-16 weeks.

---

## 16. 🔴 Ebook Editor — MAJOR GAP

Calibre's **edit book** tool is a standalone IDE for ebook files. It is vastly
more capable than xCalibre's simple EPUB tweak pipeline.

| Feature | Calibre | xCalibre | Notes |
|---------|---------|----------|-------|
| EPUB & AZW3 editing | ✅ | 🔶 | |
| HTML source editor | ✅ | 🔶 | Syntax-highlighted code editor |
| CSS editor | ✅ | 🔶 | With live preview |
| Image viewer/editor | ✅ | 🔶 | |
| Table of Contents editor | ✅ | 🔶 | |
| Spell check | ✅ | 🔶 | Hunspell-based |
| Search & replace across all files | ✅ | 🔶 | Regex support |
| Visual diff viewer | ✅ | 🔶 | Shows changes between saved versions |
| Check book (validation) | ✅ | 🔶 | Validates EPUB structure, links, CSS, fonts, images |
| Font embedding/subsetting | ✅ | 🔶 | |
| Split/merge books | ✅ | 🔶 | |
| Insert special characters | ✅ | 🔶 | |
| Reports (book statistics) | ✅ | 🔶 | |
| Undo/redo across all files | ✅ | 🔶 | |
| Import/export files | ✅ | 🔶 | |
| Code completion | ✅ | 🔶 | HTML, CSS, Python |
| Live CSS preview | ✅ | 🔶 | |
| Checkpoint-based save history | ✅ | 🔶 | |
| Snippets | ✅ | 🔶 | |
| Function replace (regex across files) | ✅ | 🔶 | |

**Calibre edit-book stack** (`src/calibre/gui2/tweak_book/` — 60+ files):
- `editor/` — Full code editor widget with syntax highlighting (HTML, CSS, XML, Python, JavaScript) and smarts/auto-complete
- `diff/` — Visual diff viewer
- `check/` — Book validation (links, CSS, fonts, images, OPF)
- `spell.py` — Spell checking
- `search.py` — Multi-file search & replace
- `toc.py` — Table of Contents editor
- `fonts.py` — Font management
- `polish.py` — Polish toolkit integration
- `reports.py` — Book statistics
- `live_css.py` — Live CSS preview panel

**Current xCalibre status:** Only has `convert_book_to_epub()` with a basic text-repackaging pipeline. No visual editing, no syntax highlighting, no validation.

**Estimated effort:** A basic EPUB editor (file tree, textarea-based HTML editing, save) is 4-6 weeks. Matching Calibre's full capability (syntax highlighting, diff, validation, spell check, ToC editor) is more like 12-16 weeks.

---

## 17. 🔴 OEB Polish Toolkit — MAJOR GAP

Calibre's OEB polish toolkit is a programmatic library for manipulating EPUB
containers. It underpins both the edit-book tool and the CLI polish command.
xCalibre has nothing comparable.

| Feature | Calibre | xCalibre | Notes |
|---------|---------|----------|-------|
| EPUB container management | ✅ | 🔶 | Open/read/write/manipulate EPUB as a container |
| CSS parsing & manipulation | ✅ | 🔶 | |
| Cover manipulation (set/replace) | ✅ | 🔶 | |
| Font embedding | ✅ | 🔶 | |
| Font subsetting | ✅ | 🔶 | Subset fonts to used characters only |
| Image optimization | ✅ | 🔶 | Compress, rescale images within EPUB |
| Book splitting | ✅ | 🔶 | Split EPUB at chapter boundary into separate books |
| Book creation from scratch | ✅ | 🔶 | |
| Spell check within polish | ✅ | 🔶 | Hunspell integration |
| ToC editor | ✅ | 🔶 | |
| KEPUB conversion (Kobo) | ✅ | 🔶 | Kepubify |
| Book reports/statistics | ✅ | 🔶 | Word count, character count, file sizes |
| EPUB validation (check sub-system) | ✅ | 🔶 | CSS validation, font validation, image validation, link checking, OPF validation, parsing errors |
| Hyphenation insertion | ✅ | 🔶 | |
| Pretty-print HTML | ✅ | 🔶 | |
| Book upgrade (EPUB 2 → 3) | ✅ | 🔶 | |

**Calibre polish stack** (`src/calibre/ebooks/oeb/polish/` — 40+ files):
- `container.py` — EPUB container management
- `css.py`, `parsing.py` — CSS parsing
- `cover.py` — Cover manipulation
- `fonts.py`, `subset.py`, `embed.py` — Font handling
- `images.py` — Image optimization
- `split.py` — Book splitting
- `check/` — Validation subsystem (CSS, fonts, images, links, OPF, parsing)
- `spell.py` — Spell checking
- `toc.py` — ToC editor
- `reports.py`, `stats.py` — Book statistics
- `hyphenation.py`, `tts.py` — Content augmentation

**Estimated effort:** 6-10 weeks for a basic EPUB manipulation library. The full polish toolkit with validation subsystem is 12-20 weeks.

---

## 18. Store & Book Discovery — GAP

Calibre has a built-in ebook store browser that connects to many online stores.

| Feature | Calibre | xCalibre | Notes |
|---------|---------|----------|-------|
| Store search dialog | ✅ | ❌ | |
| 30+ store plugins (Amazon, Kobo, Google Books, etc.) | ✅ | ❌ | |
| Download books from stores | ✅ | ❌ | |
| Search results with covers, prices, DRM info | ✅ | ❌ | |
| Advanced search builder | ✅ | ❌ | |
| Search history | ✅ | ❌ | |

---

## 19. Metadata Sources — PARTIAL GAP

| Feature | Calibre | xCalibre | Notes |
|---------|---------|----------|-------|
| Open Library | ✅ | ✅ | Implemented |
| Google Books | ✅ | ✅ | Implemented |
| Amazon | ✅ | ❌ | |
| Edelweiss | ✅ | ❌ | |
| ISBNdb / xISBN | ✅ | ❌ | |
| Google Images (cover search) | ✅ | ❌ | |
| Multiple concurrent sources | ✅ | ⚠️ | xCalibre tries OL then GB sequentially |
| Cover download from sources | ✅ | ❌ | xCalibre extracts covers from files only |
| Source priority configuration | ✅ | ❌ | |
| Automatic download (unattended) | ✅ | ❌ | xCalibre requires user confirmation |
| Metadata worker queue | ✅ | ❌ | Calibre has async worker for metadata downloads |

---

## 20. Headless / Automation — GAP

Calibre has a significant headless/automation subsystem that xCalibre does not
match.

| Feature | Calibre | xCalibre | Notes |
|---------|---------|----------|-------|
| CLI database commands (20+ subcommands) | ✅ | 🔶 | Calibre has `calibredb add`, `list`, `search`, `export`, `catalog`, `check`, `backup`, `restore`, etc. |
| Web automation API | ✅ | ❌ | Calibre `web/automate/` for remote control |
| `ebook-convert` CLI | ✅ | 🔶 | xCalibre has ingest CLI but no standalone convert CLI |
| `ebook-polish` CLI | ✅ | 🔶 | No equivalent |
| `fetch-news` CLI | ✅ | ❌ | Out of scope |
| Headless server mode | ✅ | ❌ | Calibre can run as headless content server |

---

## 21. 🔴 Database / Infrastructure — MODERATE GAP

| Feature | Calibre | xCalibre | Notes |
|---------|---------|----------|-------|
| SQLite-based storage | ✅ | ✅ | |
| In-memory metadata cache | ✅ | ❌ | Calibre's `db/cache.py` caches all metadata in memory for fast access |
| Schema migrations | ✅ | ✅ | sqlx migrations |
| DB-backed annotations | ✅ | ✅ | |
| DB-backed notes/rich text | ✅ | ❌ | Calibre has `db/notes/` subsystem |
| Library view (virtual joins) | ✅ | ❌ | Calibre `db/view.py` creates a virtual library table |
| Concurrency locking | ✅ | ❌ | Calibre `db/locking.py` for multi-process safety |
| Backup & restore | ✅ | 🔶 | |
| Copy-to-library | ✅ | 🔶 | |
| Page count estimation | ✅ | 🔶 | |

---

## 22. 🔴 Spell Check Subsystem — MODERATE GAP

Calibre has Hunspell-based spell checking integrated into multiple subsystems.
xCalibre has none.

| Feature | Calibre | xCalibre |
|---------|---------|----------|
| Hunspell integration | ✅ (`src/calibre/spell/`) | ❌ |
| Spell check in metadata editor | ✅ | ❌ |
| Spell check in edit-book tool | ✅ | ❌ |
| Spell check in polish toolkit | ✅ | ❌ |
| Custom dictionary support | ✅ | ❌ |
| Language-aware spell checking | ✅ | ❌ |

---

## 23. Utilities — LONG-TAIL GAPS

Calibre has a vast `utils/` directory (100+ files). xCalibre has 4 utility modules.

| Utility | Calibre | xCalibre |
|---------|---------|----------|
| Text normalization | ✅ `cleantext.py` | ✅ `normalise.rs` |
| Title casing | ✅ `titlecase.py` | ❌ |
| Smart punctuation | ✅ `smartypants.py` | ❌ |
| Hyphenation | ✅ `hyphenation/` | ❌ |
| Word counting | ✅ `wordcount.py` | ❌ |
| Font metadata / scanning | ✅ `fonts/` | ❌ |
| Template language engine | ✅ `formatter.py` | ❌ |
| Safe filename generation | ✅ `filenames.py` | ❌ |
| Configuration framework | ✅ `config.py` | ✅ `config.rs` |
| Search query parser | ✅ `search_query_parser.py` | ❌ |
| Image handling | ✅ `img.py`, `magick/` | ❌ |
| Archive handling | ✅ `zipfile.py`, `seven_zip.py` | ❌ |
| HTTP/browser utils | ✅ `browser.py`, `https.py` | ❌ |
| IPC / worker pool | ✅ `ipc/` | ❌ |
| OpenSearch parsing | ✅ `opensearch/` | ❌ |
| Translation utilities | ✅ `translator/` | ❌ |
| mDNS discovery | ✅ `mdns.py` | ❌ |
| Trash / recycle bin | ✅ `trash/` | ❌ |
| OS file associations | ✅ `open_with/` | ❌ |

---

## Remaining In-Scope Gaps

These features are still missing or only partially matched versus Calibre:

1. **🔴 AI/LLM integration** — Entire subsystem missing. 7 providers, chat in GUI + viewer. Effort: 2-3 weeks basic, 8-12 weeks full.
2. **🔴 Ebook editor** — Full HTML/CSS IDE missing. Effort: 4-6 weeks basic, 12-16 weeks full.
3. **🔴 OEB Polish toolkit** — EPUB container manipulation library missing. Effort: 6-10 weeks basic, 12-20 weeks full.
4. 🔴 **Multi-format output conversion** — Currently EPUB-only output. Effort: ~4-6 weeks per additional output format.
5. 🟦 Boolean search syntax.
6. 🟦 Custom columns (schema + UI).
7. ❌ Saved searches / virtual libraries.
8. ❌ Search history.
9. ❌ Author name disambiguation.
10. ❌ OPDS / third-party metadata sources.
11. ❌ Goodreads / Amazon metadata plugins.
12. ❌ Series detection from title.
13. 🟦 Spell-check in the metadata editor.
14. ❌ Calibre's richer catalog EPUB pipeline.
15. ❌ Store/book discovery browser.
16. ❌ OPF metadata write-back.
17. 🔶 Library backup/restore.
18. 🔶 Metadata OPF write-back.
19. 🔶 Database annotations sync with calibre-server (✅ with xcalibre-server).
20. 🔶 Similar books.
21. 🔶 Hierarchical tag browser.

## Intentionally Out of Scope

These remain excluded by product boundary:

1. USB / MTP device sync (30+ device drivers).
2. Built-in content server and OPDS endpoint.
3. News recipes and recipe editor.
4. Built-in ebook store browser.
5. Device-specific support (Kindle, Kobo, Nook, Sony, etc.).
6. KFX / Topaz / other proprietary Amazon formats.
7. TTS / text-to-speech.
8. RapydScript compiler infrastructure.

---

## What xCalibre Does That Calibre Does Not

1. **Async processing pipeline** with SQLite-persisted, restartable stages — Calibre uses Python threads/IPC and does not persist pipeline state.
2. **xcalibre-server push and pull** integration — Calibre has its own built-in content server rather than integrating with an external service.
3. **User-confirmed enrichment suggestions**, never auto-applied — Calibre can auto-download and apply metadata without user review.
4. **Persistent enrichment cache** in SQLite with a 30-day TTL — Calibre's metadata source caches are in-session memory only and are lost on restart.
5. **SHA-256 content deduplication** — Calibre has no equivalent; the same file can be added multiple times.
6. **OS keychain token storage** via the `keyring` crate — Calibre stores API secrets in a `chmod 600` config file, not the OS keychain.
7. **Tauri auto-updater** integration with update notification banner.
8. **Zero `unwrap()` in library code** enforced by CI — not applicable to Calibre (Python).
9. **Annotation sync** with xcalibre-server — Calibre syncs annotations to devices/content-server but has no equivalent remote-service sync model.
10. **Rust memory safety** guarantees across the entire codebase — Calibre is Python/C.
11. **Structured error types** (`thiserror`) across all library code — not applicable to Calibre.
12. **Benchmark suite** (`criterion`) for DB and pipeline performance — Calibre has no equivalent Criterion benchmarks.

*Items removed from a prior version of this list:*
- ~~CFI-based reading progress~~ — Calibre's content server also persists CFI position in a `last_read_positions` table.
- ~~Multiple format files per book~~ — Calibre has supported this since its inception; it is a core feature.

---

## Size Comparison (Source Lines)

| Metric | Calibre | xCalibre | Ratio |
|--------|---------|----------|-------|
| Python/Rust source files | 1,452 `.py` | ~120 `.rs` | 12:1 |
| Format readers | 25+ formats | 24 formats (DetectedFormat variants) | 1.1:1 |
| Conversion plugins | 40 (20 in + 20 out) | 1 output | 40:1 |
| GUI source files | 200+ files | 16 components | 13:1 |
| Device drivers | 30+ | 0 | N/A |
| AI integration files | 27 files | 0 | N/A |
| Ebook editor files | 60+ files | 0 | N/A |
| OEB polish files | 40+ files | 0 | N/A |
| Content server files | 50+ files | 0 (delegated) | N/A |
| DB layer files | 20+ files | 10 files | 2:1 |
| Utility modules | 100+ files | 4 files | 25:1 |
| Tests | Extensive | 26 integration + 11 unit + 6 e2e | Significant |

xCalibre is not attempting to replicate Calibre's full scope — see "Intentionally Out of Scope" above. The areas where xCalibre intentionally lags (device sync, content server, news recipes, store browser) account for roughly 40% of Calibre's codebase. The remaining gaps are in conversion output, ebook editing, AI integration, and polish tooling — all of which are on the table for future phases.
