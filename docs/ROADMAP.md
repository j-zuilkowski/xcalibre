# xCalibre — Phase Roadmap

Generated: 2026-05-11. Derived from GAP.md and DECISIONS.md.
All secondary decisions (S1–S8) resolved 2026-05-11. See DECISIONS.md.

Phases are sequenced by dependency. Each phase lists its hard blockers, the
gap rows it closes, and a rough effort estimate.

---

## Dependency Graph

```
TIER 1  ──── Phase 1: Multiple Libraries          (gates: AI library context, notes, search)
         ──── Phase 2: Field-scoped Search Parser  (gates: Virtual Libraries)
         ──── Phase 3: Plugin System               (gates: all plugin-dependent features)
              │
TIER 2  ──── Phase 4: OPF Write-back & Calibre Round-trip
         ──── Phase 5: EPUB Manipulation Library   (gates: Ebook Editor, Conversion Output)
         ──── Phase 6: OS-native Spell Check       (gates: Ebook Editor Level C)
         ──── Phase 7: KFX Format Support          (full S5-A implementation)
              │
TIER 3  ──── Phase 8: AI — Ollama MVP + RAG       (needs Phase 1; S7-D sqlite-vec pipeline)
         ──── Phase 9: Conversion Output Tier 1    (TXT, HTML, DOCX — needs Phase 5)
         ──── Phase 10: Virtual Libraries          (needs Phase 2)
         ──── Phase 11: Rich Notes                 (needs Phase 1)
         ──── Phase 12: Similar Books              (needs Phase 1)
              │
TIER 4  ──── Phase 13: Ebook Editor (Level C)     (needs Phase 5 + Phase 6)
         ──── Phase 14: AI — Additional Providers  (needs Phase 8)
         ──── Phase 15: Conversion Output Tier 2   (PDF via S4-B WebView print; MOBI)
              │
TIER 5  ──── Phase 16: OEB Polish Toolkit          (builds on Phase 5)
         ──── Phase 17: AI — Advanced Features     (citations, reasoning, tool use)
         ──── Phase 18: Conversion Output Tier 3   (FB2, RTF, HTMLZ)
              │
TIER 6  ──── Phase 19: Library Backup & Restore
         ──── Phase 20: Custom Columns
         ──── Phase 21: Page Count / Cross-library Copy
         ──── Phase 22: Conversion Output Tier 4   (LRF, PDB, PML, RB, SNB, TCR, LIT)
```

---

## Phase Descriptions

---

### Phase 1 — Multiple Libraries
**Tier:** 1 · **Effort:** 3–4 weeks

**Gaps closed:**
- §1: Multiple simultaneous libraries (🟦 → ✅)
- §11: Per-library server sync relationship
- §20: Headless CLI `--library` flag

**Scope:**
- Rework `Config` to store a list of library entries (name, db_path, cover_dir, xs_url)
- Add `active_library_id` to config
- Library switcher UI (sidebar or menu): create, open, rename, delete
- Startup: prompt for library selection if none configured
- All DB pool initialization, migration, and path resolution switched to the
  active library's `db_path`
- `sync` pipeline and `push` pipeline become per-library
- Library creation wizard: present layout choice (in-place vs managed
  `<Author>/<Title [ID]>/`); default is in-place (S6-C)
- Managed layout: copy files into the managed tree on library creation or
  book add; required for full OPF write-back round-trip (D8)

---

### Phase 2 — Field-scoped Search Parser
**Tier:** 1 · **Effort:** 2–3 weeks

**Gaps closed:**
- §4: Boolean search syntax (🟦 → ✅)
- §4: Advanced search query language (🔶 → ✅)

**Scope:**
- Parse query strings of the form:
  `title:rust AND author:klabnik NOT tag:beginner`
- Support fields: `title`, `author`, `tag`, `series`, `format`, `publisher`,
  `language`, `rating`, `pubdate`
- Unscoped terms fall through to FTS5
- Expose via existing `search_library` and `filter_library` commands
- Update `SearchBar.tsx` to show field-hint autocomplete

---

### Phase 3 — Plugin System
**Tier:** 1 · **Effort:** 4–5 weeks

**Gaps closed:**
- §10: ZIP-based plugin install (🟦 → ✅)
- §10: Plugin API / UI (🟦 → ✅)
- §10: Built-in plugin registry (🔶 → ✅)

**Scope:**
- `xcalibre-plugin-sdk` crate: defines stable `extern "C"` vtable traits for
  `MetadataSourcePlugin`, `ConversionOutputPlugin`, `StorePlugin`
