# RMP-16b — OEB Polish Toolkit (Green: Implementation)

> Prerequisite: rmp16a complete, rmp05b complete.
> TDD role: GREEN — implement KEPUB, pretty-print, book stats, and StatsPanel.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R16b-T01 | `convert/kepub.rs` — EPUB to KEPUB | ⬜ |
| R16b-T02 | `polish/mod.rs` — EPUB pretty-print | ⬜ |
| R16b-T03 | `stats/mod.rs` — book statistics | ⬜ |
| R16b-T04 | Tauri commands: convert_to_kepub, get_book_stats | ⬜ |
| R16b-T05 | `BookStatsPanel.tsx` component | ⬜ |
| R16b-T06 | Milestone check + visual inspection | ⬜ |

---

## R16b-T01

KEPUB is an EPUB with Kobo-specific markup: each paragraph's text nodes are wrapped in
`<span class="kobo-span" epub:type="...">` elements and a Kobo readingorder namespace is added.

Write `processing/src/convert/kepub.rs`:
```rust
use crate::error::ProcessingError;
use std::io::{Read, Write};
use std::path::Path;
use xcalibre_epub::Container;

pub fn epub_to_kepub(epub_path: &Path, out_path: &Path) -> Result<(), ProcessingError> {
    let container = Container::open(epub_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    let spine = container.spine_hrefs();

    // Open a second handle to write into
    let mut new_container = Container::open(epub_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    for href in &spine {
        if let Ok(data) = container.read_item(href) {
            let html = String::from_utf8_lossy(&data);
            let modified = inject_kobo_spans(&html);
            let _ = new_container.write_item(href, modified.as_bytes(), "application/xhtml+xml");
        }
    }

    // Add Kobo namespace to OPF
    if let Ok(opf_data) = container.read_item(container.opf_path()) {
        let opf_str = String::from_utf8_lossy(&opf_data);
        let updated = opf_str.replace(
            "<package",
            "<package xmlns:kobo=\"http://www.kobo.com\"",
        );
        let _ = new_container.write_item(
            container.opf_path(),
            updated.as_bytes(),
            "application/oebps-package+xml",
        );
    }

    new_container.save_as(out_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    Ok(())
}

fn inject_kobo_spans(html: &str) -> String {
    html.replace("<p>", "<p><span class=\"kobo-span\" epub:type=\"…\">")
        .replace("</p>", "</span></p>")
}
```

In `processing/src/convert/mod.rs`, add:
```rust
pub mod kepub;
```

Then run:
```bash
cargo test --workspace -- test_epub_to_kepub test_kepub
git add processing/src/convert/kepub.rs processing/src/convert/mod.rs
git commit -m "R16b-T01: EPUB→KEPUB conversion — all KEPUB tests green"
```

---

## R16b-T02

Write `processing/src/polish/mod.rs`:
```rust
use crate::error::ProcessingError;
use std::path::Path;
use xcalibre_epub::Container;

pub fn pretty_print_epub(epub_path: &Path, out_path: &Path) -> Result<(), ProcessingError> {
    let container = Container::open(epub_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    let mut new_container = Container::open(epub_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    let spine = container.spine_hrefs();
    for href in &spine {
        if let Ok(data) = container.read_item(href) {
            let html = String::from_utf8_lossy(&data);
            let pretty = pretty_print_xml(&html);
            let _ = new_container.write_item(href, pretty.as_bytes(), "application/xhtml+xml");
        }
    }

    new_container.save_as(out_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    Ok(())
}

fn pretty_print_xml(src: &str) -> String {
    // Simple indenting pretty-printer: tracks depth via tag open/close
    let mut result = String::with_capacity(src.len());
    let mut depth: usize = 0;
    let mut in_tag = false;
    let mut current_tag = String::new();

    for line in src.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() { continue; }

        // Detect closing tags to de-dent before printing
        if trimmed.starts_with("</") {
            depth = depth.saturating_sub(1);
        }

        result.push_str(&"  ".repeat(depth));
        result.push_str(trimmed);
        result.push('\n');

        // Detect opening tags to increase depth for next line
        if trimmed.starts_with('<')
            && !trimmed.starts_with("</")
            && !trimmed.starts_with("<!--")
            && !trimmed.starts_with("<?")
            && !trimmed.ends_with("/>")
        {
            depth += 1;
        }
        if trimmed.ends_with("/>") && depth > 0 {
            // self-closing: no depth change needed
        }
    }
    result
}
```

