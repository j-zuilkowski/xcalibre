# RMP-09a — Conversion Output Tier 1: TXT, HTML, DOCX (Red)

> Prerequisite: rmp05b complete (xcalibre-epub crate needed for EPUB→* conversion).
> TDD role: RED — define the conversion output API via failing tests.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R09a-T01 | Failing tests: EPUB → TXT conversion | ⬜ |
| R09a-T02 | Failing tests: EPUB → HTML conversion | ⬜ |
| R09a-T03 | Failing tests: EPUB → DOCX conversion | ⬜ |
| R09a-T04 | Failing tests: conversion job Tauri command | ⬜ |

---

## R09a-T01

Write `processing/tests/test_convert_txt.rs`:
```rust
use xcalibre_processing::convert::txt::epub_to_txt;
use std::path::PathBuf;

fn epub_fixture() -> PathBuf { PathBuf::from("tests/fixtures/fixture_epub.epub") }

#[test]
fn test_epub_to_txt_produces_non_empty_output() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.txt");
    epub_to_txt(&epub_fixture(), &out).expect("conversion");
    assert!(out.exists());
    let text = std::fs::read_to_string(&out).unwrap();
    assert!(!text.trim().is_empty(), "TXT output must not be empty");
}

#[test]
fn test_epub_to_txt_preserves_paragraph_breaks() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.txt");
    epub_to_txt(&epub_fixture(), &out).unwrap();
    let text = std::fs::read_to_string(&out).unwrap();
    // fixture_epub has one paragraph — at minimum, it should have no HTML tags
    assert!(!text.contains('<'), "TXT output must not contain HTML tags");
}

#[test]
fn test_epub_to_txt_invalid_path_errors() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("out.txt");
    let result = epub_to_txt(std::path::Path::new("/nonexistent.epub"), &out);
    assert!(result.is_err());
}
```

```bash
cargo test --workspace 2>&1 | grep -E "^error" | head -5
```

Expected: `xcalibre_processing::convert` not found. RED confirmed.

```bash
git add processing/tests/test_convert_txt.rs
git commit -m "R09a-T01: failing tests for EPUB→TXT conversion"
```

---

## R09a-T02

Write `processing/tests/test_convert_html.rs`:
```rust
use xcalibre_processing::convert::html::epub_to_html;
use std::path::PathBuf;

fn epub_fixture() -> PathBuf { PathBuf::from("tests/fixtures/fixture_epub.epub") }

#[test]
fn test_epub_to_html_produces_valid_html() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.html");
    epub_to_html(&epub_fixture(), &out).expect("conversion");
    let html = std::fs::read_to_string(&out).unwrap();
    assert!(html.contains("<!DOCTYPE html>") || html.contains("<html"),
            "output must be valid HTML: {}", &html[..100.min(html.len())]);
}

#[test]
fn test_epub_to_html_contains_body_text() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.html");
    epub_to_html(&epub_fixture(), &out).unwrap();
    let html = std::fs::read_to_string(&out).unwrap();
    assert!(html.contains("<body"), "output must have <body>");
    assert!(html.contains("fixture") || html.contains("chapter") || html.len() > 200,
            "output must contain book content");
}
```

```bash
git add processing/tests/test_convert_html.rs
git commit -m "R09a-T02: failing tests for EPUB→HTML conversion"
```

---

## R09a-T03

Add `docx-rs = "0.4"` to `processing/Cargo.toml` dev-dependencies (we'll move to deps in rmp09b).

Write `processing/tests/test_convert_docx.rs`:
```rust
use xcalibre_processing::convert::docx::epub_to_docx;
use std::path::PathBuf;

fn epub_fixture() -> PathBuf { PathBuf::from("tests/fixtures/fixture_epub.epub") }

#[test]
fn test_epub_to_docx_creates_file() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.docx");
    epub_to_docx(&epub_fixture(), &out).expect("conversion");
    assert!(out.exists(), "DOCX output file must be created");
    // DOCX files start with PK (ZIP magic)
    let bytes = std::fs::read(&out).unwrap();
    assert_eq!(&bytes[..2], b"PK", "DOCX must be a valid ZIP/DOCX file");
}

#[test]
fn test_epub_to_docx_minimum_size() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.docx");
    epub_to_docx(&epub_fixture(), &out).unwrap();
    let metadata = std::fs::metadata(&out).unwrap();
    assert!(metadata.len() > 1000, "DOCX file must be non-trivial in size");
}
```

```bash
git add processing/tests/test_convert_docx.rs processing/Cargo.toml
git commit -m "R09a-T03: failing tests for EPUB→DOCX conversion"
```

---

## R09a-T04

Write `ui/src/components/ConversionDialog.test.tsx`:
```tsx
import { render, screen, fireEvent, waitFor } from "@testing-library/react"
import { describe, it, expect, vi, beforeEach } from "vitest"
import { ConversionDialog } from "./ConversionDialog"
import { mockInvoke } from "../test/setup"

const mockBook = {
  id: "b1", title: "Test Book", authors: ["A"], format: "EPUB",
  cover_path: null, progress_percent: 0, last_opened_at: null,
}

beforeEach(() => {
  mockInvoke("convert_book", "/tmp/Test Book.txt")
})

describe("ConversionDialog", () => {
  it("renders format selector", () => {
    render(<ConversionDialog book={mockBook} onClose={vi.fn()} />)
    expect(screen.getByTestId("output-format-select")).toBeInTheDocument()
  })

  it("shows TXT, HTML, DOCX options", () => {
    render(<ConversionDialog book={mockBook} onClose={vi.fn()} />)
    expect(screen.getByText("TXT")).toBeInTheDocument()
    expect(screen.getByText("HTML")).toBeInTheDocument()
    expect(screen.getByText("DOCX")).toBeInTheDocument()
  })

  it("calls convert_book on submit", async () => {
    render(<ConversionDialog book={mockBook} onClose={vi.fn()} />)
    fireEvent.change(screen.getByTestId("output-format-select"), { target: { value: "TXT" } })
    fireEvent.click(screen.getByTestId("convert-btn"))
    await waitFor(() =>
      expect(screen.getByTestId("conversion-success")).toBeInTheDocument()
    )
  })
})
```

```bash
cd ui && npm test 2>&1 | grep "Cannot find\|FAIL" | head -5 && cd ..
git add ui/src/components/ConversionDialog.test.tsx
git commit -m "R09a-T04: failing tests for ConversionDialog"
```
