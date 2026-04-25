# xCalibre — Language-Agnostic Design Reference

This document describes the design of xCalibre in implementation-neutral terms.
It is derived from the Calibre source and adapted for the xCalibre scope.
Use it as the authoritative reference when porting to a new language or runtime.

---

## 1. Core Concepts

### Library
A **library** is a directory on disk that is the root of all book storage.
Every book lives under `<library>/<Author Name>/<Title [BookID]>/`.
A library is identified by a UUID stored in the database.

### Book
A **book** is a logical record. It has:
- A unique integer ID (database-assigned)
- One or more **format files** (EPUB, PDF, MOBI, …) stored on disk
- Exactly one **cover image** (`cover.jpg`) stored alongside the format files
- A set of **metadata fields** (title, authors, publisher, series, tags, …)
- A set of **identifiers** (ISBN, UUID, ASIN, …)

A book without any format files is valid (metadata-only record).

### Format
A **format** is a specific file type attached to a book. The same book may have
multiple formats (e.g., both EPUB and PDF). Each format is stored as a single
file named `<Title>.ext` in the book directory.

### Metadata
Metadata is stored in two places:
1. **Database** — canonical source of truth for all fields
2. **OPF sidecar** — `metadata.opf` written beside format files for portability

The database is always authoritative. The OPF is for export/import only.

---

## 2. Data Model

### Book Record
| Field | Type | Notes |
|---|---|---|
| id | integer | Auto-assigned primary key |
| uuid | string | UUID v4, generated on insert |
| title | string | Display title |
| title_sort | string | Sortable title (articles stripped) |
| author_sort | string | "Last, First" form for sorting |
| pubdate | datetime | Publication date |
| timestamp | datetime | Date added to library |
| last_modified | datetime | Last metadata edit |
| series_index | float | Position within a series (default 1.0) |
| path | string | Relative path from library root |
| has_cover | boolean | Whether cover.jpg exists |
| cover | bytes | Cover image (stored inline in some implementations) |

### Linked Entities (normalised)
Each entity has its own table and links to books via a join table:
- **Authors** (name, sort) — many-to-many with books
- **Tags** (name) — many-to-many with books
- **Series** (name) — many-to-many with books (with series_index per link)
- **Publishers** (name) — many-to-many with books
- **Languages** (lang_code) — many-to-many with books
- **Ratings** (rating 0–10) — one-to-one with books
- **Identifiers** (type, value) — many-to-one with books (e.g., isbn:9780…)
- **Comments** (text) — one-to-one with books (long description)

### Format File Record
| Field | Type | Notes |
|---|---|---|
| book_id | integer | Foreign key to book |
| format | string | Uppercase extension: EPUB, PDF, MOBI, AZW3, CBZ, CBR, TXT |
| name | string | Filename without extension |
| size | integer | File size in bytes |

### Reading State
| Field | Type | Notes |
|---|---|---|
| book_id | integer | Foreign key to book |
| device | string | Device or reader identifier |
| cfi | string | Canonical Fragment Identifier (position in spine) |
| pos_frac | float | Fractional progress 0.0–1.0 |
| timestamp | datetime | Last updated |

### Annotation
| Field | Type | Notes |
|---|---|---|
| id | integer | Primary key |
| book_id | integer | Foreign key to book |
| format | string | Format the annotation was made in |
| user_type | string | "local" or "sync" source |
| user | string | User/device identifier |
| timestamp | float | Unix timestamp |
| annot_id | string | Stable ID for sync |
| annot_type | string | "bookmark", "highlight", "note" |
| annot_data | json | Type-specific payload |
| searchable_text | string | Extracted text for FTS indexing |

### Custom Column
Users may define custom metadata fields at runtime. Each custom column has:
- A name, label, and datatype
- A display template and sort behaviour
- Storage as either a direct column (scalar types) or a link table (list types)

Supported custom datatypes: `text`, `integer`, `float`, `bool`, `rating`,
`datetime`, `series`, `comments`, `enumeration`, `composite`.