Add to `processing/src/lib.rs`:
```rust
pub mod polish;
```

Then run:
```bash
cargo test --workspace -- test_epub_pretty
git add processing/src/polish/mod.rs processing/src/lib.rs
git commit -m "R16b-T02: EPUB pretty-print — all pretty-print tests green"
```

---

## R16b-T03

Write `processing/src/stats/mod.rs`:
```rust
use crate::error::ProcessingError;
use serde::{Deserialize, Serialize};
use std::path::Path;
use xcalibre_epub::Container;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookStats {
    pub word_count:           u32,
    pub character_count:      u32,
    pub page_count_estimate:  u32,
    pub reading_time_minutes: u32,
}

const WORDS_PER_PAGE:   f64 = 250.0;
const WORDS_PER_MINUTE: f64 = 238.0;

pub fn compute_book_stats(epub_path: &Path) -> Result<BookStats, ProcessingError> {
    let container = Container::open(epub_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    let spine = container.spine_hrefs();
    let mut total_text = String::new();

    for href in &spine {
        if let Ok(bytes) = container.read_item(href) {
            let html = String::from_utf8_lossy(&bytes);
            total_text.push_str(&strip_html(&html));
            total_text.push(' ');
        }
    }

    let word_count      = total_text.split_whitespace().count() as u32;
    let character_count = total_text.chars().filter(|c| !c.is_whitespace()).count() as u32;
    let page_count_estimate  = ((word_count as f64 / WORDS_PER_PAGE).ceil() as u32).max(1);
    let reading_time_minutes = ((word_count as f64 / WORDS_PER_MINUTE).ceil() as u32).max(
        if word_count == 0 { 0 } else { 1 }
    );

    Ok(BookStats { word_count, character_count, page_count_estimate, reading_time_minutes })
}

fn strip_html(html: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out
}
```

Add to `processing/src/lib.rs`:
```rust
pub mod stats;
```

Then run:
```bash
cargo test --workspace -- test_book_stats
git add processing/src/stats/mod.rs processing/src/lib.rs
git commit -m "R16b-T03: book statistics — all stats tests green"
```

---

## R16b-T04

**IMPORTANT — correct file:** All edits go in `src-tauri/src/commands.rs` (the large monolithic file, ~line 1560). Do NOT edit `src-tauri/src/commands/convert.rs` — that file is orphaned and never compiled.

In `src-tauri/src/commands.rs`, update the existing `convert_book` block to add KEPUB:
```rust
// Update the import line to add kepub:
use xcalibre_processing::convert::{docx, html, txt, pdf, mobi, kepub, fb2, rtf, htmlz};

// Add to OutputFormat enum:
Kepub,

// Add to out_path match:
OutputFormat::Kepub => dir.join(format!("{stem}.kepub.epub")),

// Add to conversion dispatch match:
OutputFormat::Kepub => kepub::epub_to_kepub(&epub, &out_path).map_err(|e| e.to_string())?,
```

Then append these two commands to `src-tauri/src/commands.rs`:
```rust
#[tauri::command]
pub async fn get_book_stats(
    file_path: String,
) -> Result<xcalibre_processing::stats::BookStats, String> {
    xcalibre_processing::stats::compute_book_stats(std::path::Path::new(&file_path))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn pretty_print_epub_cmd(
    epub_path: String,
    out_path:  String,
) -> Result<(), String> {
    xcalibre_processing::polish::pretty_print_epub(
        std::path::Path::new(&epub_path),
        std::path::Path::new(&out_path),
    ).map_err(|e| e.to_string())
}
```

Register both in `src-tauri/src/main.rs` in the `.invoke_handler(tauri::generate_handler![...])` list:
```rust
commands::get_book_stats,
commands::pretty_print_epub_cmd,
```

```bash
cargo check --workspace 2>&1 | grep "^error"
git add src-tauri/src/commands.rs src-tauri/src/main.rs
git commit -m "R16b-T04: get_book_stats + pretty_print_epub_cmd + KEPUB in convert_book"
```

---

## R16b-T05

