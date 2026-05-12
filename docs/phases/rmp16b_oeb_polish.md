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

    let manifest = container.manifest_items()
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    let mut new_container = container.clone_empty()
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    // Copy all files; transform HTML/XHTML spine items
    for (id, href, media_type) in &manifest {
        let bytes = container.read_item(href)
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

        let out_bytes = if media_type.contains("html") || media_type.contains("xhtml") {
            let html = String::from_utf8_lossy(&bytes);
            let kepub_html = inject_kobo_spans(&html);
            kepub_html.into_bytes()
        } else {
            bytes
        };

        new_container.write_item(href, &out_bytes)
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    }

    new_container.save(out_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    Ok(())
}

fn inject_kobo_spans(html: &str) -> String {
    // Add xmlns:epub and xmlns:kobo to <html> or <body> tag if not present
    // Wrap each text paragraph block in a kobo:span
    // This is a simplified implementation: wrap <p> content in spans

    let mut result = String::with_capacity(html.len() + html.len() / 4);
    let mut paragraph_id = 0u32;

    // Inject namespace into <html> tag
    let html = if html.contains("xmlns:kobo") {
        html.to_string()
    } else {
        html.replacen(
            "<html",
            "<html xmlns:epub=\"http://www.idpf.org/2007/ops\" xmlns:kobo=\"http://koboapp.com/ns\"",
            1,
        )
    };

    // Wrap <p>...</p> content in kobo spans
    let p_re = regex::Regex::new(r"(<p[^>]*>)(.*?)(</p>)").unwrap();
    let output = p_re.replace_all(&html, |caps: &regex::Captures| {
        paragraph_id += 1;
        let open  = &caps[1];
        let body  = &caps[2];
        let close = &caps[3];
        format!(
            r#"{}<span class="kobo-span" id="kobo.{paragraph_id}.1" epub:type="chapter">{}</span>{}"#,
            open, body, close
        )
    });

    output.to_string()
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

    let manifest = container.manifest_items()
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    let mut new_container = container.clone_empty()
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    for (_, href, media_type) in &manifest {
        let bytes = container.read_item(href)
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

        let out_bytes = if media_type.contains("html") || media_type.contains("xhtml")
                         || media_type.contains("xml") || media_type.contains("css")
        {
            let text = String::from_utf8_lossy(&bytes);
            pretty_print_xml(&text).into_bytes()
        } else {
            bytes
        };

        new_container.write_item(href, &out_bytes)
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
    }

    new_container.save(out_path)
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))
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

    let spine = container.spine_items()
        .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;

    let mut total_text = String::new();

    for href in &spine {
        let bytes = container.read_item(href)
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
        let html = String::from_utf8_lossy(&bytes);
        let text = strip_html(&html);
        total_text.push_str(&text);
        total_text.push(' ');
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

Add to `src-tauri/src/commands/convert.rs`:
```rust
use xcalibre_processing::convert::kepub;
use xcalibre_processing::polish;
use xcalibre_processing::stats::compute_book_stats;

// Add KEPUB to OutputFormat enum:
Kepub,

// In convert_book:
OutputFormat::Kepub => ("kepub.epub", dir.join(format!("{stem}.kepub.epub"))),
// In conversion match:
OutputFormat::Kepub => kepub::epub_to_kepub(&epub, &out_path).map_err(|e| e.to_string())?,

#[tauri::command]
pub async fn get_book_stats(
    file_path: String,
    _state: State<'_, AppState>,
) -> Result<xcalibre_processing::stats::BookStats, String> {
    compute_book_stats(std::path::Path::new(&file_path))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn pretty_print_epub_cmd(
    epub_path: String,
    out_path:  String,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    let epub = std::path::Path::new(&epub_path);
    let out  = std::path::Path::new(&out_path);
    polish::pretty_print_epub(epub, out).map_err(|e| e.to_string())
}
```

Register `get_book_stats` and `pretty_print_epub_cmd` in `src-tauri/src/main.rs`.

```bash
cargo build --workspace
git add src-tauri/src/commands/convert.rs src-tauri/src/main.rs
git commit -m "R16b-T04: get_book_stats + convert_to_kepub Tauri commands"
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