---

## 3. Book Storage Layout

```
<library_root>/
  <Author Name>/
    <Title> [<id>]/
      <Title>.<ext>       ← format files (one per format)
      cover.jpg
      metadata.opf        ← OPF sidecar (optional)
      notes/              ← annotation files (optional)
  metadata.db             ← SQLite database
```

Key rules:
- The `path` field in the books table stores the relative path from library root
- Paths are URL-safe encoded (spaces replaced with `_`)
- The database is always in the library root
- Cover images are always named `cover.jpg`
- Format filenames match the book's `name` field (not the title directly)

---

## 4. Format Detection

Format detection is magic-byte-first. File extension is used only as a fallback.

| Magic Bytes | Format |
|---|---|
| `PK\x03\x04` + `mimetype` = `application/epub+zip` | EPUB |
| `PK\x03\x04` (no mimetype or wrong mime) | CBZ |
| `%PDF-` | PDF |
| bytes[60:68] == `BOOKMOBI` | MOBI |
| bytes[0:7] == `Rar!\x1a\x07\x00` or `Rar!\x1a\x07\x01\x00` | CBR |
| No null bytes, ≥50% printable ASCII | TXT |

---

## 5. Metadata Extraction Pipeline

For each format, metadata is extracted by reading format-specific structures:

### EPUB
1. Open as ZIP
2. Read `META-INF/container.xml` → locate OPF file path
3. Parse OPF XML → extract `<dc:title>`, `<dc:creator>`, `<dc:language>`,
   `<dc:publisher>`, `<dc:date>`, `<dc:description>`, `<dc:identifier>`
4. Identify ISBN from identifiers with scheme `isbn`

### PDF
1. Read `%PDF-` header
2. Parse XMP metadata stream if present
3. Fall back to document information dictionary
4. Fields: Title, Author, Subject, Keywords, Creator, Producer, CreationDate

### MOBI / AZW3
1. Read PalmDB header (first 78 bytes)
2. Read MOBI header from first record
3. Extract EXTH records for title, author, publisher, description, ISBN

### CBZ / CBR
No text metadata. Cover is the first image file in archive order.

### TXT
No embedded metadata. Filename used as title.

---

## 6. Text Extraction Pipeline

Text extraction produces a plain-text representation for search indexing.

### EPUB
1. Open as ZIP
2. Locate OPF → parse manifest and spine
3. For each spine item (XHTML file) in order:
   - Read file content
   - Strip all HTML/XML tags
   - Collapse whitespace
4. Concatenate chapters with double newline
5. Count words by splitting on whitespace

### PDF
Text extraction via embedded font mapping and content stream parsing.
Falls back to OCR for image-only PDFs (out of scope for v1).

### MOBI
Extract from HTML records in PalmDB structure.

### TXT
Content is the extracted text directly.

---

## 7. Cover Extraction

### EPUB
1. Open as ZIP, locate OPF
2. Check manifest for item with `properties="cover-image"` or `id="cover"`
3. Check metadata for `<meta name="cover" content="..."/>`
4. Read image bytes from ZIP entry
5. Normalise to JPEG, max 500×750px

### MOBI
Cover stored in EXTH record type 201 (cover offset into image records).

### PDF
First page rendered as image.

### CBZ/CBR
First file in archive (sorted by name) treated as cover.

---

## 8. Conversion Pipeline (Plumber)

Format conversion is a staged pipeline. All formats pass through an
intermediate in-memory representation (OEB — Open eBook) before output.

```
Input File
    ↓
[Input Plugin]          parse format → OEB document tree
    ↓
[Preprocessing]         normalise HTML, fix encoding issues
    ↓
[Structure Detection]   detect chapters, TOC, page breaks
    ↓
[Metadata Injection]    write metadata into OEB document
    ↓
[CSS Processing]        flatten, rescale fonts, apply transforms
    ↓
[Output Plugin]         OEB document tree → output format
    ↓
Output File
```