- `PLUGIN_API_VERSION` constant embedded in SDK and checked at load time
- Plugin loader: `libloading`-based ZIP extraction + `.so`/`.dylib` load
- Plugin manager UI: install from file, list installed, enable/disable, uninstall
- Plugin storage: `~/Library/Application Support/xcalibre/plugins/`
- Built-in plugin registry: Open Library and Google Books registered as
  built-in `MetadataSourcePlugin` implementations (no ZIP needed)

---

### Phase 4 — OPF Write-back & Calibre Round-trip
**Tier:** 2 · **Effort:** 2–3 weeks

**Gaps closed:**
- §1: OPF sidecar metadata per book (⚠️ → ✅)
- §3: Metadata OPF write-back (🔶 → ✅)
- §12: Preserve Calibre metadata on import (⚠️ → ✅)

**Scope:**
- `import/opf.rs` extended to write `metadata.opf` sidecars after any
  metadata edit
- `BookMetadata` → OPF 2.x serialization (via `quick-xml`)
- Triggered on: `update_book_details`, ingest completion, bulk metadata edit
- Round-trip validation: import a Calibre library, edit metadata, re-import
  into Calibre — verify round-trip fidelity
- OPF 3.x read support (currently only 2.x is read)

---

### Phase 5 — EPUB Manipulation Library (`xcalibre-epub`)
**Tier:** 2 · **Effort:** 6–10 weeks · **Gates Phases 9, 13, 15, 16**

**Gaps closed:**
- §17: EPUB container management (🔶 → ✅)
- §17: CSS parsing & manipulation (🔶 → ✅)
- §17: Cover manipulation (🔶 → ✅)
- §17: Font embedding (🔶 → ✅)
- §17: Font subsetting (🔶 → ✅)
- §17: Image optimization (🔶 → ✅)
- §17: Book splitting (🔶 → ✅)
- §17: EPUB validation subsystem (🔶 → ✅)
- §17: Book upgrade EPUB 2→3 (🔶 → ✅)

**Scope:**
- New `xcalibre-epub` crate in the workspace
- `Container` struct: open ZIP, list manifest items, read/write items by ID
- `OPF` struct: parse and write OPF 2.x/3.x
- CSS module: parse stylesheets, rewrite properties, inline/extract
- Cover module: extract, replace, resize cover from container
- Font module: list embedded fonts, embed new fonts, subset via `subsetter`
  crate (or custom)
- Image module: optimize, rescale images within container
- Split module: split at heading/chapter boundary
- Validation module:
  - Link checker (all `href` and `src` references resolve within container)
  - CSS validator (parse errors, unknown properties)
  - Font validator (embedded fonts match declarations)
  - Image validator (images load, correct MIME types)
  - OPF validator (required elements, spine integrity)
- Upgrade module: EPUB 2 NCX → EPUB 3 nav document migration

---

### Phase 6 — OS-native Spell Check
**Tier:** 2 · **Effort:** 2–3 weeks · **Gates Phase 13**

**Gaps closed:**
- §22: Spell check in metadata editor (❌ → ✅)
- §22: Spell check in edit-book (gates Phase 13)
- §1: Spell-check in metadata editor (🟦 → ✅)

**Scope:**
- `tauri-plugin-spellcheck`: new Tauri plugin in `src-tauri/`
  - macOS: calls `NSSpellChecker` via `objc2` crate
  - Windows: calls `ISpellChecker` via `windows-rs` crate
- Exposes: `check_word(word) → bool`, `suggestions(word) → Vec<String>`,
  `add_to_dictionary(word)`, `set_language(bcp47)`
- Wired into `MetadataEditorModal.tsx` for title/author/description fields
- Wired into the ebook editor (Phase 13) for HTML content

**Note:** Covers WebView layer only. Rust-side pipeline spell check deferred
per S2-A.

---

### Phase 7 — KFX Format Support
**Tier:** 2 · **Effort:** 4–6 weeks · **S5-A: full implementation**

**Gaps closed:**
- §2: KFX (Kindle KFX) (🔶 → ✅)

**Scope:**
- KFX container parser: handle `.kfx` (SQLite) and `.kfx-zip` (ZIP + SQLite)
- Metadata extraction from KFX fragments table
- Full text extraction from content fragments
- Cover extraction from resource fragments
- Add `DetectedFormat::Kfx` variant
- Integration tests with fixture files

---

### Phase 8 — AI Integration: Ollama MVP + RAG Pipeline
**Tier:** 3 · **Effort:** 5–7 weeks · **Needs Phase 1**

