# xCalibre — Architecture Decisions

Recorded: 2026-05-11

These decisions were made after gap analysis against Calibre and govern all
future phase planning.

---

## D1 — AI location: **Desktop-native (Option A)**

The Tauri app holds API keys and calls AI providers directly from
Rust/TypeScript. Works offline with Ollama. No server dependency for AI.
First provider to implement: Ollama (local, no API key, auto-detects model
thinking capability).

## D2 — Ebook editor fidelity: **Level C — Full Calibre parity**

Target: syntax-highlighted editor (CodeMirror/Monaco in WebView), visual diff,
TOC editor, font management, EPUB validation, spell check, checkpoint saves.
Estimated 12–16 weeks. Requires D3 (EPUB manipulation library) and D7
(spell check) first.

## D3 — EPUB manipulation: **Build native Rust library (Option A)**

No runtime dependency. Full control. Estimated 6–10 weeks for a usable
library covering container management, CSS parsing, cover/font/image handling,
splitting, validation subsystem, and spell check hooks. Gates D2 and D4.

## D4 — Conversion output: **Native Rust per-format (Option A)**

No runtime dependency. ~4–6 weeks per format. Priority order (see secondary
decisions below). Gates each format on D3 being available for EPUB container
operations.

## D5 — Plugin architecture: **Rust ZIP-installable plugins (Option A)**

Plugins are compiled Rust dynamic libraries distributed as ZIP files and
loaded via `libloading`. The plugin SDK crate defines stable `extern "C"`
trait-object vtables with a `PLUGIN_API_VERSION` guard. See secondary
decision S1 for ABI stability approach.

## D6 — Multiple libraries: **Independent local libraries (Option B)**

The desktop app manages multiple local SQLite databases. Each has its own
sync relationship with a server workspace. Requires a library switcher UI
and rework of `config.rs` and startup flow. Must be implemented early.

## D7 — Spell check: **OS-native (Option A)**

macOS: `NSSpellChecker` via Tauri plugin. Windows: Windows Spell Check API
via Tauri plugin. Zero bundled dictionaries. Platform-specific bindings.
Note: OS-native only covers the WebView layer. The Rust-side pipeline (polish
toolkit spell check) requires a separate solution — see secondary decision S2.

## D8 — Scope expansions

| Item | Decision |
|------|----------|
| Rich notes on books (author/tag HTML notes) | **In scope** |
| OPF sidecar write-back (Calibre round-trip) | **In scope** |
| KFX format support | **In scope** |
| Virtual libraries / boolean-gated saved searches | **In scope** |
| Similar books | **In scope** |
| Hierarchical tag browser | **Optional** — flat filter bar is the baseline; tree view is a stretch goal |
| Field-scoped boolean search parser | **In scope** (required foundation for virtual libraries) |

---

## Secondary Decisions

These decision points emerged from the primary decisions above.
All resolved 2026-05-11.

### Summary

| Decision | Choice |
|----------|--------|
| S1 Plugin ABI stability | **S1-A** — `extern "C"` vtables + strict semver |
| S2 Rust pipeline spell check | **S2-A** — no spell check in Rust pipeline |
| S3 Conversion format scope | **All tiers** — implement every format, Tier 1→4 priority order |
| S4 PDF output approach | **S4-B** — Tauri WebView print API |
| S5 KFX implementation depth | **S5-A** — full text extraction + metadata (4–6 weeks) |
| S6 New library on-disk layout | **S6-C** — configurable per library (default in-place; opt-in managed) |
| S7 AI text context source | **S7-D** — RAG: chunk `job_text` into segments, embed via sqlite-vec, retrieve top-K chunks per query |
| S8 Similar books algorithm | **S8-A** — metadata similarity (shared tags, series, author, publisher) |

---

### S1 — Plugin ABI stability approach

**Decision: S1-A**

Rust has no stable ABI. `extern "C"` vtables work but any change to a trait
breaks all existing plugins. Options:

- **S1-A** ✅ — `extern "C"` vtables + strict semver for the plugin SDK crate.
  Plugins must recompile against the matching SDK version. Simple; xcalibre
  embeds the expected `PLUGIN_API_VERSION` and refuses mismatches.
- **S1-B** — Use the `abi_stable` crate (stable vtables via `RBox`/`RStr`
  wrappers). More complex but allows forward-compatible vtable evolution.
  Migrate to this if plugin ecosystem grows.

---

### S2 — Rust-side spell check in the polish toolkit

**Decision: S2-A**

