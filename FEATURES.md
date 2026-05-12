# xcalibre — Features

## Library Management

- Import ebooks by drag-and-drop, file picker, or folder scan
- Automatic format detection from file content (magic bytes, not extension)
- SHA-256 duplicate detection — re-importing the same file is silently ignored
- Bulk import with progress tracking
- Import from an existing Calibre library (preserves metadata and covers)
- Multiple independent libraries — create, switch, and delete
- Copy or move books between libraries
- Library integrity check — finds missing files, broken covers, orphaned records

## Supported Formats

| Format | Read | Metadata | Full Text | Cover |
|--------|------|----------|-----------|-------|
| EPUB 2/3 | ✅ | ✅ | ✅ | ✅ |
| PDF | ✅ | ✅ | ✅ | ✅ |
| MOBI / AZW3 | ✅ | ✅ | ✅ | ✅ |
| AZW4 | ✅ | ✅ | — | — |
| KFX | ✅ | ✅ | — | — |
| CBZ | ✅ | — | — | ✅ |
| CBR | ✅ | — | — | — |
| FB2 | ✅ | ✅ | ✅ | — |
| DOCX | ✅ | ✅ | ✅ | — |
| ODT | ✅ | ✅ | ✅ | — |
| HTML | ✅ | ✅ | ✅ | — |
| HTMLZ | ✅ | ✅ | ✅ | — |
| RTF | ✅ | — | ✅ | — |
| CHM | ✅ | ✅ | ✅ | — |
| LIT | ✅ | ✅ | — | — |
| DjVu | ✅ | — | — | — |
| LRF | ✅ | — | — | — |
| PDB | ✅ | ✅ | ✅ | — |
| PML | ✅ | — | ✅ | — |
| RB | ✅ | — | — | — |
| SNB | ✅ | — | ✅ | — |
| TCR | ✅ | — | ✅ | — |
| TXT | ✅ | — | ✅ | — |
| LRX | ✅ | — | — | — |

## Metadata

- Editable fields: title, authors, publisher, series, series index, rating (0–5), tags, publication date, description, language
- Custom identifiers: ISBN, ISBN-10, ASIN, Goodreads ID, Google Books ID
- Bulk metadata editing across multiple selected books
- ISBN enrichment via Open Library and Google Books (concurrent lookup, 30-day cache)
- User confirms enrichment suggestions before they are applied
- Sort key generation: `title_sort` (articles moved to end), `author_sort` (Last, First)
- OPF write-back — metadata edits update the EPUB's OPF file on disk
- Metadata export as JSON or CSV

## Reading

- In-app EPUB reader with custom rendering engine
- CFI (Canonical Fragment Identifier) position tracking — resumes from exact position
- Reading progress as percentage
- Reading session statistics — time per session, total reading time, daily streak
- In-app comic book viewer (CBZ page-by-page)
- Open in OS default application for unsupported viewer formats

## Reader Appearance

- Light, Dark, and Sepia themes
- Adjustable font size (14–24px), font family, and line height
- Column layout (single or multi-column)
- Custom CSS injection per book or globally

## Annotations

- Highlights with configurable colour (yellow, green, blue, pink)
- Inline notes attached to any highlight
- Bookmarks with optional label
- Full annotation list view per book
- Annotation sync to xcalibre-server

## Search

- Full-text search across titles, authors, descriptions, and book body text
- FTS5 BM25 relevance ranking
- Field-prefix syntax: `title:dune`, `authors:herbert`, `tags:scifi`
- Boolean operators: `AND` (default), `OR`, `NOT`
- Phrase search: `"lord of the rings"`
- Prefix search: `tolk*`
- Filter bar: format, reading status, author, tag, series

## Collections and Organisation

- User-created collections (manual groupings)
- Virtual libraries — saved search expressions that dynamically filter the library
- Virtual library query language: field prefixes, comparison operators (`>=`, `<=`), `NOT`, quoted strings
- Tag management

## Format Conversion

Convert any supported input format to any of these output formats:

TXT · HTML · DOCX · PDF · MOBI · KEPUB · FB2 · RTF · HTMLZ · LRF · PDB · PML · RB · SNB · TCR

Conversion pipeline: source → EPUB (if not already EPUB) → target format.

## EPUB Editor

- In-place ZIP editing without full re-pack
- Spine chapter list with reordering
- Per-chapter HTML editor with syntax highlighting
- CSS editor
- Font management (embed, remove)
- Image management (replace, remove, optimise)
- OS-native spell check integration (macOS NSSpellChecker, Windows, Linux hunspell)
- EPUB pretty-print / normalise

## AI Features

- Book chat — RAG-based Q&A grounded in the book's text
- Conversation history across turns
- Context chunks scored by exact-word relevance (top 5 chunks injected)
- Citations — model output linked back to source passages
- Quick actions: Summarise, Key themes, Characters, Writing style, Critical analysis
- Reasoning budgets: None / Low / Medium / High (controls chain-of-thought tokens)
- Save AI response as a note attached to the book
- Copy full conversation to clipboard

### Supported AI Providers

| Provider | Type | API Key |
|----------|------|---------|
| Ollama | Local | Not required |
| LM Studio | Local | Not required |
| OpenAI | Remote | Required |
| Google Gemini | Remote | Required |
| OpenRouter | Remote | Required |

## Plugin System

- Install plugins from ZIP archives via the Plugin Manager
- Plugin types: MetadataSource, ConversionOutput, Store (future)
- Enable / disable plugins without uninstalling
- Stable C ABI (`extern "C"` vtables) for cross-compiler compatibility
- API version guard — mismatched plugins are rejected at install time
- Plugin SDK crate: `xcalibre-plugin-sdk`

See [docs/DEVELOPER_GUIDE.md §Plugin Development](docs/DEVELOPER_GUIDE.md#10-plugin-development) for the authoring guide.

## Server Sync (xcalibre-server)

- Push books and metadata to a self-hosted xcalibre-server instance
- Annotation sync (push unsynced highlights and notes)
- Pull remote library changes
- Exponential backoff retry (5 attempts before marking FAILED)
- OS keychain token storage (no plaintext credentials on disk)

## Backup and Restore

- Full library backup to a single ZIP archive
- Optional inclusion of ebook files in the backup
- Compressed backup format
- Restore from backup into any directory

## Custom Columns

Add user-defined metadata columns of type: text, number, boolean, date, rating, list.
Custom column values are stored per book and visible in the library grid.

## Developer / Power User

- CLI binary: `xcalibre-processing ingest`, `sync`, `auth`
- Environment variable overrides for server URL and database path
- `cargo test --workspace` — full test suite
- SQLite database at `{app_data}/xcalibre/library.db` — inspectable with any SQLite tool