**Gaps closed (partial §15):**
- §15.1: Ollama provider backend (🔶 → ✅)
- §15.2: Text-to-text capability (🔶 → ✅)
- §15.3: Streaming responses (🔶 → ✅)
- §15.3: Multi-turn conversation history (🔶 → ✅)
- §15.3: Markdown detection & rendering (🔶 → ✅)
- §15.4: Per-model thinking detection (🔶 → ✅)
- §15.5: Built-in quick actions (🔶 → ✅)
- §15.5: Configurable metadata context (🔶 → ✅)

**Scope:**

*AI provider layer:*
- `xcalibre-ai` crate: `AIProvider` trait, `OllamaBackend`, `ChatMessage`,
  `ChatResponse`, `StreamedResponseAccumulator`
- Per-library AI config stored in library DB (active provider, model, prefs)
- Keyboard shortcut: `Ctrl+Alt+A` (matches Calibre)

*RAG pipeline (S7-D):*
- `sqlite-vec` added to `xcalibre-processing` (align version with
  xcalibre-server)
- `book_chunks` table: `book_id`, `chunk_index`, `chunk_text`, `embedding`
  (stored as sqlite-vec float32 blob)
- Chunking step added to ingest pipeline: after `job_text` is written, split
  into overlapping ~512-token segments and embed each via Ollama
  `nomic-embed-text` (or user-configured embedding model)
- `retrieve_chunks(book_id, query, top_k) → Vec<String>` DB helper: embeds
  the query and ANN-searches `book_chunks` for the closest segments
- Re-chunk command: triggered when the user switches embedding model or
  upgrades an existing library to RAG

*UI layer:*
- `AIChatPanel.tsx`: streaming chat widget, conversation history, cost display
- `BookDiscussDialog.tsx`: quick actions (summarize, chapters, read next,
  universe, series), configurable metadata fields sent as context
- Context sent to AI = book metadata fields + top-K retrieved chunks (not
  raw `job_text`), so books of any length fit in context
- AI panel in reader (`ReaderView.tsx` sidebar)

---

### Phase 9 — Conversion Output Tier 1: TXT, HTML, DOCX
**Tier:** 3 · **Effort:** 5–7 weeks · **Needs Phase 5**

**Gaps closed:**
- §2 Conversion: TXT output (🔶 → ✅)
- §2 Conversion: HTML output (🔶 → ✅)
- §2 Conversion: DOCX output (🔶 → ✅)

**Scope:**
- `xcalibre-epub` container → plain text (strip HTML tags, preserve paragraphs)
- `xcalibre-epub` container → single-file HTML (inline CSS, embed images as
  base64 or external)
- `xcalibre-epub` container → DOCX via `docx-rs` crate
- Conversion job UI: output format selector in the conversion dialog
- Each output format registered as a built-in `ConversionOutputPlugin`

---

### Phase 10 — Virtual Libraries & Saved Searches
**Tier:** 3 · **Effort:** 3–4 weeks · **Needs Phase 2**

**Gaps closed:**
- §4: Saved searches / virtual libraries (❌ → ✅)
- §4: Search history (❌ → ✅)

**Scope:**
- `virtual_libraries` table: name, query string (field-scoped syntax from Phase 2)
- Virtual library appears alongside real collections in the sidebar
- Active virtual library filters the book list dynamically
- Saved searches: named queries persisted to the library DB
- Search history: last 20 queries stored per library
- `VirtualLibraryEditor.tsx`: create/edit/delete virtual libraries

---

### Phase 11 — Rich Notes
**Tier:** 3 · **Effort:** 4–5 weeks · **Needs Phase 1**

**Gaps closed:**
- §1: Comments / rich notes on books (❌ → ✅)

**Scope:**
- `notes` table: item_type (author/tag/book), item_id, html_content,
  searchable_text, updated_at, images (stored as blobs or file refs)
- `NotesEditor.tsx`: rich-text editor (Tiptap or Quill) embedded in the
  book detail panel and accessible from author/tag views
- Notes browser: list all notes with search
- Export: notes to HTML or Markdown
- Import: import notes from Calibre `notes/` export format

---

### Phase 12 — Similar Books
**Tier:** 3 · **Effort:** 2–3 weeks · **Needs Phase 1**

**Gaps closed:**
- §5: Similar books (🔶 → ✅)

**Scope (S8-A — metadata similarity):**
- Similarity score based on shared tags, series, author, publisher, language
- `similar_books(book_id, limit) → Vec<BookId>` DB query
- "Similar books" section in `BookDetail.tsx`
- Optional: `SimilarBooksPanel.tsx` as a dedicated panel

---

