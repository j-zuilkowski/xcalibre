# RMP-15a — Conversion Tier 2: PDF + MOBI (Red: Failing Tests)

> Prerequisite: rmp09b complete (Tier 1 conversion crate structure exists).
> Decision: S4-B — PDF via WebView print pipeline.
> TDD role: RED — define PDF and MOBI conversion APIs via failing tests.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R15a-T01 | Failing tests: EPUB → PDF conversion | ⬜ |
| R15a-T02 | Failing tests: EPUB → MOBI conversion | ⬜ |
| R15a-T03 | Add PDF and MOBI to ConversionDialog tests | ⬜ |

---

## R15a-T01

Write `processing/tests/test_convert_pdf.rs`:
```rust
use xcalibre_processing::convert::pdf::epub_to_pdf;
use std::path::PathBuf;

fn epub_fixture() -> PathBuf { PathBuf::from("tests/fixtures/fixture_epub.epub") }

#[test]
fn test_epub_to_pdf_creates_file() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.pdf");
    epub_to_pdf(&epub_fixture(), &out).expect("conversion");
    assert!(out.exists(), "PDF output file must be created");
}

#[test]
fn test_epub_to_pdf_has_pdf_magic() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.pdf");
    epub_to_pdf(&epub_fixture(), &out).unwrap();
    let bytes = std::fs::read(&out).unwrap();
    // PDF files start with %PDF-
    assert!(bytes.starts_with(b"%PDF-"), "output must be a valid PDF: {:?}", &bytes[..8.min(bytes.len())]);
}

#[test]
fn test_epub_to_pdf_minimum_size() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.pdf");
    epub_to_pdf(&epub_fixture(), &out).unwrap();
    let meta = std::fs::metadata(&out).unwrap();
    assert!(meta.len() > 500, "PDF must be non-trivial in size");
}

#[test]
fn test_epub_to_pdf_invalid_path_errors() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("out.pdf");
    let result = epub_to_pdf(std::path::Path::new("/nonexistent.epub"), &out);
    assert!(result.is_err());
}
```

```bash
cargo test --workspace 2>&1 | grep -E "^error" | head -5
```

Expected: `convert::pdf` not found. RED confirmed.

```bash
git add processing/tests/test_convert_pdf.rs
git commit -m "R15a-T01: failing tests for EPUB→PDF conversion"
```

---

## R15a-T02

Write `processing/tests/test_convert_mobi.rs`:
```rust
use xcalibre_processing::convert::mobi::epub_to_mobi;
use std::path::PathBuf;

fn epub_fixture() -> PathBuf { PathBuf::from("tests/fixtures/fixture_epub.epub") }

#[test]
fn test_epub_to_mobi_creates_file() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.mobi");
    epub_to_mobi(&epub_fixture(), &out).expect("conversion");
    assert!(out.exists(), "MOBI output file must be created");
}

#[test]
fn test_epub_to_mobi_has_mobi_magic() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.mobi");
    epub_to_mobi(&epub_fixture(), &out).unwrap();
    let bytes = std::fs::read(&out).unwrap();
    // MOBI/PalmDB files start with a 32-byte database name header,
    // then have "BOOK" + "MOBI" identifiers at offsets 60 and 96.
    // Minimum: file is non-empty and of reasonable size.
    assert!(bytes.len() > 1000, "MOBI must be non-trivial in size: {} bytes", bytes.len());
}

#[test]
fn test_epub_to_mobi_invalid_path_errors() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("out.mobi");
    let result = epub_to_mobi(std::path::Path::new("/nonexistent.epub"), &out);
    assert!(result.is_err());
}
```

```bash
cargo test --workspace 2>&1 | grep -E "^error" | head -5
```

Expected: `convert::mobi` not found. RED confirmed.

```bash
git add processing/tests/test_convert_mobi.rs
git commit -m "R15a-T02: failing tests for EPUB→MOBI conversion"
```

---

## R15a-T03

Extend `ui/src/components/ConversionDialog.test.tsx` (or write a new file `ConversionDialog.tier2.test.tsx`):
```tsx
import { render, screen, fireEvent, waitFor } from "@testing-library/react"
import { describe, it, expect, vi, beforeEach } from "vitest"
import { ConversionDialog } from "./ConversionDialog"
import { mockInvoke } from "../test/setup"

const mockBook = {
  id: "b1", title: "Test Book", authors: ["A"], format: "EPUB",
  cover_path: null, progress_percent: 0, last_opened_at: null,
  file_path: "/tmp/test.epub",
}

beforeEach(() => {
  mockInvoke("convert_book", "/tmp/Test Book.pdf")
})

describe("ConversionDialog Tier 2 formats", () => {
  it("shows PDF option", () => {
    render(<ConversionDialog book={mockBook} onClose={vi.fn()} />)
    expect(screen.getByText("PDF")).toBeInTheDocument()
  })

  it("shows MOBI option", () => {
    render(<ConversionDialog book={mockBook} onClose={vi.fn()} />)
    expect(screen.getByText("MOBI")).toBeInTheDocument()
  })

  it("calls convert_book with PDF format", async () => {
    render(<ConversionDialog book={mockBook} onClose={vi.fn()} />)
    fireEvent.change(screen.getByTestId("output-format-select"), { target: { value: "PDF" } })
    fireEvent.click(screen.getByTestId("convert-btn"))
    await waitFor(() =>
      expect(screen.getByTestId("conversion-success")).toBeInTheDocument()
    )
  })
})
```

```bash
cd ui && npm test 2>&1 | grep "Cannot find\|FAIL" | head -5 && cd ..
git add ui/src/components/ConversionDialog.tier2.test.tsx
git commit -m "R15a-T03: failing tests for PDF and MOBI in ConversionDialog"
```

---

### ✅ RED Checkpoint

```bash
cargo test --workspace 2>&1 | grep -c "^error"
cd ui && npm test 2>&1 | grep -c "FAIL" && cd ..
```

Both should show failures. Proceed to **rmp15b**.