D7 (OS-native) covers the WebView (metadata editor, ebook editor UI).
The Rust-side polish pipeline is headless and does not spell-check. Options:

- **S2-A** ✅ — Accept no spell check in the Rust pipeline. Spell checking is
  always user-facing (WebView); the pipeline only processes, never checks.
- **S2-B** — Add `hunspell-rs` crate only for the polish pipeline. Bundles
  one default dictionary (~5 MB). OS-native still used in WebView.

---

### S3 — Conversion format scope and priority order

**Decision: All tiers. Implement every format in Tier 1→4 priority order.**

| Tier | Formats | Rationale |
|------|---------|-----------|
| 1 (high) | TXT, HTML, DOCX | Highest demand; TXT/HTML are trivial; DOCX uses `docx-rs` crate |
| 2 (medium) | PDF, MOBI/AZW3 | PDF via S4-B; MOBI needs Kindle-compatible output |
| 3 (low) | FB2, RTF, HTMLZ | Moderate complexity |
| 4 (niche) | LRF, PDB, PML, RB, SNB, TCR, LIT | Legacy formats; built last |

All 22 phases including Tier 4 formats are in scope.

---

### S4 — PDF output approach

**Decision: S4-B**

Pure-Rust HTML→PDF rendering does not have a mature crate. Options:

- **S4-A** — Use `wkhtmltopdf` as a bundled binary (LGPL, ~15 MB). Adds a
  bundled binary dependency.
- **S4-B** ✅ — Use headless Chromium/WebKit via Tauri's existing WebView.
  Load EPUB spine with print CSS applied, trigger `window.print()` → PDF.
  No new binary dependency.
- **S4-C** — Defer until a pure-Rust renderer matures.

---

### S5 — KFX implementation depth

**Decision: S5-A — full implementation (4–6 weeks)**

KFX is a complex container format (SQLite-based `.kfx` + sidecar `.kfx-zip`).

- **S5-A** ✅ — Full text extraction + metadata (matches every other supported
  format). Implement KFX container parser from scratch in Rust. 4–6 weeks.
- **S5-B** — Metadata only + best-effort heuristic text recovery. 1–2 weeks.

---

### S6 — Multiple-library new-library layout

**Decision: S6-C — configurable per library**

When the user creates a brand-new local library (D6), what is the on-disk
layout for book files?

- **S6-A** — Process-in-place (current behaviour): xcalibre records the
  existing file path; does not move or copy files.
- **S6-B** — Adopt Calibre's `<Author>/<Title [ID]>/` layout: xcalibre copies
  files into a managed folder tree.
- **S6-C** ✅ — Configurable per library: new libraries default to in-place;
  user can opt into managed `<Author>/<Title [ID]>/` layout during library
  creation. Managed layout is required for full OPF write-back round-trip
  compatibility (D8).

Library creation wizard must present the layout choice and explain the
trade-off.

---

### S7 — AI text context source

**Decision: S7-D — RAG with sqlite-vec**

When the AI discusses a book (D1), what text does it send as context?

- **S7-A** — Use full `job_text` stored in the DB. Fast, zero re-processing.
  Risk: large books overflow context window.
- **S7-B** — Re-extract text on demand for freshness.
- **S7-C** — `job_text` with freshness check; re-extract if source file mtime
  is newer.
- **S7-D** ✅ — RAG pipeline: chunk `job_text` into overlapping segments at
  ingest time, embed each chunk via a local embedding model (Ollama
  `nomic-embed-text` or similar), store vectors in sqlite-vec. At query time,
  embed the user's message and retrieve the top-K most relevant chunks to send
  as context. Handles books of any length without truncation.

Implications for Phase 8:
- Ingest pipeline gains a chunking + embedding step (run after `job_text` is
  stored)
- sqlite-vec must be added as a `xcalibre-processing` dependency (already
  planned for xcalibre-server — align versions)
- `BookDiscussDialog` sends retrieved chunks rather than raw `job_text`
- Adds ~1–2 weeks to Phase 8 scope

---

### S8 — Similar books algorithm

**Decision: S8-A — metadata similarity**

- **S8-A** ✅ — Metadata similarity: match on shared tags, series, author,
  publisher. Cheap, no ML required.
- **S8-B** — Content similarity: TF-IDF or embedding-based similarity on
  `job_text`. Requires sqlite-vec.
- **S8-C** — Hybrid: metadata primary, content similarity as optional
  enhancement when xcalibre-server is connected. Follow-on to S8-A.
