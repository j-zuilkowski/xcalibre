# Calibre — Reference Feature Set

Generated: 2026-05-11 from `/Users/jonzuilkowski/Documents/localProject/archive/calibre/src/calibre/`

This documents Calibre's major subsystems for cross-reference with xCalibre.
Only features relevant to gap analysis are listed; build infrastructure,
packaging, and platform-specific shims are omitted.

---

## 1. Ebook Format Engine (`src/calibre/ebooks/`)

### 1.1 Format Readers/Writers
| Module | Formats | Notes |
|--------|---------|-------|
| `epub/` | EPUB 2/3 | OPF, NCX parsing, CFI implementation |
| `mobi/` | MOBI/AZW3 | Full reader (headers, index, markup, NCX), writer (MOBI6, MOBI8, AZW3) |
| `pdf/` | PDF | HtmlWriter, ImageWriter, pdftohtml, reflow, render engine |
| `comic/` | CBZ/CBR | Input plugin |
| `chm/` | CHM | Metadata + text reader |
| `djvu/` | DJVU | DJVU parsing, BZZ decoding |
| `docx/` | DOCX | Full reader (styles, fonts, footnotes, images, tables) + OOXML writer |
| `fb2/` | FB2 | FictionBook parser |
| `html/` | HTML | Input, metadata, to-ZIP conversion |
| `htmlz/` | HTMLZ | OEB to HTML conversion |
| `lit/` | LIT | LZX decompression, OPF/HTML mapping, reader + writer |
| `lrf/` | LRF/LRX | Full parser, HTML converter, font support |
| `odt/` | ODT | OpenDocument input |
| `pdb/` | PDB | eReader (130/202), PalmDOC, zTXT, Plucker, haodoo readers + writers |
| `pml/` | PML | PML converter |
| `rb/` | RB | RocketBook reader + writer |
| `rtf/` | RTF | Input + full rtf2xml pipeline (30+ modules) |
| `snb/` | SNB | SNB file + markup |
| `txt/` | TXT | Markdown, Textile, raw text processors |
| `textile/` | Textile | Textile functions + unsmarten |

### 1.2 Conversion (`ebooks/conversion/`)
- `plumber.py` — OEB-based conversion pipeline
- `plugins/` — 30+ input/output conversion plugins
  - Inputs: azw4, chm, comic, djvu, docx, epub, fb2, html, htmlz, lit, lrf, mobi, odt, pdb, pdf, pml, rb, recipe, rtf, snb, tcr, txt
  - Outputs: docx, epub, fb2, html, htmlz, lit, lrf, mobi, oeb, pdb, pdf, pml, rb, rtf, snb, tcr, txt
- `search_replace.py` — content search-and-replace during conversion
- `preprocess.py` — CSS/HTML preprocessing
- `config.py` — Conversion option configuration

### 1.3 OEB Polish Toolkit (`ebooks/oeb/polish/`)
- Container management, CSS parsing, cover manipulation, font embedding/subsetting
- Image management, book creation, spell checking (Hunspell)
- ToC editor, book splitting, text-to-speech integration
- Check subsystem: CSS validation, font validation, image validation, link checking, OPF validation

### 1.4 OEB Transforms (`ebooks/oeb/transforms/`)
- `cover.py`, `split.py`, `jacket.py` — book structure transforms
- `embed_fonts.py`, `subset.py` — font handling
- `flatcss.py` — CSS flattening
- `rasterize.py`, `rescale.py` — image transforms
- `metadata.py` — metadata injection

### 1.5 EPUB Reading (`ebooks/oeb/iterator/`)
- `book.py` — OEB book iterator
- `spine.py` — Spine traversal
- `bookmarks.py` — Bookmark management

### 1.6 Other
- `BeautifulSoup.py` — Forked HTML parser
- `covers.py` — Cover extraction from ebook files
- `css_transform_rules.py` — CSS transformation rule engine
- `html_transform_rules.py` — HTML transformation rule engine
- `hyphenate.py` — Hyphenation support
- `readability/` — Readability extraction

---

## 2. Metadata Engine (`src/calibre/ebooks/metadata/`)