Write `ui/src/components/BookStatsPanel.tsx`:
```tsx
import { useState, useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"

interface Book {
  id: string
  title: string
  authors: string[]
  format: string
  cover_path: string | null
  progress_percent: number
  last_opened_at: string | null
  file_path?: string
}

interface BookStats {
  word_count:           number
  character_count:      number
  page_count_estimate:  number
  reading_time_minutes: number
}

interface Props { book: Book }

function fmtNum(n: number)   { return n.toLocaleString() }
function fmtTime(mins: number) {
  const h = Math.floor(mins / 60)
  const m = mins % 60
  return h > 0 ? `${h}h ${m}m` : `${m}m`
}

export function BookStatsPanel({ book }: Props) {
  const [stats,   setStats]   = useState<BookStats | null>(null)
  const [loading, setLoading] = useState(true)
  const [error,   setError]   = useState<string | null>(null)

  useEffect(() => {
    if (!book.file_path) { setLoading(false); return }
    invoke<BookStats>("get_book_stats", { filePath: book.file_path })
      .then(setStats)
      .catch(e => setError(String(e)))
      .finally(() => setLoading(false))
  }, [book.file_path])

  if (loading) {
    return <div data-testid="stats-loading" style={{ color: "var(--text-muted, #6c7086)", fontSize: "0.9rem" }}>Calculating…</div>
  }
  if (error) {
    return <div style={{ color: "var(--red, #f38ba8)", fontSize: "0.85rem" }}>Could not load stats.</div>
  }
  if (!stats) return null

  const rows = [
    { testId: "stat-word-count",    label: "Words",        value: fmtNum(stats.word_count) },
    { testId: "stat-char-count",    label: "Characters",   value: fmtNum(stats.character_count) },
    { testId: "stat-page-count",    label: "Est. Pages",   value: fmtNum(stats.page_count_estimate) },
    { testId: "stat-reading-time",  label: "Reading Time", value: fmtTime(stats.reading_time_minutes) },
  ]

  return (
    <section>
      <h3 style={{ margin: "0 0 0.75rem", fontSize: "1rem", fontWeight: 600 }}>Book Statistics</h3>
      <dl style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "0.5rem 1rem" }}>
        {rows.map(r => (
          <div key={r.testId} style={{ display: "contents" }}>
            <dt style={{ color: "var(--text-muted, #6c7086)", fontSize: "0.85rem" }}>{r.label}</dt>
            <dd data-testid={r.testId} style={{ margin: 0, fontWeight: 600 }}>{r.value}</dd>
          </div>
        ))}
      </dl>
    </section>
  )
}
```

Then run:
```bash
cd ui && npm test -- BookStatsPanel && cd ..
git add ui/src/components/BookStatsPanel.tsx
git commit -m "R16b-T05: BookStatsPanel component — all stats UI tests green"
```

---

## R16b-T06 — Milestone Check + Visual Inspection

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cd ui && npm test && cd ..
```

**Visual inspection:**
```bash
pkill -x xcalibre 2>/dev/null || true
cargo tauri dev &>/tmp/xcalibre_tauri_dev.log &
# Poll until xcalibre process appears — first-run compilation can take 3-5 min
for i in $(seq 1 30); do
  sleep 10
  if pgrep -x xcalibre > /dev/null 2>&1; then
    echo "xcalibre running after $((i*10))s"
    sleep 3
    break
  fi
  echo "Waiting for xcalibre… $((i*10))s elapsed"
  [ "$i" -eq 30 ] && echo "ERROR: xcalibre did not launch within 5 minutes" && exit 1
done
```

1. Verify the UI:
2. Open a book's detail panel → scroll to "Book Statistics" section
3. Verify word count, page count, reading time all display correctly
4. Verify reading time shows in "Xh Ym" format for long books
5. Right-click an EPUB → Convert → select "KEPUB" → verify file created as `.kepub.epub`
6. Transfer to a Kobo device (or open in Kobo Desktop) and verify it reads correctly
7. Test pretty-print: use CLI or add a "Polish EPUB" button — verify output is indented XML

```bash
pkill -x xcalibre 2>/dev/null || true
```

```bash
git add -A
git commit -m "R16b-T06: RMP-16 OEB Polish Toolkit — all tests green, UI wired"
```