### Phase 13 — Ebook Editor (Level C)
**Tier:** 4 · **Effort:** 12–16 weeks · **Needs Phase 5 + Phase 6**

**Gaps closed:**
- All of §16 (🔶 → ✅)

**Scope:**
- `EditBookView.tsx`: file tree sidebar, tabbed editor panels
- CodeMirror 6 embedded in WebView for HTML, CSS, XML, JavaScript editing
  with syntax highlighting and code completion
- Write-back: edits serialized and saved back into the EPUB container
  via `xcalibre-epub`
- Visual diff: before/after diff view using `xcalibre-epub` checkpoint saves
- TOC editor: drag-reorder navdoc entries, add/remove entries
- Font manager: list embedded fonts, add/remove, subset on save
- Check book: run `xcalibre-epub` validation, display results with jump-to-file
- Spell check: wired via Phase 6 `tauri-plugin-spellcheck`
- Multi-file search & replace with regex
- Snippet library
- Checkpoint-based save history (undo back to any checkpoint)
- Function replace (regex with transformation function)

---

### Phase 14 — AI: Additional Providers
**Tier:** 4 · **Effort:** 8–10 weeks · **Needs Phase 8**

**Gaps closed (remaining §15.1 and §15.4):**
- OpenAI, Google AI, GitHub Copilot, LM Studio, OpenRouter,
  OpenAI-compatible backends
- Web-grounded citations (Google)
- Thinking / reasoning model support
- Text-to-image capability
- TTS capability
- Tool use / function calling
- Content safety filtering
- Configurable reasoning budget
- API key storage via OS keychain (already done via `keyring` crate)

**Scope:**
- One backend implementation per provider following the `AIProvider` trait
  established in Phase 8
- `AISettingsModal.tsx`: provider selector, API key, model tier, reasoning
  strategy
- Citation rendering: inline hyperlinks injected into markdown output
- Reasoning trace dialog
- Cost display with currency

---

### Phase 15 — Conversion Output Tier 2: PDF, MOBI
**Tier:** 4 · **Effort:** 6–10 weeks · **Needs Phase 5**

**Gaps closed:**
- §2 Conversion: PDF output (🔶 → ✅)
- §2 Conversion: MOBI/AZW3 output (🔶 → ✅)

**Scope (PDF — S4-B Tauri WebView print):**
- Load EPUB spine into a hidden WebView with print CSS applied
- Trigger `window.print()` → PDF via Tauri's `print` API
- Write PDF to conversion output path

**Scope (MOBI):**
- MOBI 6 + MOBI 8 (AZW3) output from `xcalibre-epub` container
- Requires implementing PalmDOC record structure, EXTH header, NCX-to-Guide

---

### Phase 16 — OEB Polish Toolkit (remaining items)
**Tier:** 5 · **Effort:** 4–6 weeks · **Needs Phase 5**

**Gaps closed:**
- §17 remaining items: KEPUB conversion, hyphenation, pretty-print HTML,
  book reports/statistics (word count, character count, file sizes)

**Scope:**
- KEPUB output: inject Kobo-specific `epub:type` attributes and chapter
  markers into the EPUB container
- Hyphenation: `hyphenation` crate for soft-hyphen insertion
- Pretty-print HTML: format HTML source with consistent indentation
- Book statistics: word count (from `job_text`), character count, file sizes
  per spine item

---

### Phase 17 — AI: Advanced Features
**Tier:** 5 · **Effort:** 4–6 weeks · **Needs Phase 14**

**Gaps closed (remaining §15.3):**
- Save discussion as note (integrates with Phase 11)
- Localized results
- Image generation (cover from AI)

**Scope:**
- "Save to notes" button in `AIChatPanel.tsx` → creates a rich note
  (Phase 11 must exist)
- `llm_localized_results` preference: request AI responses in the user's
  locale
- Cover generation: send a text prompt describing the book to a
  text-to-image provider → save result as book cover

---

### Phase 18 — Conversion Output Tier 3: FB2, RTF, HTMLZ
**Tier:** 5 · **Effort:** 6–9 weeks · **Needs Phase 5**

**Gaps closed:**
- §2 Conversion: FB2, RTF, HTMLZ output

**Scope:**
- **FB2** (FictionBook 2): XML-based; generate `<FictionBook>` document from
  EPUB spine via `quick-xml`; embed cover as base64 binary section
- **RTF** (Rich Text Format): walk EPUB HTML spine, emit RTF control words for
  headings, paragraphs, bold/italic/underline; embed cover as `{\pict}`
