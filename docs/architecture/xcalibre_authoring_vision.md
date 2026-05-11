# xcalibre Authoring Vision

*Architectural discussion — 2026-05-11*

---

## TeX / LaTeX Integration

**Not recommended for native integration.**

- TeX is a system install (texlive/miktex), not a Rust library — adds a 2–4 GB runtime dependency
- Target user (general ebook reader) almost never encounters raw `.tex` files
- Academic users who do already have purpose-built tools (Overleaf, TeXstudio)
- If ever needed: thin `convert/latex.rs` shim shelling out to `pandoc --from latex --to epub`, identical pattern to `convert/pdf.rs` — no architectural complexity

---

## xcalibre as a First-Rate Authoring Program

### Context

Used alongside **xcalibre-server** (Rust/Axum ebook server with RAG memory endpoint) and **Merlin** (multi-LLM supervisor-worker AI system). The three-tool stack makes xcalibre uniquely positioned for AI-assisted authoring — the competitive moat is not features but Merlin integration.

### Core principle

Don't rebuild Scrivener. A minimal authoring shell with excellent AI assistance beats a feature-complete editor without it.

### The xcalibre-manuscript format

Introduce an intermediate directory project format as the canonical authoring source. Never author directly into EPUB.

```
my-book/
  manuscript.xcm          # project manifest (TOML)
  chapters/
    01-intro.md           # CommonMark + extensions
    02-rising.md
  assets/
    cover.jpg
  styles/
    print.css
    screen.css
  notes/                  # existing notes system (rmp11)
  output/
    build.epub
    build.pdf
```

**Why markdown source:**
- Human-readable, git-versionable
- Merlin can read and edit `.md` files natively without any adapter
- Pandoc/pulldown-cmark compile it
- One source → many output targets

### New crate: `xcalibre-manuscript`

Sits alongside `xcalibre-epub` and `xcalibre-ai`. Responsibilities:
- Parse `.xcm` TOML manifest
- Compile chapters → EPUB via `xcalibre-epub`
- Track word counts, chapter structure, cross-references
- Export targets (PDF via rmp15 pipeline, DOCX via rmp09)

### Author-mode UI

Separate view from the library browser — closer to Scrivener than to a file manager:
- **Outline panel**: drag-and-drop chapter reordering (writes to manifest)
- **Writing surface**: ProseMirror or Tiptap (already a dep from rmp11) with typewriter mode, focus mode, custom schema (headings, scenes, comments, tracked changes)
- **Compile panel**: one-click build to any output format
- **Target tracker**: word goal progress bars per chapter (rmp21 `ReadingProgressBar` reused)

Note: The rmp13 editor (CodeMirror on raw HTML) is for post-production fixes on imported books. Authoring needs this purpose-built prose surface.

### Compile step

Pandoc is the pragmatic answer. Handles footnotes, citations, cross-references, LaTeX math. Shell out to it — don't reimplement. Same pattern as PDF/Calibre in rmp15b.

### Database separation

- **Per-project SQLite**: manuscript metadata (word targets, chapter status, character sheets, scene cards) — lives alongside the project directory
- **Main library DB**: tracks the published artifact only
- The library DB records the compiled EPUB once it's "published"

### Version control

Build in `git init` on project creation. Diff views between drafts become trivial. This is something Scrivener has always done badly and is a genuine differentiator.

### xcalibre-server integration

- Published drafts served at `/draft/:book-id` for beta readers
- Webhook on compile → auto-push to server → shareable preview URL
- Reader comments/annotations flow back into xcalibre as notes (rmp11 `NoteRow`)

### Merlin integration

Merlin operates directly on the manuscript directory — plain `.md` files, its native environment:
- Context-aware editing: "Expand this scene", "rewrite chapter 3 in third person", "check continuity"
- RAG pipeline (rmp08) on the manuscript itself: answers "what did I say about the antagonist in chapter 2?" without re-reading the whole book
- Style guide enforcement: stored in xcalibre-server memory, Merlin checks drafts against it
- `include_fields` AI config (rmp08 `ai_config` table) extended for manuscript-specific context