### 2.1 Format Metadata Readers
| Module | Formats |
|--------|---------|
| `epub.py` | EPUB OPF |
| `pdf.py` | PDF info dict + XMP |
| `mobi.py` | MOBI EXTH headers |
| `docx.py` | DOCX |
| `fb2.py` | FB2 |
| `html.py` | HTML meta tags |
| `odt.py` | ODT |
| `rtf.py` | RTF |
| `lit.py` | LIT |
| `lrx.py` | LRX |
| `pdb.py` | PDB |
| `pml.py` | PML |
| `rb.py` | RB |
| `snb.py` | SNB |
| `ereader.py` | eReader PDB |
| `txt.py` | TXT |
| `rar.py` | RAR |
| `zip.py` | ZIP |
| `archive.py` | Generic archive |
| `topaz.py` | Topaz |
| `haodoo.py` | Haodoo |
| `kdl.py` | KDL |
| `kfx.py` | KFX |
| `extz.py` | ExtZ |
| `imp.py` | IMP |
| `plucker.py` | Plucker |

### 2.2 Metadata Sources (`metadata/sources/`)
- `amazon.py` — Amazon metadata (ISBN-based)
- `google.py` — Google Books
- `openlibrary.py` — Open Library
- `edelweiss.py` — Edelweiss
- `google_images.py` — Cover downloads
- `covers.py` — Cover download management
- `identify.py` — Book identification from metadata
- `search_engines.py` — Generic search engine metadata
- `worker.py` — Async metadata download worker
- `base.py` — Source plugin base class
- `update.py` — Built-in metadata source update
- `prefs.py` — Metadata source configuration
- `cli.py` / `test.py` — CLI + test harness

### 2.3 Metadata Book Model (`metadata/book/`)
- `base.py` — Core `Metadata` class
- `formatter.py` — Template-based formatting
- `json_codec.py` — JSON serialization
- `render.py` — HTML rendering
- `serialize.py` — OPF serialization

### 2.4 Other
- `author_mapper.py` — Author name mapping rules
- `tag_mapper.py` — Tag mapping rules
- `toc.py` — Table of Contents metadata
- `worker.py` — Download worker
- `search_internet.py` — Web search for metadata
- `xmp.py` — XMP metadata parser
- `xisbn.py` — xISBN service
- `opf.py` / `opf2.py` / `opf3.py` — OPF 2.x/3.x read/write
- `opf_2_to_3.py` — OPF version migration
- `meta.py` — Generic file metadata

---

## 3. Database Layer (`src/calibre/db/`)

| Module | Purpose |
|--------|---------|
| `backend.py` | Core DB implementation |
| `cache.py` | In-memory metadata cache |
| `view.py` | Library view (virtual table/join) |
| `write.py` | Write operations |
| `search.py` | Full search query parser (boolean, field-scoped) |
| `fields.py` | Field definitions |
| `adding.py` | Book ingestion |
| `categories.py` | Tags, authors, series, publishers |
| `covers.py` | Cover storage |
| `annotations.py` | Reader annotations |
| `fts/` | Full-text search (connect, pool, schema, text) |
| `notes/` | Rich notes (connect, export/import, schema) |
| `legacy.py` | Legacy DB support |
| `schema_upgrades.py` | Schema migration |
| `search.py` | Advanced search query language |
| `page_count.py` | Page count estimation |
| `backup.py` | Library backup |
| `restore.py` | Library restore |
| `copy_to_library.py` | Cross-library copy |
| `errors.py` | DB error types |
| `listeners.py` | DB event listeners |
| `locking.py` | DB concurrency |

### 3.1 DB CLI (`db/cli/`)
- 20+ CLI subcommands: add, remove, search, export, catalog, check, backup, etc.

---

## 4. GUI (`src/calibre/gui2/`)

### 4.1 Main Window
- `main.py` / `main_window.py` — Application entry, main window
- `ui.py` — UI layout
- `layout.py` — Panel layout management
- `bars.py` — Status bars, toolbars