- **HTMLZ** (HTML ZIP): repackage `xcalibre-epub` container as a flat ZIP with
  a single `index.html`, inlined CSS, and image assets — effectively a
  simplified EPUB without the OPF layer

---

### Phase 19 — Library Backup & Restore
**Tier:** 6 · **Effort:** 2–3 weeks

**Gaps closed:**
- §1: Library backup & restore (🔶 → ✅)
- §1: Cross-library copy (🔶 → ✅)

**Scope:**
- `backup_library()`: ZIP the library DB + cover directory to a timestamped
  archive
- `restore_library()`: unzip archive to a new library path, re-register in
  config
- Cross-library copy: copy a book (metadata + formats + cover) from one
  library to another

---

### Phase 20 — Custom Columns
**Tier:** 6 · **Effort:** 3–4 weeks

**Gaps closed:**
- §1: Custom columns (user-defined fields) (🟦 → ✅)

**Scope:**
- `custom_columns` schema: name, label, datatype (text, int, float, bool,
  datetime, rating, comments, tags)
- Column values stored in `custom_column_values` (EAV pattern or typed
  per-column tables, decide on schema)
- Custom columns appear in book detail, metadata editor, search
- Custom columns can be sent as AI context (Phase 8/14)

---

### Phase 21 — Page Count & Cross-library Copy
**Tier:** 6 · **Effort:** 1–2 weeks

**Gaps closed:**
- §1: Page count estimation (🔶 → ✅)
- §21: Cross-library copy (🔶 → ✅)

---

### Phase 22 — Conversion Output Tier 4: Legacy Formats
**Tier:** 6 · **Effort:** ~28 weeks (~4 weeks each) · **Needs Phase 5**

**Gaps closed:**
- §2 Conversion: LRF, PDB, PML, RB, SNB, TCR, LIT output

**Formats (in suggested build order):**
1. LRF — BBeB format for Sony Reader
2. PDB — Palm Database eBook (multiple sub-formats)
3. PML — Palm Markup Language
4. RB — Rocket Book
5. SNB — S60 Note Book (Nokia)
6. TCR — Psion text compression
7. LIT — Microsoft LIT (requires reverse-engineered DES key table)

Build sequentially within the Tier 6 window; each format is self-contained.

---

### Phase 25 — Release Infrastructure & CI/CD ✅
**Tier:** — · **Effort:** completed 2026-05-12 · **No feature prerequisites**

**Delivered:**
- `CLAUDE.md` and `AGENTS.md` — session instructions for Claude Code and Codex
- `.github/workflows/ci.yml` — Rust + frontend checks on every push/PR
- `.github/workflows/release.yml` — cross-platform installer builds (macOS arm64/x86_64, Windows MSI+NSIS, Linux deb) on `v*` tag push
- Version alignment: all 8 crate/package version sources set to `1.0.0`
- chmlib removed; CHM text and metadata extractors rewritten to pure Rust
- `v1.0.0` release published to GitHub

See `docs/phases/rmp25_release_ci.md` for full task breakdown.

---

## Total Effort Estimate

| Tier | Phases | Estimated Weeks |
|------|--------|-----------------|
| 1 | 1–3 | 9–12 |
| 2 | 4–7 | 12–22 |
| 3 | 8–12 | 19–26 |
| 4 | 13–15 | 26–36 |
| 5 | 16–18 | 14–21 |
| 6 | 19–22 | 34–43 |
| **Total** | **22 phases** | **~114–160 weeks** |

At one developer working full time, this represents roughly **2.5–3 years**
of work (longer than the pre-S-series estimate due to S3-all adding Tier 4
formats and S7-D adding the RAG pipeline). The Tier 1–3 phases (Phases 1–12)
represent the highest-value work and can be shipped as a coherent product
increment in ~40–60 weeks.

---

## Secondary Decisions — Resolved

All S1–S8 decisions resolved 2026-05-11. See DECISIONS.md for details.

| Decision | Resolution | Affected Phases |
|----------|-----------|-----------------|
| S1 Plugin ABI | S1-A: `extern "C"` + semver | Phase 3 |
| S2 Rust spell check | S2-A: none in Rust pipeline | Phase 5, 13, 16 |
| S3 Conversion scope | All tiers in scope | Phase 9, 15, 18, 22 |
| S4 PDF output | S4-B: Tauri WebView print | Phase 15 |
| S5 KFX depth | S5-A: full 4–6 week impl | Phase 7 |
| S6 Library layout | S6-C: configurable per library | Phase 1 |
| S7 AI context | S7-D: RAG + sqlite-vec chunks | Phase 8 |
| S8 Similar books | S8-A: metadata similarity | Phase 12 |
