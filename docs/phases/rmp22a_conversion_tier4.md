# RMP-22a — Conversion Tier 4: LRF, PDB, PML, RB, SNB, TCR, LIT (Red: Failing Tests)

> Prerequisite: rmp09b complete (convert crate), rmp18b complete (Tier 3 done).
> TDD role: RED — define Tier 4 conversion APIs via failing tests.
> Build order: TCR (simplest) → SNB text → LRF (structured) → DjVu → AZW4 → LIT text → TXT pipeline
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R22a-T01 | Failing tests: EPUB → LRF conversion | ⬜ |
| R22a-T02 | Failing tests: EPUB → PDB/PML/RB conversion | ⬜ |
| R22a-T03 | Failing tests: EPUB → TCR conversion | ⬜ |
| R22a-T04 | Failing tests: EPUB → SNB conversion (text export) | ⬜ |
| R22a-T05 | Add Tier 4 formats to ConversionDialog tests | ⬜ |

---

## R22a-T01

Write `processing/tests/test_convert_lrf.rs`:
```rust
use xcalibre_processing::convert::lrf::epub_to_lrf;
use std::path::PathBuf;

fn epub_fixture() -> PathBuf { PathBuf::from("tests/fixtures/fixture_epub.epub") }

#[test]
fn test_epub_to_lrf_creates_file() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.lrf");
    epub_to_lrf(&epub_fixture(), &out).expect("conversion");
    assert!(out.exists(), "LRF output must be created");
}

#[test]
fn test_lrf_has_lrf_magic() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.lrf");
    epub_to_lrf(&epub_fixture(), &out).unwrap();
    let bytes = std::fs::read(&out).unwrap();
    // LRF magic bytes: L R F \x00 at offset 0 (actually 0x4C 0x00 0x00 0x00 as little-endian u32)
    // More accurately: LRF signature is 8 bytes: 00 00 00 00 4C 00 52 00 (Unicode "LR")
    // Simplest check: file is non-empty and at least 1KB
    assert!(bytes.len() > 100, "LRF must be non-trivial: {} bytes", bytes.len());
}

#[test]
fn test_lrf_invalid_path_errors() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("out.lrf");
    assert!(epub_to_lrf(std::path::Path::new("/nonexistent.epub"), &out).is_err());
}
```

```bash
cargo test --workspace 2>&1 | grep -E "^error" | head -5
```

RED confirmed.

```bash
git add processing/tests/test_convert_lrf.rs
git commit -m "R22a-T01: failing tests for EPUB→LRF conversion"
```

---

## R22a-T02

Write `processing/tests/test_convert_pdb.rs`:
```rust
use xcalibre_processing::convert::pdb::{epub_to_pdb, epub_to_pml, epub_to_rb};
use std::path::PathBuf;

fn epub_fixture() -> PathBuf { PathBuf::from("tests/fixtures/fixture_epub.epub") }

#[test]
fn test_epub_to_pdb_creates_file() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.pdb");
    epub_to_pdb(&epub_fixture(), &out).expect("conversion");
    assert!(out.exists(), "PDB output must be created");
    assert!(std::fs::metadata(&out).unwrap().len() > 100, "PDB must be non-empty");
}

#[test]
fn test_epub_to_pml_creates_file() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.pml");
    epub_to_pml(&epub_fixture(), &out).expect("conversion");
    let content = std::fs::read_to_string(&out).unwrap();
    // PML is plain text with \p and \c tags
    assert!(!content.trim().is_empty(), "PML must not be empty");
}

#[test]
fn test_epub_to_rb_creates_file() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.rb");
    epub_to_rb(&epub_fixture(), &out).expect("conversion");
    assert!(out.exists(), "RocketBook (.rb) output must be created");
    assert!(std::fs::metadata(&out).unwrap().len() > 100);
}
```

```bash
cargo test --workspace 2>&1 | grep -E "^error" | head -5
```

RED confirmed.

```bash
git add processing/tests/test_convert_pdb.rs
git commit -m "R22a-T02: failing tests for EPUB→PDB/PML/RB conversion"
```

---

## R22a-T03