### Phasing

1. `xcalibre-manuscript` crate — project model + compile pipeline (2–3 weeks)
2. Author shell UI — outline + writing surface + compile panel (4–6 weeks)
3. xcalibre-server draft publishing — serve previews, accept reader annotations (2–3 weeks)
4. Merlin manuscript tools — context-aware editing, continuity checks, style enforcement (ongoing)

---

## Bringing Any Format Into the Editor

### Two distinct pipelines

| Pipeline | Purpose | Fidelity requirement |
|----------|---------|----------------------|
| Ingest (existing) | Text + metadata for indexing and display | Lossy, one-way |
| Edit (new) | Structurally faithful editable EPUB | Round-trippable, structure-preserving |

The edit pipeline needs a new code path: `xcalibre-epub/src/import.rs`.

### ImportResult

```rust
pub enum ImportFidelity { High, Medium, Low, TextOnly }

pub struct ImportResult {
    pub container: Container,
    pub fidelity:  ImportFidelity,
    pub warnings:  Vec<String>,
}

// Each format importer signature:
pub fn import_for_editing(path: &Path) -> Result<ImportResult, ImportError>
```

### Format fidelity

| Format | Edit fidelity | Round-trip | Notes |
|--------|--------------|------------|-------|
| EPUB | Perfect | Native | Already done (rmp13) |
| FB2 | High | Yes | Best non-EPUB; clean XML → EPUB mapping 1:1 |
| DOCX | High (mammoth) | Lossy | Highest priority; mammoth JS lib is exceptional |
| HTML/HTMLZ | High | Yes | Near-trivial; HTML is already EPUB internals |
| MOBI (no DRM) | Medium | Lossy | NCX/TOC preserves chapter structure |
| KFX (no DRM) | Medium | No | Flat import; no clean chapter boundaries in SQLite schema |
| RTF | Low | Lossy | Only if author used Word heading styles correctly |
| PDF | Very low | No | Text extraction only; layout is irrecoverable natively |
| TXT | Text only | Trivial | Single-chapter EPUB; user adds structure manually |
| PML | Medium | Yes | `\c` and `\p` tags map cleanly to EPUB structure |
| LRF, PDB, RB, SNB, TCR | Text only | No | Native extraction; see visual reconstruction section |

### Copy-on-edit — never touch the original

```rust
pub struct EditorSession {
    pub editor:        EpubEditor,
    pub source_path:   PathBuf,       // original file
    pub source_format: SourceFormat,  // detected format
    pub edit_path:     PathBuf,       // working copy in app data dir
    pub from_import:   bool,
}
```

"Save" writes to `edit_path`. "Export" offers format choices including original format. "Save as EPUB" is always the default.

### DRM detection — hard stop

Before opening Amazon formats:
- **MOBI DRM**: inspect PalmDB header ExthRecord for encryption type
- **KFX DRM**: check for `KFX-IDX` fragment type alongside encrypted content fragments
- If DRM present: refuse with a clear message, offer no partial extraction

### Fidelity warning UI

Pre-edit dialog shown for all non-EPUB imports:
- Format detected
- Fidelity level (color-coded: green High / amber Medium / red Low or TextOnly)
- Specific warnings (e.g., "PDF: page layout will not be preserved", "RTF: heading styles may not be detected")
- "Open anyway" or "Cancel"

### Round-trip export

After editing, "Save as [original format]" uses the existing conversion pipeline (rmp09–rmp22). For formats where round-trip is inherently lossy (PDF, RTF), this option is grayed out or shown with a warning. Sensible default is always "Save as EPUB".

### Build priority

EPUB (done) → DOCX → FB2 → HTML/HTMLZ → MOBI → everything else

---

## Visual Reconstruction (Screenshot → Editable)

### The core question

Can you render a format visually, screenshot it, and reconstruct it as an editable document?

### When visual reconstruction beats native extraction

