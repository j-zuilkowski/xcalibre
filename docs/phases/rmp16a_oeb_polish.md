# RMP-16a — OEB Polish Toolkit (Red: Failing Tests)

> Prerequisite: rmp05b complete (xcalibre-epub crate).
> TDD role: RED — define KEPUB conversion, pretty-print, and book stats APIs via failing tests.
> Scope: KEPUB output, EPUB pretty-print, hyphenation hints, word/page/reading-time stats.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R16a-T01 | Failing tests: EPUB → KEPUB conversion | ⬜ |
| R16a-T02 | Failing tests: EPUB pretty-print / clean | ⬜ |
| R16a-T03 | Failing tests: book statistics calculation | ⬜ |
| R16a-T04 | Failing tests: BookStatsPanel component | ⬜ |

---

## R16a-T01

Write `processing/tests/test_convert_kepub.rs`:
```rust
use xcalibre_processing::convert::kepub::epub_to_kepub;
use std::path::PathBuf;

fn epub_fixture() -> PathBuf { PathBuf::from("tests/fixtures/fixture_epub.epub") }

#[test]
fn test_epub_to_kepub_creates_file() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.kepub.epub");
    epub_to_kepub(&epub_fixture(), &out).expect("conversion");
    assert!(out.exists(), "KEPUB output must be created");
}

#[test]
fn test_kepub_is_valid_zip() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.kepub.epub");
    epub_to_kepub(&epub_fixture(), &out).unwrap();
    let bytes = std::fs::read(&out).unwrap();
    assert_eq!(&bytes[..2], b"PK", "KEPUB must be a ZIP file");
}

#[test]
fn test_kepub_contains_kobo_spans() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.kepub.epub");
    epub_to_kepub(&epub_fixture(), &out).unwrap();
    // Open as zip and check at least one spine HTML has kobo:* attributes
    let file = std::fs::File::open(&out).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    let mut found_kobo = false;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).unwrap();
        let name = entry.name().to_string();
        if name.ends_with(".html") || name.ends_with(".xhtml") {
            let mut content = String::new();
            std::io::Read::read_to_string(&mut entry, &mut content).unwrap();
            if content.contains("kobo:") || content.contains("epub:type") {
                found_kobo = true;
                break;
            }
        }
    }
    assert!(found_kobo, "KEPUB spine items must contain Kobo markup");
}
```

Add `zip = "2"` to `processing/Cargo.toml` dev-dependencies.

```bash
cargo test --workspace 2>&1 | grep -E "^error" | head -5
```

Expected: `convert::kepub` not found. RED confirmed.

```bash
git add processing/tests/test_convert_kepub.rs processing/Cargo.toml
git commit -m "R16a-T01: failing tests for EPUB→KEPUB conversion"
```

---

## R16a-T02

Write `processing/tests/test_epub_pretty.rs`:
```rust
use xcalibre_processing::polish::pretty_print_epub;
use std::path::PathBuf;

fn epub_fixture() -> PathBuf { PathBuf::from("tests/fixtures/fixture_epub.epub") }

#[test]
fn test_pretty_print_creates_output() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("pretty.epub");
    pretty_print_epub(&epub_fixture(), &out).expect("pretty-print");
    assert!(out.exists(), "pretty-printed EPUB must be created");
}

#[test]
fn test_pretty_print_output_is_valid_zip() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("pretty.epub");
    pretty_print_epub(&epub_fixture(), &out).unwrap();
    let bytes = std::fs::read(&out).unwrap();
    assert_eq!(&bytes[..2], b"PK");
}

#[test]
fn test_pretty_print_spine_items_are_indented() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("pretty.epub");
    pretty_print_epub(&epub_fixture(), &out).unwrap();
    let file = std::fs::File::open(&out).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).unwrap();
        if entry.name().ends_with(".xhtml") || entry.name().ends_with(".html") {
            let mut content = String::new();
            std::io::Read::read_to_string(&mut entry, &mut content).unwrap();
            // After pretty-printing, content should have newlines and indentation
            assert!(content.contains('\n'), "pretty-printed HTML must contain newlines");
            break;
        }
    }
}
```

```bash
cargo test --workspace 2>&1 | grep -E "^error" | head -5
```

Expected: `polish::pretty_print_epub` not found. RED confirmed.