### 4.2 Actions (50+)
| Action | File |
|--------|------|
| Add books | `actions/add.py` |
| Edit metadata | `actions/edit_metadata.py` |
| Convert books | `actions/convert.py` |
| View book | `actions/view.py` |
| Device management | `actions/device.py` |
| Delete books | `actions/delete.py` |
| Save to disk | `actions/save_to_disk.py` |
| Catalog | `actions/catalog.py` |
| Fetch news | `actions/fetch_news.py` |
| Polish books | `actions/polish.py` |
| Tweak EPUB | `actions/tweak_epub.py` |
| Preferences | `actions/preferences.py` |
| Help | `actions/help.py` |
| FTS search | `actions/fts.py` |
| Manage categories | `actions/manage_categories.py` |
| Copy to library | `actions/copy_to_library.py` |
| Choose library | `actions/choose_library.py` |
| Show book details | `actions/show_book_details.py` |
| Show quickview | `actions/show_quickview.py` |
| Annotate | `actions/annotate.py` |
| Browse annotations | `actions/browse_annots.py` |
| Browse notes | `actions/browse_notes.py` |
| LLM discuss book | `actions/llm_book.py` |
| Open book | `actions/open.py` |
| Plugin updates | `actions/plugin_updates.py` |
| Random pick | `actions/random.py` |
| Saved searches | `actions/saved_searches.py` |
| Similar books | `actions/similar_books.py` |
| Sort dialog | `actions/sort.py` |
| Store | `actions/store.py` |
| Tag mapper | `actions/tag_mapper.py` |
| Author mapper | `actions/author_mapper.py` |
| ToC edit | `actions/toc_edit.py` |
| Virtual libraries | `actions/virtual_library.py` |
| Mark books | `actions/mark_books.py` |
| Match books | `actions/match_books.py` |
| Embed metadata | `actions/embed.py` |

### 4.3 Viewer (`gui2/viewer/`)
- Full ebook reader: annotations, bookmarks, highlights, lookup, search, printing, TTS
- LLM integration (`llm.py`)
- Overlay, toolbars, keyboard shortcuts
- TOC panel, search panel
- Configurable by book format

### 4.4 Tweak Book / Ebook Editor (`gui2/tweak_book/`)
- Full code editor with syntax highlighting (HTML, CSS, XML, JavaScript, Python)
- Visual diff viewer
- Multi-file search & replace
- Spell check (Hunspell)
- ToC editor
- Font management
- Live CSS preview
- Check book (links, CSS, fonts, images, OPF validation)
- Reports (book statistics)
- Code completion
- Snippets
- Function replace
- Save with checkpoint history

### 4.5 Store Browser (`gui2/store/`)
- 30+ store plugins: Amazon (12 regions), Kobo, Google Books, Barnes & Noble, Gutenberg, etc.
- Search dialog with multi-store concurrent search
- Advanced search builder
- Results with covers, prices, DRM info
- Download to library

### 4.6 Preferences (`gui2/preferences/`)
- Adding, Behavior, Coloring, Columns, Conversion, Keyboard, Look & Feel
- Metadata sources, Plugins, Plugboard, Saving, Sending, Server
- Template functions, Toolbar, Tweaks

### 4.7 Metadata Editor (`gui2/metadata/`)
- Single metadata download
- Bulk metadata download
- PDF cover extraction
- Metadata diff viewer
- Rich widget set for all metadata fields

### 4.8 Library Views (`gui2/library/`)
- Book list with custom columns
- Cover grid view
- Cover flow view
- Annotations view
- Notes view

### 4.9 Other GUI Subsystems
- `tag_browser/` — Hierarchical tag browser sidebar
- `dialogs/` — 30+ dialog types (search, metadata, catalog, conversion, etc.)
- `convert/` — Conversion dialog with 20+ output format config panels
- `catalog/` — Catalog generation UI
- `chat_widget.py` — AI chat widget
- `llm.py` — LLM integration bridge
- `tts/` — Text-to-speech configuration (Piper, SpeechD)
- `fts/` — FTS search UI
- `jobs.py` — Background job queue + progress
- `keyboard.py` — Keyboard shortcut configuration
- `dnd.py` — Drag and drop

