# xcalibre User Guide

**Version 1.0.0**

xcalibre is a desktop ebook library manager. It imports, organises, and reads ebooks locally — no account required. An optional connection to xcalibre-server adds cloud backup and annotation sync.

---

## Table of Contents

1. [Installation](#1-installation)
2. [First Launch and Application Layout](#2-first-launch-and-application-layout)
3. [Importing Books](#3-importing-books)
4. [The Library](#4-the-library)
5. [Reading EPUB Books](#5-reading-epub-books)
6. [Reading Comics (CBZ)](#6-reading-comics-cbz)
7. [Opening Other Formats](#7-opening-other-formats)
8. [Metadata Editing](#8-metadata-editing)
9. [Collections](#9-collections)
10. [Virtual Libraries](#10-virtual-libraries)
11. [Bookmarks](#11-bookmarks)
12. [Annotations and Highlights](#12-annotations-and-highlights)
13. [Notes](#13-notes)
14. [Similar Books](#14-similar-books)
15. [AI Assistant](#15-ai-assistant)
16. [Bulk Actions](#16-bulk-actions)
17. [Format Conversion](#17-format-conversion)
18. [Export](#18-export)
19. [Backup and Restore](#19-backup-and-restore)
20. [Custom Columns](#20-custom-columns)
21. [EPUB Editor](#21-epub-editor)
22. [Plugins](#22-plugins)
23. [Reading Statistics](#23-reading-statistics)
24. [Multiple Libraries](#24-multiple-libraries)
25. [Library Maintenance](#25-library-maintenance)
26. [Settings Reference](#26-settings-reference)
27. [Keyboard Shortcuts](#27-keyboard-shortcuts)
28. [Supported Formats](#28-supported-formats)
29. [Troubleshooting](#29-troubleshooting)

---

## 1. Installation

xcalibre is distributed as a native desktop application.

**macOS**

1. Download the `.dmg` file from the xcalibre releases page.
2. Open the `.dmg` — a Finder window opens showing the xcalibre icon.
3. Drag xcalibre to your **Applications** folder.
4. Open xcalibre from Applications or Spotlight. On first launch macOS may ask you to confirm opening an app from the internet — click **Open**.

**Windows**

1. Download the `.msi` installer.
2. Double-click to run it. Follow the installer prompts.
3. xcalibre is added to your Start menu.

**Automatic updates**

xcalibre checks for updates at startup and when the app regains focus. When an update is available, a banner appears at the top of the library window:

> *xcalibre 1.1.0 is available — Update & Restart*

Click **Update & Restart** to download and apply the update. The app restarts automatically. You can dismiss the banner to defer the update.

**System requirements**

| Platform | Minimum version |
|----------|----------------|
| macOS | 12 Monterey |
| Windows | Windows 10 (64-bit) |

---

## 2. First Launch and Application Layout

### Data directory

On first launch xcalibre creates a local database and data directory:

| Platform | Path |
|----------|------|
| macOS | `~/Library/Application Support/xcalibre/` |
| Windows | `%APPDATA%\xcalibre\` |

The database file is `jobs.db` inside that directory. Cover thumbnails are stored alongside it. **xcalibre never moves or modifies your original book files** — it records their paths and makes its own copies of cover images only.

No account is required. xcalibre is fully functional with no internet connection.

### Window layout

The xcalibre window is divided into three areas:

```
┌──────────────────────────────────────────────────────┐
│  Toolbar: Search · Filter · Import · Settings        │
├────────────────┬─────────────────────────────────────┤
│                │                                     │
│  Sidebar       │   Library (book grid)               │
│                │                                     │
│  • All Books   │   [cover] [cover] [cover] [cover]  │
│  • Collections │   [cover] [cover] [cover] [cover]  │
│  • Virtual     │                                     │
│    Libraries   │                                     │
│  • Libraries   │                                     │
│                │                                     │
└────────────────┴─────────────────────────────────────┘
```

Clicking a book opens its **detail panel** on the right side of the library. The sidebar can be collapsed to give more room to the grid.

---

## 3. Importing Books

### Drag and drop

The most direct way to add books: drag one or more files onto the xcalibre window. You can drag a single file, a selection, or an entire folder. xcalibre processes each file through a six-stage import pipeline:

| Stage | What happens |
|-------|-------------|
| **1. Detect** | Identifies the file format from its binary content — magic bytes — not the filename extension. A file named `book.pdf` that is actually an EPUB is detected correctly. |
| **2. Validate** | Checks structural integrity: EPUB files are valid ZIPs with `mimetype` and `container.xml`; PDFs have the correct `%PDF-` header and `%%EOF` trailer; MOBI files have the correct PalmDOC header signature. |
| **3. Deduplicate** | Computes a SHA-256 checksum of the file contents. If a file with that exact checksum is already in the library with `COMPLETED` status, the import is skipped. Renaming or moving a file does not create a duplicate. |
| **4. Extract metadata** | Reads title, authors, publisher, series, series index, language, description, and identifiers (ISBN, ASIN) from the file. The extraction method is format-specific — OPF for EPUB, Info dict/XMP for PDF, EXTH records for MOBI, FictionBook XML for FB2, etc. |
| **5. Extract cover** | Extracts the embedded cover image, resizes it to fit within 500 × 750 pixels, and saves a JPEG copy in the xcalibre data directory. If no cover is found, a placeholder is generated. |
| **6. Extract text** | Extracts the full text content and indexes it for full-text search. |

Each stage runs independently. If text extraction fails for an unusual EPUB, the book still appears in your library with its metadata and cover — only search over the body text is affected. You can re-run the pipeline at any time with **Re-ingest**.

### ISBN enrichment

If xcalibre finds an ISBN in the file's metadata, it queries **Open Library** and **Google Books** (each with a 10-second timeout, running concurrently). When results are returned, a confirmation dialog shows you the suggested changes:

- Corrected title or subtitle
- Author names in canonical form
- Publisher, publication date, description, cover art

You choose which suggestions to accept, individually or all at once. xcalibre never applies enrichment without your confirmation.

Enrichment results are cached for 30 days. Re-ingesting a book with the same ISBN uses the cached result rather than making new network requests.

### Import from Calibre

If you have an existing Calibre library, xcalibre can import it in bulk.

1. Click the **Import from Calibre** button in the toolbar, or go to **Settings → Import**.
2. Select your Calibre library folder — the one that contains `metadata.db` and subdirectories for each author.
3. xcalibre walks the folder tree. For each book directory it finds:
   - Imports all supported book files (EPUB, PDF, MOBI, AZW3, CBZ, CBR, TXT)
   - Reads the `metadata.opf` sidecar file for pre-existing metadata (title, authors, tags, series, identifiers, description)
   - Skips files already in the library by SHA-256

The import result shows: **found** (total files discovered), **imported** (new books added), **queued** (books waiting to be processed), **skipped** (duplicates), **errors** (failed files).

The Calibre import does not require Calibre to be installed — xcalibre reads the folder structure and OPF files directly.

### File size limit

xcalibre's import pipeline has a 500 MB per-file limit. Files larger than this are rejected at stage 1 with a clear error message.

### Supported import formats

EPUB · PDF · MOBI · AZW3 · AZW4 · CBZ · CBR · TXT · FB2 · HTML · HTMLZ · RTF · DOCX · ODT · CHM · LRF · LRX · PDB · PML · RB · SNB · TCR · DJVU · LIT

See [Section 28 — Supported Formats](#28-supported-formats) for per-format capability details.

---

## 4. The Library

### Grid view

The library shows books as a responsive grid of cover thumbnails. The number of columns adjusts automatically with window width (2 on narrow windows, up to 5 on wide displays).

Each card shows:
- **Cover image** (or format name as placeholder if no cover was extracted)
- **Title** (truncated to one line)
- **Authors** (truncated to one line)
- **Reading progress bar** — a thin bar at the bottom of the card, visible once you have started reading. Shows percentage complete.
- **Checkbox** — top-right corner of each card. Used for [bulk selection](#16-bulk-actions).

Click a card to open the book's detail panel on the right. Double-click to open it directly in the reader.

### Searching

Type in the search bar at the top of the library. Press **Enter** to search; press **Escape** to clear.

xcalibre uses SQLite FTS5 (full-text search). Searches are fast even on libraries with tens of thousands of books.

**What is searched:** title, authors, description, and the full extracted text of every book.

**Search syntax**

xcalibre supports a field-prefix query syntax. The search bar shows an autocomplete hint when you type a field prefix ending with `:`:

| Prefix | Searches |
|--------|---------|
| `title:` | Book title |
| `author:` | Author names |
| `tag:` | Tags |
| `series:` | Series name |
| `format:` | File format (EPUB, PDF, etc.) |
| `publisher:` | Publisher name |
| `language:` | Language code or name |

**Examples**

```
tolkien                          → full-text search for "tolkien"
title:dune                       → books with "dune" in the title
author:asimov tag:scifi          → Asimov books tagged "scifi"
series:"The Expanse"             → exact series name
format:epub tag:unread           → unread EPUBs
author:le guin NOT tag:short     → Le Guin but not short stories
```

**Tips**
- Multiple terms without a field prefix are combined with implicit AND.
- Phrase search: wrap in double quotes — `"foundation and empire"`.
- Partial words are matched — `tolkien` matches "Tolkien" and "Tolkienian".
- An unbalanced quote or a bare `AND`/`OR`/`NOT` without operands returns empty results rather than an error.

### Filtering

The filter bar below the toolbar narrows results by metadata fields without affecting the search query. Filters and search work together — filtered results are searched, searched results are filtered.

| Filter | Values | Match type |
|--------|--------|-----------|
| **Format** | EPUB, PDF, MOBI, AZW3, CBZ, CBR, TXT | Exact |
| **Author** | Text field | Substring match on `authors_json` |
| **Tag** | Dropdown of all tags in library | Exact |
| **Series** | Dropdown of all series in library | Exact |
| **Status** | PENDING, COMPLETED, FAILED, RETRYING, etc. | Exact |

Click **Clear filters** (×) to reset all filters at once.

**Status values explained**

| Status | Meaning |
|--------|---------|
| PENDING | Queued for processing |
| COMPLETED | Fully imported and indexed |
| RETRYING | A push to xcalibre-server failed; automatic retry scheduled |
| FAILED | Push failed 5 times; manual intervention required |
| READY\_TO\_PUSH | Processing complete, waiting to send to server |

For most local-only use, all books will be COMPLETED.

### Selecting books

To select a book for bulk operations, click its checkbox (top-right of the card). The card gets a blue ring outline when selected.

Keyboard selection:
- Click a book card to select it.
- `Ctrl+A` (Windows) / `Cmd+A` (macOS) — select all visible books.
- Click one book, then Shift-click another to select a range.

Selected count is shown in the [Bulk Action bar](#16-bulk-actions) at the bottom of the screen.

### Book detail panel

Click a book card to open the detail panel on the right side of the screen. The panel shows:

- **Cover** (large)
- **Title, Authors, Format**
- **Reading progress** — percentage and progress bar
- **Reading stats** — total reading time, session count, estimated page count
- **Tabs**: Details · Notes · Stats · Custom
- Action buttons: **Read**, **Edit Metadata**, **Edit EPUB** (EPUBs only), **Convert**, **Open in OS**, **Similar Books**, **Discuss with AI**

---

## 5. Reading EPUB Books

### Opening the reader

Double-click a book card, or click a book and press **Enter**, or click **Read** in the detail panel.

The reader opens as a full-window view, replacing the library.

### Chapter navigation

EPUBs are divided into spine items (chapters, sections, or parts). The reader displays one spine item at a time.

| Action | Method |
|--------|--------|
| Next chapter | Arrow button (right side of screen), `→` key, or `Page Down` |
| Previous chapter | Arrow button (left side), `←` key, or `Page Up` |
| Jump to a chapter | Click the **chapter list** icon in the toolbar to open a dropdown of all spine items |
| Scroll within a chapter | Mouse wheel, trackpad swipe, or keyboard scroll keys |

### Progress tracking

xcalibre automatically saves your reading position using a **CFI (Canonical Fragment Identifier)** — a precise location within the EPUB structure, accurate to the element and scroll offset. Your position is saved every few seconds while reading and when you close the reader.

When you reopen a book, the reader scrolls to exactly where you left off.

**Progress percentage** is calculated from your position in the spine relative to the total number of spine items and scroll position within each item. It appears in the library grid as a thin bar at the bottom of the cover card.

### Reading themes

Three built-in colour themes are available from the reader toolbar:

| Theme | Background | Text | Best for |
|-------|-----------|------|---------|
| **Light** | White | Black | Bright environments |
| **Dark** | Dark grey/charcoal | Light grey | Low-light reading |
| **Sepia** | Warm off-white | Dark brown | Reducing eye strain in daylight |

Switch themes with the theme selector in the toolbar, or press `T` to cycle through them.

### Typography settings

The reader toolbar provides three typography controls that take effect instantly:

**Font size** — slider from 14 px to 24 px. Default is 18 px. You can also use keyboard shortcuts:
- `Ctrl+=` / `Cmd+=` — increase font size
- `Ctrl+-` / `Cmd+-` — decrease font size

**Font family**

| Option | Description |
|--------|-------------|
| **Serif** | Traditional book font (e.g. Georgia). Good for long-form reading. |
| **Sans-serif** | Clean, modern font (e.g. system sans). Good for technical content. |
| **Monospace** | Fixed-width font. Good for code-heavy content. |

**Note:** These settings override the EPUB's own CSS. If an EPUB specifies its own fonts and they conflict with xcalibre's reader CSS, xcalibre's settings take precedence.

All typography settings are persisted to `localStorage` and apply to every book you read.

### The toolbar

The reader toolbar is visible at the top of the reader. It contains:

- **← Library** — return to the library (position is saved)
- **Chapter list** — jump to any spine item
- **Font size slider**
- **Theme selector** (Light / Dark / Sepia)
- **Font family selector** (Serif / Sans / Mono)
- **Bookmarks panel** toggle
- **Annotations panel** toggle

### Closing the reader

Press **Escape**, click **← Library** in the toolbar, or use the back button. Your position is saved automatically — you do not need to do anything special to preserve it.

---

## 6. Reading Comics (CBZ)

CBZ (Comic Book ZIP) files open in the built-in comic viewer.

### Navigation

| Action | Method |
|--------|--------|
| Next page | Click the right arrow, press `→` |
| Previous page | Click the left arrow, press `←` |
| First page | Press `Home` |
| Last page | Press `End` |

### How it works

When you open a CBZ, xcalibre extracts each image (JPEG, PNG, WebP) from the archive to a temporary folder. The viewer displays them in filename order — which corresponds to page order for well-named archives.

The temporary files are cleaned up when you close the comic viewer. If xcalibre crashes while a CBZ is open, the temp files remain in your system temp directory and will be cleaned up by your OS on next restart.

CBR (Comic Book RAR) files cannot be displayed in the built-in viewer. They open in your system's default app instead.

---

## 7. Opening Other Formats

For formats other than EPUB and CBZ, xcalibre opens the file in your operating system's default application:

- PDF → Preview (macOS) or Adobe Reader / Edge (Windows)
- MOBI, AZW3 → Kindle app (if installed) or browser
- DOCX → Pages / Word
- etc.

To override and always open in the OS app — even for EPUB — click **Open in OS** in the book detail panel.

xcalibre still indexes the text and metadata of all these formats for search and organisation purposes; you just read them in your preferred app.

---

## 8. Metadata Editing

### Opening the editor

Click **Edit Metadata** in the book detail panel. The metadata editor opens as a modal.

### Editable fields

| Field | Type | Notes |
|-------|------|-------|
| **Title** | Text | Plain text. The sort key is auto-generated (articles like "The", "A", "An" are moved to the end). |
| **Authors** | Text | Enter multiple authors separated by commas or one per line. The first author's name is used for sort key generation (Last, First order). |
| **Publisher** | Text | Publisher name |
| **Publication date** | Text | ISO format preferred: `YYYY-MM-DD` or `YYYY`. Partial dates are accepted. |
| **Series** | Text | Series or sequence name |
| **Series index** | Number | Position in series. Decimals supported: `1.5` for a novella between books 1 and 2. |
| **Rating** | 0–5 | Star rating. 0 = unrated. |
| **Description** | Text | Synopsis or back-cover text. Multi-line. |
| **Tags** | Text | Comma-separated list of tags. Tags are created automatically if they don't exist. |
| **Identifiers** | Key=value pairs | One per line: `isbn=978-0-06-112008-4`, `asin=B000FC1PW0`, `goodreads=375802` |

Click **Save** to apply all changes. The library grid, search index, and sort order update immediately.

### Identifiers

Identifiers link a book to external databases. Standard identifier types:

| Type | Example value | Used for |
|------|--------------|---------|
| `isbn` | `978-0-06-112008-4` | ISBN-13 lookup |
| `isbn10` | `0-06-112008-2` | ISBN-10 lookup |
| `asin` | `B000FC1PW0` | Amazon ASIN |
| `goodreads` | `375802` | Goodreads book ID |
| `google` | `ASIN123` | Google Books ID |

You can add any custom identifier type you like (`openlibrary`, `barnesnoble`, etc.).

### Bulk metadata editing

Select multiple books (using checkboxes), then click **Edit Metadata** in the bulk action bar. The same editor opens. Rules for bulk editing:

- Fields you fill in are applied to **all selected books**.
- Fields you leave blank are **not changed** for any book.
- This lets you, for example, set a tag on 50 books at once while leaving all other metadata untouched.

**Rating**, **Series index**, and **Series name** can be bulk-applied this way. Be careful with Title and Authors in bulk mode — you probably want to leave those blank.

### Cover replacement

In the metadata editor, click the cover image to open a file picker. Select any image file (JPEG, PNG, WebP, etc.). xcalibre resizes it to fit within 500 × 750 pixels and stores a JPEG copy. The original image file is not modified.

To remove a custom cover and regenerate it from the book file, use **Re-ingest**.

### Sort keys

xcalibre maintains separate sort keys for title and primary author:

- **Title sort**: "The Lord of the Rings" → "Lord of the Rings, The"
- **Author sort**: "J.R.R. Tolkien" → "Tolkien, J.R.R."

These are generated automatically when you save metadata. They are used when sorting the library by title or author, so books sort sensibly rather than under "The" or "A".

---

## 9. Collections

Collections are manually curated groups of books. A book can belong to any number of collections simultaneously.

### Creating a collection

1. In the sidebar, click the **+** button next to "Collections".
2. Type a name and press Enter.

The collection appears in the sidebar immediately.

### Adding books to a collection

**Method 1 — Selection:** Select one or more books in the library, then click **Add to Collection** in the bulk action bar. Choose the target collection from the dropdown.

**Method 2 — Detail panel:** Open a book's detail panel, go to the **Details** tab, and use the Collections section to add it to one or more collections.

**Method 3 — Right-click:** Right-click a book card in the grid and choose **Add to Collection**.

### Viewing a collection

Click the collection name in the sidebar. The library grid updates to show only that collection's books. The search bar and filter bar continue to work within the collection view.

Click **All Books** at the top of the sidebar to return to the full library.

### Removing a book from a collection

Right-click the book in the library grid and choose **Remove from [collection name]**. Or open the detail panel and remove it from the Collections section there.

### Deleting a collection

Right-click the collection name in the sidebar and choose **Delete**. This deletes the collection and its membership records. No books are deleted.

### Collection book count

The sidebar shows the number of books in each collection next to its name, updating in real time as you add or remove books.

---

## 10. Virtual Libraries

Virtual libraries are saved queries that work like automatic, dynamic collections. The books shown in a virtual library update automatically whenever the underlying library changes.

Unlike collections (which you manage manually), virtual libraries are defined by a search expression and re-run every time you open them.

### Creating a virtual library

1. Click **+** next to "Virtual Libraries" in the sidebar, or go to **View → Virtual Libraries → New**.
2. Fill in:
   - **Name** — displayed in the sidebar
   - **Search expression** — the filter query (see below)
   - **Sort field** — title, author, series, pubdate, rating, or last_opened
   - **Sort direction** — ascending or descending
3. Click **Save**.

### Search expressions

Virtual library expressions use the same syntax as the search bar:

```
tag:unread                         → all unread books
tag:unread rating:>=3              → unread books rated 3 or higher
author:asimov pubdate:>1950        → Asimov after 1950
series:"Foundation"                → the Foundation series
format:epub NOT tag:read           → EPUBs you haven't read
language:en tag:science            → English-language science books
```

**Available operators**

| Operator | Example | Meaning |
|----------|---------|---------|
| (space) | `tolkien fantasy` | AND (both terms must match) |
| `NOT` | `tag:fiction NOT tag:read` | Exclude |
| Field prefix | `author:tolkien` | Search specific field |
| `>=`, `<=`, `>`, `<` | `rating:>=4` | Numeric comparison |
| Quoted string | `series:"The Expanse"` | Exact phrase |

### Managing virtual libraries

Right-click a virtual library in the sidebar to:
- **Edit** — change the name, expression, or sort order
- **Delete** — permanently remove the virtual library (no books are affected)

---

## 11. Bookmarks

Bookmarks mark named positions within an EPUB. They are stored with a precise CFI position and an optional label.

### Adding a bookmark

While reading, click the **bookmark icon** in the reader toolbar, or press **`B`**. A dialog appears asking for an optional label. If you skip the label, the bookmark is saved with the position description auto-generated from the CFI.

### Viewing bookmarks

Click the **Bookmarks panel** icon in the reader toolbar (bookmark ribbon icon). The panel opens on the right side of the reader, listing all bookmarks for the current book in position order:

- Bookmark label (or position description)
- Chapter reference

Click any bookmark to jump to that position in the book.

### Deleting a bookmark

Click the trash icon next to a bookmark in the Bookmarks panel.

### Bookmarks vs. annotations

Bookmarks are lightweight position markers with a label. [Annotations](#12-annotations-and-highlights) are associated with specific selected text (highlights) and can have a full note attached.

---

## 12. Annotations and Highlights

Annotations let you mark up EPUB text — highlight passages in colour and attach written notes to them.

### Highlighting text

1. In the reader, click and drag to select a passage of text.
2. A small toolbar appears above the selection.
3. Click a colour to create a highlight: **yellow**, **green**, **blue**, or **pink**.

The highlight is saved immediately with a CFI pointing to the exact position of the selected text.

### Adding a note to a highlight

After creating a highlight, click on the highlighted text. A popup appears. Click **Add Note** (or the pencil icon). Type your note and click **Save**.

Notes support multi-line text. They are displayed in the Annotations panel alongside the highlighted excerpt.

### Viewing annotations

Click the **Annotations panel** icon in the reader toolbar (speech bubble icon). The panel lists all annotations for the current book, sorted by their position in the text:

For each annotation:
- **Colour indicator**
- **Selected text** (the highlighted passage)
- **Your note** (if one was added)
- **CFI position reference**

Click any annotation to jump to that position in the book.

### Editing an annotation note

In the Annotations panel, click the pencil icon next to any annotation. Edit the note text and click **Save**.

### Deleting an annotation

Click the trash icon next to an annotation in the panel. The highlight is removed from the text and the annotation record is deleted.

### Annotation types

Internally, xcalibre distinguishes three annotation types:

| Type | Created by |
|------|-----------|
| `highlight` | Selecting text and choosing a colour |
| `note` | Highlight with a text note attached |
| `bookmark` | Reader bookmarks (see Section 11) |

### Syncing annotations

If you have xcalibre-server configured, click **Sync Annotations** in the Annotations panel toolbar to push all unsynced annotations to the server. Annotations are marked as synced after a successful push and will not be re-sent unless modified.

---

## 13. Notes

Notes are freeform documents attached to a book. Unlike annotation notes, they are not tied to a specific position in the text — use them for reading journals, summaries, research notes, or anything else.

### Creating a note

1. Open a book's detail panel.
2. Click the **Notes** tab.
3. Click **New Note**.
4. Enter a title and write the body text.
5. Click **Save**.

Notes support rich text (bold, italic, lists, links) stored as HTML internally. Plain-text view is also available.

### Viewing and editing notes

All notes for the current book are listed in the Notes tab, sorted by last modified date. Click any note to open it for editing.

### Searching notes

The search bar in the Notes tab searches both title and body text of all notes for the current book using full-text search.

### Deleting a note

Click the trash icon next to the note in the Notes list, or use the delete button inside the note editor.

### Saving AI responses as notes

After an AI conversation (see [Section 15](#15-ai-assistant)), click **Save as Note** below any AI response. The response is automatically added as a new note for that book, titled with the original query.

---

## 14. Similar Books

The **Similar Books** panel shows books in your library that are related to the current book.

### Opening the panel

Click **Similar Books** in the book detail panel, or click the similar-books icon while reading.

### Similarity criteria

xcalibre calculates similarity using up to four factors, each of which can be toggled on or off:

| Factor | Description |
|--------|-------------|
| **Same author** | Books sharing at least one author name |
| **Same series** | Books in the same series |
| **Shared tags** | Books with overlapping tags |
| **Same language** | Books in the same language |

Each matched factor increases the similarity score. Results are sorted by score, most similar first. Up to 10 results are shown by default.

### Using the panel

Click any result in the Similar Books panel to open that book's detail panel.

---

## 15. AI Assistant

xcalibre includes an AI assistant that can discuss, summarise, and answer questions about any book in your library. It uses the extracted text of the book as context, so it can reference specific passages.

### Setting up an AI provider

Before using the AI features, configure a provider:

1. Click **Settings** → **AI Provider** (or click the AI settings icon in the AI chat panel).
2. Select your provider from the dropdown:

| Provider | Requires key | Default base URL |
|----------|-------------|-----------------|
| **Ollama** | No | `http://localhost:11434` |
| **LM Studio** | No | `http://localhost:1234` |
| **OpenAI** | Yes | `https://api.openai.com/v1` |
| **Google Gemini** | Yes | `https://generativelanguage.googleapis.com/v1beta` |
| **OpenRouter** | Yes | `https://openrouter.ai/api/v1` |

3. For Ollama or LM Studio: verify the base URL matches where your local server is running. Click **Test Connection** to confirm xcalibre can reach it — the test lists all available models.
4. For OpenAI, Gemini, and OpenRouter: enter your API key. The key is stored in your OS keychain (macOS Keychain / Windows Credential Manager) — it is never written to disk, never logged, and never sent to xcalibre-server.
5. Enter the model name you want to use (or select from the dropdown after a successful connection test).
6. Optionally set an **embed model** for providers that support semantic embeddings.
7. Click **Save**.

### Starting a conversation

1. Open a book in the library view (click its card to open the detail panel).
2. Click **Discuss with AI** or the AI panel icon.
3. Type a question and press Enter (or click Send).

xcalibre automatically selects the 5 most relevant text chunks from the book and includes them as context in the request. The assistant's response is grounded in the actual book content.

### Quick actions

Five pre-built prompts appear at the top of the AI panel for common tasks:

| Button | Prompt sent |
|--------|------------|
| **Summarize** | "Give me a concise summary of this book." |
| **Chapters** | "What are the main chapters or sections?" |
| **Read Next** | "Based on this book, what should I read next?" |
| **Universe** | "Tell me about the world and universe of this book." |
| **Series** | "Is this part of a series? What's the reading order?" |

Click any button to send that prompt immediately.

### Citations

When the AI response references a passage from the book, it may include citation markers. These are displayed below the response with the relevant text excerpts, allowing you to see exactly which parts of the book informed the answer.

### Conversation history

Your conversation history is maintained for the current session. The assistant remembers earlier messages in the conversation and can answer follow-up questions that refer to previous exchanges.

To start a fresh conversation, click the **Clear** button at the top of the AI panel.

### Reasoning mode

For models that support extended thinking (such as Claude claude-sonnet-4-6):

Click the **Reasoning budget** selector in the AI panel toolbar:

| Setting | Effect |
|---------|--------|
| **None** | No extended reasoning. Fastest response. |
| **Low** | Brief reasoning pass. |
| **Medium** | Moderate reasoning. Better for complex questions. |
| **High** | Extended reasoning. Slowest but most thorough. |

### Copying the conversation

Press `Ctrl+Shift+A` (Windows) / `Cmd+Shift+A` (macOS) to copy the entire conversation (all user questions and assistant responses) to your clipboard as plain text.

### Compacting context

Long conversations accumulate context that can slow down responses or exceed model limits. Press `Ctrl+Shift+K` / `Cmd+Shift+K` to compact the context: xcalibre summarises earlier messages while preserving the most recent exchange.

### Saving responses as notes

Click **Save as Note** below any AI response. The response is saved as a new note attached to the current book (see [Section 13 — Notes](#13-notes)).

---

## 16. Bulk Actions

The bulk action bar appears at the bottom of the screen whenever one or more books are selected.

### Selecting books

- **Click a checkbox** (top-right of a book card) to select that book.
- **`Ctrl+A` / `Cmd+A`** — select all books currently visible (respects active search and filters).
- **Shift-click** — select a range of books between the last clicked and the current click.
- Click the **×** in the bulk action bar to clear the selection.

### Available actions

**Edit Metadata**
Opens the bulk metadata editor. Fill in only the fields you want to change across all selected books. Empty fields are left untouched. See [Section 8 — Metadata Editing](#8-metadata-editing).

**Convert / Tweak EPUB**
Opens the [format conversion dialog](#17-format-conversion). If all selected books are EPUB, the button reads "Tweak EPUB" (in-place EPUB cleanup). Otherwise it reads "Convert".

**Repair**
Attempts to fix common data problems for selected books: missing authors, title, sort fields, cover, or broken file paths. See [Section 25 — Library Maintenance](#25-library-maintenance).

**Re-ingest**
Resets selected books to `PENDING` and re-runs the full import pipeline (detect → validate → metadata → cover → text → search index). Use this after:
- Modifying a book file externally (e.g. editing an EPUB in another tool)
- A failed or incomplete initial import
- Wanting to refresh metadata from a file you've updated

Re-ingest is non-destructive: existing metadata you've edited manually is overwritten by what is extracted from the file.

**Export Metadata (CSV)**
Downloads a CSV file containing metadata for all selected books. Columns: `id, title, authors, format, publisher, series, series_index, pubdate, description`. All fields are properly quoted; descriptions containing commas, quotes, or newlines are handled correctly.

**Delete**
Permanently removes selected books from the library. This deletes:
- The database record
- The cover image file
- All tags, identifiers, annotations, notes, bookmarks, reading sessions, and full-text index entries for those books

**The original book files on disk are not deleted.** xcalibre only removes its own records.

A confirmation dialog shows the count of books to be deleted before proceeding.

---

## 17. Format Conversion

xcalibre can convert books between formats using its built-in conversion engine.

### Opening the conversion dialog

**From the detail panel:** Click **Convert** on a book's detail panel.

**From bulk actions:** Select books and click **Convert** in the bulk action bar.

### Choosing an output format

The conversion dialog shows a dropdown with all supported output formats, organised into two groups:

**Standard formats**
TXT · HTML · DOCX · PDF · MOBI · KEPUB · FB2 · RTF · HTMLZ

**Legacy formats**
LRF · PDB · PML · RB · SNB · TCR

Select a format and click **Convert**. xcalibre shows a progress indicator while the conversion runs.

On success, the output file path is shown. Converted files are saved to your Downloads folder by default.

### EPUB tweaking

For EPUB files, the "Convert" button may read "Tweak EPUB". Tweaking performs in-place fixes to an EPUB without changing its format:
- Validates and corrects OPF metadata
- Cleans up malformed HTML content files
- Normalises the spine and manifest

Use Tweak when an EPUB has display issues in the reader but you don't want to convert it to a different format.

### Conversion limitations

- Conversion is performed from the book's extracted text and structure. Some formatting and layout may not survive the conversion perfectly.
- Complex EPUB layouts (multi-column, heavy CSS) may be simplified in the output.
- DRM-protected files cannot be converted.

---

## 18. Export

### Export metadata for selected books

Select books, then click **Export Metadata (CSV)** in the bulk action bar. A save dialog opens.

The CSV file contains one row per book with these columns:

```
id, title, authors, format, publisher, series, series_index, pubdate, description
```

All text fields are properly CSV-escaped: fields containing commas, double-quotes, or newlines are wrapped in double-quotes with internal quotes doubled.

### Export full library as CSV

Go to **File → Export Library → CSV**. Exports all books in the library with the same column set as the per-book CSV above.

### Export full library as HTML

Go to **File → Export Library → HTML**. Produces a styled HTML page with a table listing all books (title, authors, format, series). Can be opened in any browser, printed, or shared.

---

## 19. Backup and Restore

### Creating a backup

Go to **File → Backup Library**, or find the Backup option in the Settings area.

The backup dialog offers one option:
- **Include book files** — when checked, the archive includes copies of all book files referenced by the library (not just the database and covers). This makes the backup fully self-contained but significantly larger.

Click **Create Backup**. A save dialog opens. The backup is saved as a ZIP archive containing:
- The `jobs.db` database
- All cover images
- (If "Include book files" is checked) All book files referenced by the library

### Restoring from a backup

Go to **File → Restore Library**, or find the Restore option in the Settings area.

Select a backup ZIP file. xcalibre reads the archive and imports all books from the backup into the current library. Restore is **additive** — it merges the backup into the current library. It does not wipe existing data.

The restore result shows how many books were restored from the backup.

---

## 20. Custom Columns

Custom columns extend xcalibre's per-book data model with your own fields. You can track anything not covered by the built-in fields: reading priority, loan status, your personal rating scale, purchase date, format quality, and so on.

### Creating a custom column

1. Open **Settings → Custom Columns**.
2. Click **Add Column**.
3. Fill in:

| Field | Description |
|-------|-------------|
| **Name** | Internal identifier (lowercase, no spaces, e.g. `read_next`) |
| **Label** | Display name shown in the UI (e.g. "Read Next") |
| **Type** | Data type (see below) |
| **Show in grid** | Toggle to display this column in the library list view |

**Column types**

| Type | Description | Example values |
|------|-------------|----------------|
| `text` | Freeform string | "borrowed from library", "signed copy" |
| `number` | Numeric value | `42`, `3.5` |
| `bool` | Yes/No toggle | true / false |
| `date` | ISO date | `2026-01-15` |
| `rating` | Star rating (1–5) | `4` |
| `list` | Comma-separated list | "hardcover, first edition" |

4. Click **Save**.

### Setting values for a book

1. Open the book's detail panel.
2. Click the **Custom** tab.
3. Each custom column appears as an editable field. Fill in the value.
4. Changes are saved automatically on blur.

### Using custom columns in search

Custom column values can be included in virtual library expressions and advanced search once you know the column's internal name:

```
read_next:true
date_purchased:>2025-01-01
```

### Editing and deleting custom columns

Open **Settings → Custom Columns**. Click the edit icon to rename a column or toggle its visibility. Click the trash icon to delete it — this removes the column definition and all stored values for that column across all books.

---

## 21. EPUB Editor

The built-in EPUB editor allows direct editing of EPUB file contents without external tools.

### When to use it

- Fix a broken or malformed EPUB that doesn't display correctly in the reader
- Correct typos or errors in the text
- Update or replace metadata embedded in the OPF file
- Change or replace the cover image
- Edit CSS stylesheets to adjust formatting

### Opening the editor

In the book detail panel, click **Edit EPUB**. This button only appears for EPUB-format books.

xcalibre opens the EPUB file and parses its structure. The editor shell has two panels:

**Left panel — File tree**
All files inside the EPUB ZIP are shown as a tree:
- `META-INF/` — container definition
- `OEBPS/` (or `OPS/`, etc.) — content files, images, CSS
- Individual XHTML chapter files
- CSS stylesheets
- Images

Click any file to open it in the right panel.

**Right panel — Content editor**
A text editor showing the raw content of the selected file. For HTML and CSS files, this is editable source code. For images, a preview is shown.

### Editing content

Click any XHTML or CSS file in the file tree. The source appears in the editor. Make your changes directly in the editor. Changes are held in memory until you save.

### Editing metadata

Click the **Metadata** tab in the editor shell. Editable fields:
- **Title** — updates the OPF `<dc:title>` and xcalibre's library record simultaneously
- **Authors** — updates the OPF `<dc:creator>` entries

Other OPF fields (language, publisher, identifiers) can be edited by opening the OPF file directly in the file tree editor.

### Replacing the cover

Click **Set Cover** in the editor toolbar. Select an image file from your filesystem. xcalibre:
1. Adds the image to the EPUB archive
2. Updates the OPF `<guide>` and `<meta name="cover">` references
3. Updates xcalibre's library record cover thumbnail

### Saving

Click **Save** in the editor toolbar. All pending changes are written back to the EPUB file on disk. The library record (title, authors) is updated if you edited those fields.

After saving, the book is re-indexed for search to reflect any content changes.

### Closing without saving

Click **Close** (or **×**). Any changes made since the last save are discarded. The EPUB file on disk is unchanged.

---

## 22. Plugins

xcalibre supports plugins that extend its functionality. Plugins are distributed as ZIP files containing a compiled library (`.dylib` on macOS, `.dll` on Windows).

### Installing a plugin

1. Open **Settings → Plugins**.
2. Click **Install Plugin from ZIP**.
3. Select the plugin ZIP file.

xcalibre extracts the plugin, reads its metadata (name, version, API version, type), and registers it in the library. The plugin appears in the Plugins list.

### Enabling and disabling plugins

In the Plugins manager, toggle the **Enabled** switch next to any plugin. Disabled plugins remain installed but are not loaded.

### Uninstalling a plugin

Click the trash icon next to a plugin to uninstall it. xcalibre removes the plugin library file and its registration record.

### Plugin compatibility

Plugins declare an API version. xcalibre checks the API version at install time and will warn if there is a mismatch.

---

## 23. Reading Statistics

xcalibre automatically tracks time spent reading every book.

### How tracking works

A reading session starts when you open a book in the EPUB reader and ends when you:
- Close the reader (press Escape or click Back)
- Open a different book
- Close the xcalibre window

Each session records:
- Start and end timestamp
- Duration in seconds
- Progress percentage at start and end

Sessions are persisted to the database immediately on close, so they survive crashes.

### Viewing statistics

Open a book's detail panel and click the **Stats** tab. The panel shows:

**Summary**
| Metric | Description |
|--------|-------------|
| Total reading time | Sum of all session durations, shown in minutes |
| Sessions | Number of reading sessions |
| Estimated pages | Derived from word count at ~250 words/page |

**Session history**
Each session is listed with:
- Date and time
- Duration (minutes)
- Progress start → end (e.g. 12% → 18%)

### Reading progress in the library

The progress bar on each book card and in the detail panel shows the current `progress_percent` stored for that book. This is updated continuously while you read.

---

## 24. Multiple Libraries

xcalibre supports multiple independent libraries stored as separate database files. Each library is completely isolated — its own books, collections, virtual libraries, custom columns, and settings.

### Use cases

- Separate personal and work libraries
- A library for books you own vs. a library for books you're reviewing
- Per-user libraries on a shared machine

### Creating a new library

1. Click the **Library switcher** in the sidebar (shows the current library name).
2. Click **New Library**.
3. Enter a name.
4. Choose a location for the database file (the default is the xcalibre data directory).
5. Click **Create**.

xcalibre switches to the new (empty) library immediately.

### Switching libraries

Click the library name in the sidebar to open the switcher. All configured libraries are listed. Click any library to switch to it. xcalibre loads the database for that library.

### Setting a default library

The last-used library opens automatically at startup. There is no separate "set default" step.

### Deleting a library

In the library switcher, click the trash icon next to a library. This removes the library record from xcalibre. **The database file on disk is not deleted** — you must remove it manually if you want to free the disk space.

You cannot delete the currently active library.

### Copying a book between libraries

In the book detail panel, click **Copy to Library** (or right-click the book card). Select the destination library. xcalibre copies the book's database record (metadata, tags, identifiers, collections) to the destination library. The original book file is referenced from the new record — it is not duplicated on disk (unless you check "Move file").

---

## 25. Library Maintenance

### Integrity check

**Tools → Check Library Integrity** (or the Integrity menu item in the library)

Scans every book's recorded `local_path` and verifies the file exists on disk. Reports all books where the file is:
- Missing (deleted or not accessible)
- On a volume that is not currently mounted (external drive disconnected)

The report lists each affected book with its expected file path and the issue. Use this after:
- Moving a book file or folder to a new location
- Ejecting an external drive that contained books
- After a system migration or backup restore

### Repair

**Select books → Bulk Actions → Repair**, or **Tools → Repair Books**

The repair function attempts to fix common data quality problems. For each selected book it checks and corrects:

| Problem | Fix applied |
|---------|------------|
| Empty `authors` field | Re-extracted from the book file |
| Empty `title` field | Re-extracted from the book file |
| Missing `title_sort` | Regenerated from the title |
| Missing `author_sort` | Regenerated from the first author |
| Missing cover | Re-extracted from the book file |
| Broken `local_path` | Searched nearby; updated if found |
| Rating out of range | Clamped to 0–5 |

Repair is non-destructive: it only fills in missing or broken values. Fields you have manually set that are already valid are not changed.

### Re-ingest

**Select books → Bulk Actions → Re-ingest**

Re-runs the full import pipeline (stages 1–6: detect, validate, deduplicate, metadata, cover, text). Use this to:
- Refresh metadata after editing the book file externally
- Re-extract a cover from an updated version of a file
- Force re-indexing of the full text after a text extraction failure

Re-ingest overwrites metadata with what is found in the file. If you have manually edited metadata that you want to keep, use **Repair** instead, or re-apply your edits after re-ingesting.

### Dealing with missing files

If an integrity check reveals missing files:

1. **If you moved the files:** Use **Edit Metadata** to update the file path, or delete the book record and re-import from the new location.
2. **If the files are on an unmounted drive:** Reconnect the drive and run the integrity check again.
3. **If the files are permanently gone:** Delete the book records from xcalibre to keep your library tidy.

### Database location

The database is a standard SQLite file. You can open it with any SQLite browser (DB Browser for SQLite, TablePlus, etc.) for inspection.

| Platform | Path |
|----------|------|
| macOS | `~/Library/Application Support/xcalibre/jobs.db` |
| Windows | `%APPDATA%\xcalibre\jobs.db` |

---

## 26. Settings Reference

Open Settings from the gear icon in the toolbar.

### Reader

| Setting | Values | Default | Description |
|---------|--------|---------|-------------|
| Theme | Light / Dark / Sepia | Light | Reader background and text colour |
| Font family | Serif / Sans-serif / Monospace | Serif | Reader text typeface |
| Font size | 14–24 px | 18 px | Reader base font size |

Reader settings are stored in `localStorage` in the app's WebView storage. They persist across sessions and survive app restarts.

### AI Provider

| Setting | Description |
|---------|-------------|
| Provider | Ollama / LM Studio / OpenAI / Google Gemini / OpenRouter |
| Base URL | HTTP endpoint for the AI provider |
| API Key | Required for OpenAI, Gemini, OpenRouter. Stored in OS keychain. |
| Model | Model name or ID (e.g. `llama3`, `gpt-4o`, `claude-sonnet-4-6`) |
| Embed model | Embedding model for semantic search (optional) |

Use **Test Connection** after entering provider details to verify connectivity and list available models.

### Server (xcalibre-server)

| Setting | Description |
|---------|-------------|
| Server URL | Base URL of your xcalibre-server instance (labeled "Autolib URL" in the UI) |

Leave blank if you are not using xcalibre-server. All features work locally without it.

### Custom Columns

See [Section 20 — Custom Columns](#20-custom-columns).

### Plugins

See [Section 22 — Plugins](#22-plugins).

### About

The About section in Settings shows the current xcalibre version number.

---

## 27. Keyboard Shortcuts

### Global (library)

| Shortcut | Action |
|----------|--------|
| `Ctrl+F` / `Cmd+F` | Focus the search bar |
| `/` | Focus the search bar |
| `Escape` | Clear search and return to full library; or close open panel |
| `Ctrl+A` / `Cmd+A` | Select all books |
| `↑` `↓` | Navigate the book list |
| `Enter` | Open selected book |
| `Delete` / `Backspace` | Delete selected books (confirmation required) |
| `Ctrl+Shift+A` / `Cmd+Shift+A` | Copy AI conversation to clipboard |
| `Ctrl+Shift+K` / `Cmd+Shift+K` | Compact AI context |

### Reader

| Shortcut | Action |
|----------|--------|
| `→` or `↓` or `Page Down` | Next chapter |
| `←` or `↑` or `Page Up` | Previous chapter |
| `Escape` | Close reader, return to library |
| `B` | Add bookmark at current position |
| `T` | Cycle reading theme (Light → Dark → Sepia) |
| `Ctrl+=` / `Cmd+=` | Increase font size |
| `Ctrl+-` / `Cmd+-` | Decrease font size |

### Key format for custom integrations

xcalibre's keyboard system uses the following modifier+key format internally, useful to know if you are building plugins or scripts:
- Modifier keys: `ctrl`, `meta` (Cmd on macOS), `alt`, `shift`
- Combined: `ctrl+k`, `meta+f`, `ctrl+shift+k`
- Special keys: `escape`, `arrowleft`, `arrowright`, `arrowup`, `arrowdown`, `enter`, `delete`, `backspace`

---

## 28. Supported Formats

### Import and organisation

xcalibre can import and organise all of the following. "Heuristic" metadata means xcalibre scans the raw file bytes for recognisable patterns — accuracy depends on how the original file was created.

| Format | Full name | Metadata | Text search | Cover |
|--------|-----------|----------|------------|-------|
| EPUB | Electronic Publication | Full (OPF) | Yes | Yes |
| PDF | Portable Document Format | Full (Info dict + XMP) | Yes | Yes (embedded or first page) |
| MOBI | Mobipocket | Full (PalmDOC + EXTH) | Yes | — |
| AZW3 | Kindle Format 8 | Full (EXTH) | Yes | — |
| AZW4 | Kindle Print Replica | Heuristic | Yes | — |
| CBZ | Comic Book ZIP | — | — | Yes (first image) |
| CBR | Comic Book RAR | — | — | — |
| TXT | Plain text | — | Yes | — |
| FB2 | FictionBook 2 | Full (FictionBook XML) | Yes | — |
| HTML | HyperText Markup Language | Partial (`<meta>` tags) | Yes | — |
| HTMLZ | Zipped HTML | Partial | Yes | — |
| RTF | Rich Text Format | Partial (Info dict) | Yes | — |
| DOCX | Office Open XML | Full (core.xml) | Yes | — |
| ODT | OpenDocument Text | Full (meta.xml) | Yes | — |
| CHM | Compiled HTML Help | Heuristic | Yes | — |
| LRF | BBeB Book | Heuristic | Yes | — |
| LRX | Librie Reader XML | Heuristic | Yes | — |
| PDB | Palm Database | Heuristic | Yes | — |
| PML | Palm Markup Language | Heuristic | Yes | — |
| RB | RocketBook | Heuristic | Yes | — |
| SNB | Shanda Bambook | Heuristic | Yes | — |
| TCR | Psion Text Compressed | Heuristic | Yes | — |
| DJVU | DjVu (Déjà Vu) | Heuristic | Yes | — |
| LIT | Microsoft Reader | Heuristic | Yes | — |

### Built-in readers

| Format | Reader |
|--------|--------|
| EPUB | Built-in reader with themes, font control, annotations, bookmarks |
| CBZ | Built-in page-by-page comic viewer |
| All others | Opens in the OS default app for that format |

### Conversion output formats

xcalibre can convert books into any of these formats:

| Format | Notes |
|--------|-------|
| TXT | Plain text, no formatting |
| HTML | Single HTML file |
| DOCX | Microsoft Word |
| PDF | Portable Document Format |
| MOBI | Mobipocket (for older Kindle devices) |
| KEPUB | Kobo Enhanced EPUB |
| FB2 | FictionBook 2 |
| RTF | Rich Text Format |
| HTMLZ | Zipped HTML |
| LRF | Sony Reader |
| PDB | Palm Database Book |
| PML | Palm Markup Language |
| RB | RocketBook |
| SNB | Shanda Bambook |
| TCR | Psion Text Compressed |

### Format detection

xcalibre detects format from file content, not filename extension. A file named `book.pdf` that is actually an EPUB is detected as EPUB and processed correctly. This means:

- Renaming a file does not fool xcalibre
- Files without extensions are handled correctly
- Corrupt files whose content does not match any known format are rejected at the validation stage

---

## 29. Troubleshooting

### A book shows a broken icon or "file not found"

The book file has been moved, renamed, or deleted since it was imported. xcalibre records file paths at import time but does not copy your files.

**If you moved the file:**
- Open the book's detail panel → Edit Metadata → update the file path field
- Or delete the book record from xcalibre and re-import from the new location

**To find all affected books at once:**
- Run **Tools → Check Library Integrity**

### Metadata is wrong or incomplete

Three approaches, from least to most disruptive:

1. **Manual edit** — Open Edit Metadata, correct the fields by hand. Fastest for one or two fields.

2. **Re-ingest** — Select the book, use Bulk Actions → Re-ingest. This re-reads metadata from the file. Use this if the file itself has been corrected (e.g. you edited it in Calibre).

3. **Enrich from ISBN** — If the book has an ISBN identifier, open Edit Metadata and click **Enrich**. xcalibre fetches metadata from Open Library and Google Books and shows you suggestions to accept or reject.

### Search doesn't find a book I know is there

1. **Check the word count** — Open the book's detail panel. If word count shows 0, text extraction failed or hasn't run. Re-ingest the book.

2. **Check for quote characters in your query** — An unbalanced double-quote causes the search to return empty results. Try removing quotes from your query.

3. **Check field prefix** — If you used `author:` and the name contains punctuation (e.g. `author:O'Brien`), try searching without the prefix first.

4. **Clear filters** — Make sure the filter bar is not hiding the book. Click the × button in the filter bar to reset all filters.

### The EPUB reader shows blank or garbled content

**Blank content:** The EPUB's chapter file may use non-standard encoding or reference resources that weren't bundled. Try:
- Opening the EPUB in an external app to confirm the file itself is valid
- Using **Edit EPUB** to inspect the chapter file and its referenced CSS

**Garbled layout:** Many EPUBs assume specific fonts or fixed viewport sizes.
- Try switching themes (Light/Dark/Sepia) — the theme CSS sometimes overrides problematic EPUB CSS
- Increase the font size
- Use **Edit EPUB** → select the CSS file → look for `font-size`, `line-height`, or `width` rules set in absolute units (px or pt) and remove or adjust them

### A book fails to import

Before reporting a bug, check:

1. **File size** — xcalibre rejects files over 500 MB
2. **File integrity** — Try opening the file in another application to confirm it isn't corrupted
3. **Format support** — Check the [Supported Formats](#28-supported-formats) table
4. **Duplicate** — If the file was previously imported, the SHA-256 deduplication will silently skip it. Check your library for the book under a different title.

If the import fails with an error, the message is stored in the database. To see it: hover over the book's status indicator in the detail panel, or check the library filtered by Status = FAILED.

### The app is slow with a large library

- **Use search instead of scrolling** — the search bar is fast even on 50,000-book libraries
- **Use collections or virtual libraries** — browsing 500 books is faster than browsing 10,000
- **Ensure sort fields are populated** — run **Tools → Repair Books** to regenerate missing `title_sort` and `author_sort` fields, which speeds up sorting significantly
- **Simplify virtual library expressions** — complex expressions with multiple NOT clauses are evaluated in full on every load

### The AI assistant isn't responding

1. **Check provider config** — Open Settings → AI Provider and click **Test Connection**
2. **For Ollama/LM Studio** — Make sure the local server is running. Ollama: `ollama serve`. LM Studio: start the local server from its interface.
3. **For cloud providers** — Verify your API key is correct. The key is stored in the OS keychain — if it shows `••••••••`, a key is saved. If the connection test fails, re-enter it.
4. **Check the model name** — The model must be pulled/available on the local server, or valid for the cloud provider. Use the dropdown after a successful Test Connection to pick from available models.

### Annotations or bookmarks aren't appearing in the reader

Annotations and bookmarks are stored by book ID and CFI position. If you re-imported a book (which changes its book ID), existing annotations are not automatically re-linked to the new record.

To avoid this: use **Re-ingest** (which updates the existing record) rather than deleting and re-importing (which creates a new record).

### xcalibre crashed

Check the crash log:

| Platform | Log path |
|----------|---------|
| macOS | `~/Library/Application Support/xcalibre/crash.log` |
| Windows | `%APPDATA%\xcalibre\crash.log` |

Include the full contents of this file when reporting a bug.

### Reporting bugs

Open a bug report at the xcalibre issue tracker. Include:
- xcalibre version (visible in Settings → About)
- Your operating system version
- Steps to reproduce the problem
- The crash log if the app crashed
- The error message if an operation failed (copy it from the error dialog)

---

*xcalibre User Guide — v1.0.0*  
*Last updated: 2026-05-12*