```bash
git add processing/tests/test_epub_pretty.rs
git commit -m "R16a-T02: failing tests for EPUB pretty-print"
```

---

## R16a-T03

Write `processing/tests/test_book_stats.rs`:
```rust
use xcalibre_processing::stats::{compute_book_stats, BookStats};
use std::path::PathBuf;

fn epub_fixture() -> PathBuf { PathBuf::from("tests/fixtures/fixture_epub.epub") }

#[test]
fn test_book_stats_word_count() {
    let stats = compute_book_stats(&epub_fixture()).expect("stats");
    assert!(stats.word_count > 0, "word count must be positive");
}

#[test]
fn test_book_stats_page_count_estimate() {
    let stats = compute_book_stats(&epub_fixture()).expect("stats");
    // Estimate: 250 words per page
    assert!(stats.page_count_estimate > 0, "page count estimate must be positive");
    assert_eq!(
        stats.page_count_estimate,
        ((stats.word_count as f64 / 250.0).ceil() as u32).max(1)
    );
}

#[test]
fn test_book_stats_reading_time_minutes() {
    let stats = compute_book_stats(&epub_fixture()).expect("stats");
    // Average reading speed: 238 wpm
    assert!(stats.reading_time_minutes > 0 || stats.word_count < 238,
            "reading time must be positive for non-trivial books");
}

#[test]
fn test_book_stats_character_count() {
    let stats = compute_book_stats(&epub_fixture()).expect("stats");
    assert!(stats.character_count > 0);
    assert!(stats.character_count >= stats.word_count,
            "characters must be >= words");
}

#[test]
fn test_book_stats_invalid_path_errors() {
    let result = compute_book_stats(std::path::Path::new("/nonexistent.epub"));
    assert!(result.is_err());
}
```

```bash
cargo test --workspace 2>&1 | grep -E "^error" | head -5
```

Expected: `stats::compute_book_stats` not found. RED confirmed.

```bash
git add processing/tests/test_book_stats.rs
git commit -m "R16a-T03: failing tests for book statistics"
```

---

## R16a-T04

Write `ui/src/components/BookStatsPanel.test.tsx`:
```tsx
import { render, screen, waitFor } from "@testing-library/react"
import { describe, it, expect, vi, beforeEach } from "vitest"
import { BookStatsPanel } from "./BookStatsPanel"
import { mockInvoke } from "../test/setup"

const mockBook = {
  id: "b1", title: "Dune", authors: ["Frank Herbert"],
  format: "EPUB", cover_path: null, progress_percent: 0, last_opened_at: null,
  file_path: "/tmp/dune.epub",
}

const mockStats = {
  word_count: 188000,
  character_count: 900000,
  page_count_estimate: 752,
  reading_time_minutes: 789,
}

beforeEach(() => {
  mockInvoke("get_book_stats", mockStats)
})

describe("BookStatsPanel", () => {
  it("shows word count", async () => {
    render(<BookStatsPanel book={mockBook} />)
    await waitFor(() =>
      expect(screen.getByTestId("stat-word-count")).toHaveTextContent("188,000")
    )
  })

  it("shows page count estimate", async () => {
    render(<BookStatsPanel book={mockBook} />)
    await waitFor(() =>
      expect(screen.getByTestId("stat-page-count")).toHaveTextContent("752")
    )
  })

  it("shows reading time in hours and minutes", async () => {
    render(<BookStatsPanel book={mockBook} />)
    await waitFor(() => {
      const el = screen.getByTestId("stat-reading-time")
      // 789 minutes = 13h 9m
      expect(el.textContent).toMatch(/13h/)
    })
  })

  it("shows loading state initially", () => {
    render(<BookStatsPanel book={mockBook} />)
    expect(screen.getByTestId("stats-loading")).toBeInTheDocument()
  })
})
```

```bash
cd ui && npm test 2>&1 | grep "Cannot find\|FAIL" | head -5 && cd ..
git add ui/src/components/BookStatsPanel.test.tsx
git commit -m "R16a-T04: failing tests for BookStatsPanel component"
```

---

### ✅ RED Checkpoint

```bash
cargo test --workspace 2>&1 | grep -c "^error"
cd ui && npm test 2>&1 | grep -c "FAIL" && cd ..
```

Both should show failures. Proceed to **rmp16b**.