Each stage is independently pluggable. Input and output plugins are selected
by source and target format.

---

## 9. Search and Filtering

### Full-Text Search (FTS)
- Annotations and book comments are indexed in FTS5 virtual tables
- Stemmed and unstemmed variants maintained in parallel
- Search returns book IDs with rank scores

### Field Filtering
Books can be filtered by any metadata field using a restriction syntax:
- `tag:"Science Fiction"` — exact tag match
- `author:Asimov` — partial author match
- `series:Foundation` — series match
- `pubdate:>2000` — date comparison
- `rating:>=4` — numeric comparison
- Multiple terms combined with `and`/`or`/`not`

### Virtual Libraries
A **virtual library** is a saved search restriction. It presents a filtered
subset of the full library as if it were an independent library. No data is
duplicated; the filter is applied at query time.

---

## 10. Sync Protocol

### Push (local → remote)
1. Compute SHA-256 of format file
2. Check remote for existing book with same SHA-256
3. If not found: POST metadata + format file to remote API
4. On success: store remote `book_id` locally, mark job COMPLETED
5. On failure: record error, increment retry count, schedule retry
6. Retry schedule: exponential backoff, cap at 60 minutes, max 5 retries

### Pull (remote → local)
1. GET paginated list of books from remote (newest first)
2. For each remote book not in local DB: insert into `local_books`
3. For each local book with changed remote status: update local status

### Status Back-propagation
Jobs marked COMPLETED on remote are updated locally. This keeps the local
job DB consistent with the remote library state.

---

## 11. Enrichment (Metadata Lookup)

Enrichment fetches additional metadata from external sources to fill gaps.

### Flow
1. Extract ISBN from book (from embedded metadata or filename pattern)
2. Query Open Library API: `https://openlibrary.org/api/books?bibkeys=ISBN:...`
3. Query Google Books API: `https://www.googleapis.com/books/v1/volumes?q=isbn:...`
4. Merge results: prefer Open Library for subject/publisher; Google for cover
5. Present as **suggestions** — never auto-apply
6. User confirms or rejects each field individually
7. Cache results for 30 days (keyed by ISBN)

### Rules
- All enrichment calls have a 10-second timeout
- Network failure is silent — enrichment is best-effort
- Enrichment never blocks processing
- No enrichment without an ISBN

---

## 12. Plugin System

Plugins extend the system without modifying core code.

### Plugin Types
| Type | Purpose |
|---|---|
| Input Format | Parse a file format into the internal representation |
| Output Format | Serialise internal representation to a file format |
| Metadata Reader | Extract metadata from a format file |
| Metadata Writer | Embed metadata into a format file |
| Metadata Source | Fetch metadata from an external service (Open Library, Google Books, …) |
| Device | Sync books to/from an e-reader device |
| Store | Browse and purchase from online book stores |
| Catalog | Generate catalogues (OPDS, HTML, …) |
| Custom Column | Add new field types to the metadata schema |

### Plugin Discovery
Plugins are discovered at startup from a designated plugins directory.
Each plugin declares the interface it implements and the formats/services it supports.

---

## 13. Offline-First Constraint

The system must be fully functional without any network connection:
- All format processing runs locally
- The job database is local SQLite
- The library database is local SQLite
- Sync, enrichment, and push to remote are optional operations
- The absence of a service token must not degrade local functionality
- Remote API errors must never surface to the user as blocking failures

---

## 14. Configuration

Configuration is layered:
1. **Defaults** — hardcoded sensible values
2. **Config file** — `config.toml` in the platform app data directory
3. **Environment variables** — override any config file value

Key configuration values:
| Key | Default | Description |
|---|---|---|
| `library_path` | Platform documents dir | Root of the book library |
| `db_path` | Platform app data dir | Path to the jobs SQLite database |
| `xs_url` | `https://api.xcalibre.app` | Remote API base URL |
| `log_level` | `info` | Tracing verbosity |

Service tokens (API keys) are stored in the OS keychain, never in config files or logs.