---

## 5. AI/LLM Integration (`src/calibre/ai/`)

| Module | Provider |
|--------|----------|
| `openai/` | OpenAI API |
| `google/` | Google AI (Gemini) |
| `ollama/` | Ollama (local) |
| `github/` | GitHub Copilot |
| `lm_studio/` | LM Studio |
| `open_router/` | OpenRouter (multi-provider) |
| `openai_compatible/` | Generic OpenAI-compatible API |
| `config.py` | Backend configuration |
| `prefs.py` | AI preferences management |
| `utils.py` | Shared AI utilities |

**Capabilities:** Streaming chat, text-to-text, text-to-image, TTS, embeddings.
Integrated into: main GUI chat widget, viewer LLM panel, "Discuss book" action.

---

## 6. Content Server (`src/calibre/srv/`)

- `standalone.py`, `embedded.py` — Server launchers
- `routes.py` — URL routing
- `ajax.py` — AJAX endpoints
- `books.py` — Book delivery
- `metadata.py` — Metadata endpoints
- `content.py` — Content serving
- `opds.py` — OPDS catalog
- `auth.py` / `users.py` — Authentication
- `cdb.py` — Change database
- `changes.py` — Real-time change notifications
- `web_socket.py` — WebSocket support
- `bonjour.py` — mDNS/Bonjour discovery
- `convert.py` — Server-side conversion
- `fts.py` — Server-side FTS
- `last_read.py` — Reading position tracking
- `render_book.py` — Book rendering to HTML
- `library_broker.py` — Multi-library management

---

## 7. Device Support (`src/calibre/devices/`)

30+ device drivers: Kindle, Kobo, Nook, Sony PRS, Cybook, Hanlin, iRiver, JetBook, Android, MTP (generic), USBMS (generic), folder device, and more.

---

## 8. News/Recipes (`src/calibre/web/feeds/`)

- Recipe framework (`recipes/collection.py`, `recipes/model.py`)
- 100+ built-in news source recipes
- Template-based news formatting
- Scheduler for auto-download

---

## 9. Plugin System (`src/calibre/customize/`)

- `__init__.py` — Plugin base classes
- `builtins.py` — Built-in plugin registry
- `conversion.py` — Input/Output conversion plugin API
- `profiles.py` — Device profiles
- `ui.py` — UI plugin API
- `zipplugin.py` — ZIP-based plugin installation

---

## 10. Utilities (`src/calibre/utils/`)

| Module | Purpose |
|--------|---------|
| `fonts/` | Font scanning, metadata, subsetting, SFNT/CFF tables |
| `formatter.py`, `formatter_functions.py` | Template language engine |
| `search_query_parser.py` | Boolean search query parser |
| `img.py`, `imghdr.py`, `magick/` | Image handling |
| `cleantext.py`, `smartypants.py`, `unsmarten.py` | Text cleanup |
| `titlecase.py` | Title casing |
| `wordcount.py` | Word counting |
| `hyphenation/` | Hyphenation dictionaries |
| `spell/` | Spell checking |
| `tts/`, `piper.py` | TTS integration |
| `iso8601.py`, `date.py` | Date/time handling |
| `filenames.py` | Safe filename generation |
| `config.py`, `config_base.py` | Configuration framework |
| `zipfile.py`, `localunzip.py`, `seven_zip.py` | Archive handling |
| `browser.py`, `https.py` | HTTP/certificate handling |
| `ipc/` | Inter-process communication (worker pool) |
| `lock.py` | File-based locking |
| `network.py`, `ip_routing.py` | Network utilities |
| `opensearch/` | OpenSearch description parsing |
| `exim.py`, `serialize.py` | Import/export |
| `translator/` | Content translation |
| `rapydscript.py` | RapydScript compiler |
| `mdns.py` | mDNS discovery |
| `shared_file.py`, `shm.py` | Shared memory |
| `mem.py` | Memory monitoring |
| `webengine.py` | WebEngine utilities |
| `terminal.py` | Terminal helpers |
| `trash/` | OS trash/recycle bin |
| `open_with/` | OS file associations |