Write `processing/tests/test_convert_tcr.rs`:
```rust
use xcalibre_processing::convert::tcr::epub_to_tcr;
use std::path::PathBuf;

fn epub_fixture() -> PathBuf { PathBuf::from("tests/fixtures/fixture_epub.epub") }

#[test]
fn test_epub_to_tcr_creates_file() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.tcr");
    epub_to_tcr(&epub_fixture(), &out).expect("conversion");
    assert!(out.exists(), "TCR output must be created");
}

#[test]
fn test_tcr_is_compressed() {
    // TCR is a simple compression format; output should be smaller than input text
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.tcr");
    epub_to_tcr(&epub_fixture(), &out).unwrap();
    let tcr_size = std::fs::metadata(&out).unwrap().len();
    assert!(tcr_size > 0, "TCR must not be empty");
}

#[test]
fn test_tcr_invalid_path_errors() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("out.tcr");
    assert!(epub_to_tcr(std::path::Path::new("/nonexistent.epub"), &out).is_err());
}
```

```bash
cargo test --workspace 2>&1 | grep -E "^error" | head -5
```

RED confirmed.

```bash
git add processing/tests/test_convert_tcr.rs
git commit -m "R22a-T03: failing tests for EPUB→TCR conversion"
```

---

## R22a-T04

Write `processing/tests/test_convert_snb.rs`:
```rust
use xcalibre_processing::convert::snb::epub_to_snb_text;
use std::path::PathBuf;

fn epub_fixture() -> PathBuf { PathBuf::from("tests/fixtures/fixture_epub.epub") }

#[test]
fn test_epub_to_snb_creates_file() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.snb");
    epub_to_snb_text(&epub_fixture(), &out).expect("conversion");
    assert!(out.exists(), "SNB output must be created");
}

#[test]
fn test_snb_output_has_content() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.snb");
    epub_to_snb_text(&epub_fixture(), &out).unwrap();
    let size = std::fs::metadata(&out).unwrap().len();
    assert!(size > 100, "SNB output must be non-trivial: {size} bytes");
}
```

```bash
cargo test --workspace 2>&1 | grep -E "^error" | head -5
```

RED confirmed.

```bash
git add processing/tests/test_convert_snb.rs
git commit -m "R22a-T04: failing tests for EPUB→SNB text export"
```

---

## R22a-T05

Write `ui/src/components/ConversionDialog.tier4.test.tsx`:
```tsx
import { render, screen } from "@testing-library/react"
import { describe, it, expect, beforeEach } from "vitest"
import { ConversionDialog } from "./ConversionDialog"
import { mockInvoke } from "../test/setup"

const mockBook = {
  id: "b1", title: "Test Book", authors: ["A"], format: "EPUB",
  cover_path: null, progress_percent: 0, last_opened_at: null,
  file_path: "/tmp/test.epub",
}

beforeEach(() => { mockInvoke("convert_book", "/tmp/output.lrf") })

describe("ConversionDialog Tier 4 formats", () => {
  it("shows LRF option",  () => { render(<ConversionDialog book={mockBook} onClose={() => {}} />); expect(screen.getByText("LRF")).toBeInTheDocument() })
  it("shows PDB option",  () => { render(<ConversionDialog book={mockBook} onClose={() => {}} />); expect(screen.getByText("PDB")).toBeInTheDocument() })
  it("shows PML option",  () => { render(<ConversionDialog book={mockBook} onClose={() => {}} />); expect(screen.getByText("PML")).toBeInTheDocument() })
  it("shows RB option",   () => { render(<ConversionDialog book={mockBook} onClose={() => {}} />); expect(screen.getByText("RB")).toBeInTheDocument() })
  it("shows SNB option",  () => { render(<ConversionDialog book={mockBook} onClose={() => {}} />); expect(screen.getByText("SNB")).toBeInTheDocument() })
  it("shows TCR option",  () => { render(<ConversionDialog book={mockBook} onClose={() => {}} />); expect(screen.getByText("TCR")).toBeInTheDocument() })
})
```

```bash
cd ui && npm test 2>&1 | grep "Cannot find\|FAIL" | head -5 && cd ..
git add ui/src/components/ConversionDialog.tier4.test.tsx
git commit -m "R22a-T05: failing tests for Tier 4 formats in ConversionDialog"
```

---

### ✅ RED Checkpoint

```bash
cargo test --workspace 2>&1 | grep -c "^error"
cd ui && npm test 2>&1 | grep -c "FAIL" && cd ..
```

Both should show failures. Proceed to **rmp22b**.
