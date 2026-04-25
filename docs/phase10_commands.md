# Phase 10 — Remaining In-Scope Calibre Parity

> HOW TO USE: For every "Write `path`" line → call your file-write tool with that path and content.
> For every "Then run:" block → call your shell tool for each command.
> DO NOT print code as output. Write it to disk using your tools.
> Prerequisite: Phase 9 complete and all tests green.
> Status: ✅ complete

Phase 10 covers the remaining Calibre-parity work that is now explicitly in scope
for xCalibre. It does not include the deliberately excluded product areas:
USB / MTP device sync, built-in content server, or news recipes.

## Status

| Task | Title | Status |
|------|-------|--------|
| P10-T01 | CBZ / CBR completeness | ✅ |
| P10-T02 | Exotic format completion (CHM, LIT, LRF/LRX, PDB/PML/RB, SNB, TCR, AZW4, DJVU) | ✅ |
| P10-T03 | Conversion pipeline + EPUB tweak workflows | ✅ |
| P10-T04 | Full Calibre-style repair/editing workflows | ✅ |
| P10-T05 | Tests, fixture coverage, and GAP sync | ✅ |

---

## P10-T01

### CBZ / CBR Completeness

Scope:
- Treat comic archives as first-class library objects, not just lightweight viewer files.
- Expand archive metadata extraction so series, issue, page count, and cover heuristics are more consistent.
- Tighten detection and fallback behavior for malformed archives.
- Keep comic viewing and metadata handling aligned with the main library schema.

Acceptance:
- CBZ / CBR items are ingested, searchable, and browsable without special-case UI gaps.
- Comic metadata remains stable across re-ingest and sync.
- Viewer behavior is consistent for page order, navigation, and missing-image cases.

Then run:
```bash
cargo test --workspace -- test_formats_extended
cargo test --workspace -- test_library
```

---

## P10-T02

### Exotic Format Completion

Formats:
- CHM
- LIT
- LRF / LRX
- PDB / PML / RB
- SNB
- TCR
- AZW4
- DJVU

Scope:
- Replace best-effort or empty-result handlers with fuller parsers where feasible.
- Preserve silent fallback behavior only when a format cannot be reliably parsed.
- Keep magic-byte-first detection and format routing intact.
- Add coverage for both metadata extraction and text extraction edge cases.

Acceptance:
- These formats no longer behave like stubs where practical.
- Unsupported sub-features fail gracefully without breaking ingest.
- Each handler has fixture-backed regression coverage.

Then run:
```bash
cargo test --workspace -- test_formats_extended
cargo test --workspace -- test_formats_primary
```

---

## P10-T03

### Conversion Pipeline + EPUB Tweak Workflows

Scope:
- Build the conversion pipeline that Calibre users expect, but align it with xCalibre's local-first job model.
- Add EPUB tweak workflows for in-place editing where the format allows it.
- Keep conversion operations explicit, reversible where possible, and isolated from ingest jobs.

Acceptance:
- Conversion jobs have their own pipeline state and persistence.
- EPUB tweak actions do not corrupt existing library records.
- UI surfaces clearly distinguish conversion from ingest.

Then run:
```bash
cargo test --workspace -- test_conversion
cargo clippy --workspace -- -D warnings
```

---

## P10-T04

### Full Calibre-Style Repair / Editing Workflows

Scope:
- Expand integrity checking into repair workflows.
- Add richer metadata editing surfaces, including bulk edit flows and the remaining editor affordances.
- Keep user confirmation barriers where changes are destructive or ambiguous.

Acceptance:
- Repair actions can fix common library inconsistencies instead of only reporting them.
- Metadata edits are applied through explicit user actions.
- Bulk edit and single-record edit flows remain consistent.

Then run:
```bash
cargo test --workspace -- test_repair
cargo test --workspace -- test_metadata_editor
```

---

## P10-T05

### Tests, Fixture Coverage, and GAP Sync

Scope:
- Add fixture coverage for all Phase 10 features.
- Update `docs/GAP.md` whenever implementation status changes.
- Keep the phase doc and the implementation reality aligned.

Acceptance:
- Every newly implemented feature has regression coverage.
- GAP analysis reflects the current state of the codebase.
- Phase 10 remains the source of truth for the remaining in-scope backlog.

Then run:
```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
```