**PDF** — PDF's fundamental design is visual-first: it encodes where ink goes on a page, not document structure. The visual rendering IS the ground truth. Native text extraction reconstructs structure from layout coordinates — error-prone (column order, footnotes mixed into body). Rendering each page to an image and using vision to reconstruct it is genuinely superior.

**LRF** (Sony BBeB) — same logic; layout-first format.

**Scanned EPUBs/PDFs** (image-only, no text layer) — visual reconstruction is the only option.

### When native extraction is better

**KFX, PDB, PML, RB, SNB, TCR** — fundamentally text containers with structure in binary or markup. You already have the actual bytes. Screenshots would be a step backward — you'd be OCR-ing text you already have access to natively.

### Three reconstruction tiers

| Tier | Tool | Structure recovery |
|------|------|--------------------|
| Traditional OCR | Apple Vision `VNRecognizeTextRequest`, Tesseract | Text + rough paragraph breaks; no heading detection; tables are guesswork |
| Layout analysis | Azure Document Intelligence, AWS Textract | Headings, tables, lists, columns; cloud API with per-page cost |
| Vision LLM | Claude Vision, GPT-4V, Gemini Vision | Best: heading hierarchy, footnotes, captions, semantic roles; production-grade |

LlamaParse (from LlamaIndex) is built entirely on the Vision LLM tier — render PDF page → image → GPT-4V → structured markdown.

### Merlin integration pipeline

```
xcalibre (pdfium-render)
  → Vec<page_image>
    → Merlin (per page)
      → vision model (Claude/GPT-4V/Gemini)
        → structured markdown
          → assemble in page order
            → split on heading boundaries (chapter detection)
              → xcalibre-epub Container
                → editable EPUB
```

**Prompt template for each page:**
> "You are reconstructing a book page. Extract all text as structured markdown. Preserve heading hierarchy (use # ## ###), paragraph breaks, lists, footnotes (as [^1] markers), table structure (as markdown tables), and image captions. Do not infer or add content. Output only the markdown."

**Per-page approach**: One page per request gives the model best resolution and attention. Context window not wasted on other pages.

### New modules required

**`processing/src/render/pdf.rs`**
```rust
// Wraps pdfium-render crate
// Produces Vec<RgbaImage> from PDF path at configurable DPI
// 150 DPI for text, 300 DPI for image-heavy pages
```

**`processing/src/render/lrf.rs`**
```rust
// Shells out to Calibre's lrfviewer or headless renderer
// Same output interface as pdf.rs: Vec<RgbaImage>
```

**`src-tauri/src/commands/reconstruct.rs`**
```rust
// Orchestrates full pipeline:
// - calls render → page images
// - batches pages to Merlin (or xcalibre-ai factory directly)
// - reassembles markdown in page order
// - splits on heading boundaries for chapter inference
// - feeds into xcalibre-epub Container
// - returns editable EPUB path
// Runs as background Tauri command (not blocking UI)
```

### Practical caveats

**Cost**: GPT-4V ~$0.01/page → ~$3 for a 300-page book. Acceptable but surface to user. Local vision model (LLaVA, Qwen-VL via LM Studio) → free but lower quality.

**Speed**: 1–3 seconds/page → 5–15 minutes for 300 pages. Must run as background job. Progress indicator: "Reconstructing page N of M."

**Quality ceiling**: Cleanly typeset PDF (most ebooks) → minimal cleanup needed. Heavy LaTeX math and complex multi-column academic layouts are the main failure cases.

**Copyright**: UI note that reconstruction is for personal editing of legally owned books only.

**Most valuable immediate target**: PDF — both the format where visual reconstruction is genuinely superior AND the most common format where users want to edit content (academic papers, self-published PDFs, scanned books).

---

## Future Document Split

When implementation planning begins for each tool's specific role, consider splitting into:

- `xcalibre_authoring_and_editing.md` — authoring program, format import pipeline, visual reconstruction
- `merlin_document_intelligence.md` — vision reconstruction pipeline, manuscript-aware RAG, style guide enforcement, authoring assistance commands
- `xcalibre_server_publishing.md` — draft publishing endpoint, webhook on compile, reader annotation flow back to notes
