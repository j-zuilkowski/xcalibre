# RMP-18a — Conversion Tier 3: FB2, RTF, HTMLZ (Red: Failing Tests)

> Prerequisite: rmp09b complete (convert crate structure).
> TDD role: RED — define FB2, RTF, HTMLZ conversion APIs via failing tests.
> Scope: FB2 via quick-xml writer, RTF via control-word walking, HTMLZ as flat ZIP.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R18a-T01 | Failing tests: EPUB → FB2 | ⬜ |
| R18a-T02 | Failing tests: EPUB → RTF | ⬜ |
| R18a-T03 | Failing tests: EPUB → HTMLZ | ⬜ |
| R18a-T04 | Add FB2/RTF/HTMLZ to ConversionDialog tests | ⬜ |

---

## R18a-T01

Write `processing/tests/test_convert_fb2.rs`:
```rust
use xcalibre_processing::convert::fb2::epub_to_fb2;
use std::path::PathBuf;

fn epub_fixture() -> PathBuf { PathBuf::from("tests/fixtures/fixture_epub.epub") }

#[test]
fn test_epub_to_fb2_creates_file() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.fb2");
    epub_to_fb2(&epub_fixture(), &out).expect("conversion");
    assert!(out.exists(), "FB2 output must be created");
}

#[test]
fn test_fb2_is_valid_xml() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.fb2");
    epub_to_fb2(&epub_fixture(), &out).unwrap();
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(content.contains("<?xml") || content.starts_with("<FictionBook"),
            "FB2 must be valid XML: {}", &content[..100.min(content.len())]);
}

#[test]
fn test_fb2_has_fictionbook_root() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.fb2");
    epub_to_fb2(&epub_fixture(), &out).unwrap();
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(content.contains("<FictionBook"), "FB2 must have <FictionBook> root element");
}

#[test]
fn test_fb2_has_body_section() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.fb2");
    epub_to_fb2(&epub_fixture(), &out).unwrap();
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(content.contains("<body>") || content.contains("<body "),
            "FB2 must have a <body> element");
}

#[test]
fn test_fb2_invalid_path_errors() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("out.fb2");
    assert!(epub_to_fb2(std::path::Path::new("/nonexistent.epub"), &out).is_err());
}
```

```bash
cargo test --workspace 2>&1 | grep -E "^error" | head -5
```

RED confirmed.

```bash
git add processing/tests/test_convert_fb2.rs
git commit -m "R18a-T01: failing tests for EPUB→FB2 conversion"
```

---

## R18a-T02

Write `processing/tests/test_convert_rtf.rs`:
```rust
use xcalibre_processing::convert::rtf::epub_to_rtf;
use std::path::PathBuf;

fn epub_fixture() -> PathBuf { PathBuf::from("tests/fixtures/fixture_epub.epub") }

#[test]
fn test_epub_to_rtf_creates_file() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.rtf");
    epub_to_rtf(&epub_fixture(), &out).expect("conversion");
    assert!(out.exists(), "RTF output must be created");
}

#[test]
fn test_rtf_starts_with_rtf_header() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.rtf");
    epub_to_rtf(&epub_fixture(), &out).unwrap();
    let bytes = std::fs::read(&out).unwrap();
    // All RTF files start with "{\\rtf"
    assert!(bytes.starts_with(b"{\\rtf"),
            "RTF must start with {{\\rtf: {:?}", &bytes[..8.min(bytes.len())]);
}

#[test]
fn test_rtf_contains_text_content() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.rtf");
    epub_to_rtf(&epub_fixture(), &out).unwrap();
    let content = std::fs::read_to_string(&out).unwrap();
    // Should contain some readable ASCII text
    let ascii_count = content.chars().filter(|c| c.is_ascii_alphabetic()).count();
    assert!(ascii_count > 10, "RTF must contain readable text, got {} ascii chars", ascii_count);
}

#[test]
fn test_rtf_invalid_path_errors() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("out.rtf");
    assert!(epub_to_rtf(std::path::Path::new("/nonexistent.epub"), &out).is_err());
}
```

```bash
cargo test --workspace 2>&1 | grep -E "^error" | head -5
```

RED confirmed.

```bash
git add processing/tests/test_convert_rtf.rs
git commit -m "R18a-T02: failing tests for EPUB→RTF conversion"
```

---

## R18a-T03

Write `processing/tests/test_convert_htmlz.rs`:
```rust
use xcalibre_processing::convert::htmlz::epub_to_htmlz;
use std::path::PathBuf;

fn epub_fixture() -> PathBuf { PathBuf::from("tests/fixtures/fixture_epub.epub") }

#[test]
fn test_epub_to_htmlz_creates_file() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.htmlz");
    epub_to_htmlz(&epub_fixture(), &out).expect("conversion");
    assert!(out.exists(), "HTMLZ output must be created");
}

#[test]
fn test_htmlz_is_valid_zip() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.htmlz");
    epub_to_htmlz(&epub_fixture(), &out).unwrap();
    let bytes = std::fs::read(&out).unwrap();
    assert_eq!(&bytes[..2], b"PK", "HTMLZ must be a ZIP file");
}

#[test]
fn test_htmlz_contains_index_html() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.htmlz");
    epub_to_htmlz(&epub_fixture(), &out).unwrap();
    let file = std::fs::File::open(&out).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    let entry_names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().to_string())
        .collect();
    assert!(
        entry_names.iter().any(|n| n == "index.html" || n.ends_with("/index.html")),
        "HTMLZ must contain index.html: {:?}", entry_names
    );
}

#[test]
fn test_htmlz_contains_metadata_opf() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.htmlz");
    epub_to_htmlz(&epub_fixture(), &out).unwrap();
    let file = std::fs::File::open(&out).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    let entry_names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().to_string())
        .collect();
    assert!(
        entry_names.iter().any(|n| n.contains("metadata") || n.ends_with(".opf")),
        "HTMLZ should contain metadata: {:?}", entry_names
    );
}
```

```bash
cargo test --workspace 2>&1 | grep -E "^error" | head -5
```

RED confirmed.

```bash
git add processing/tests/test_convert_htmlz.rs
git commit -m "R18a-T03: failing tests for EPUB→HTMLZ conversion"
```

---

## R18a-T04

Write `ui/src/components/ConversionDialog.tier3.test.tsx`:
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

beforeEach(() => {
  mockInvoke("convert_book", "/tmp/Test Book.fb2")
})

describe("ConversionDialog Tier 3 formats", () => {
  it("shows FB2 option",   () => { render(<ConversionDialog book={mockBook} onClose={() => {}} />); expect(screen.getByText("FB2")).toBeInTheDocument() })
  it("shows RTF option",   () => { render(<ConversionDialog book={mockBook} onClose={() => {}} />); expect(screen.getByText("RTF")).toBeInTheDocument() })
  it("shows HTMLZ option", () => { render(<ConversionDialog book={mockBook} onClose={() => {}} />); expect(screen.getByText("HTMLZ")).toBeInTheDocument() })
})
```

```bash
cd ui && npm test 2>&1 | grep "Cannot find\|FAIL" | head -5 && cd ..
git add ui/src/components/ConversionDialog.tier3.test.tsx
git commit -m "R18a-T04: failing tests for FB2/RTF/HTMLZ in ConversionDialog"
```

---

### ✅ RED Checkpoint

```bash
cargo test --workspace 2>&1 | grep -c "^error"
cd ui && npm test 2>&1 | grep -c "FAIL" && cd ..
```

Both should show failures. Proceed to **rmp18b**.
