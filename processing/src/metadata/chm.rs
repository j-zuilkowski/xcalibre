use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use std::path::Path;

/// CHM (Microsoft HTML Help) — magic bytes: ITSF at offset 0.
/// Parses the ITSF container via the `chmlib` crate, locates the home/default
/// HTML page, and extracts <title> and <meta name="author">.
/// Falls back to recover_title() on any parse failure.
pub fn extract(path: &Path) -> Result<BookMetadata, ProcessingError> {
    let mut title = None;
    let mut authors = Vec::new();

    // Attempt ITSF container parse.
    if let Ok(mut chm) = chmlib::ChmFile::open(path) {
        // Try common default page paths.
        let candidates = [
            "/default.html",
            "/index.html",
            "/Default.html",
            "/index.htm",
        ];
        for candidate in &candidates {
            if let Some(unit) = chm.find(candidate) {
                if unit.is_file() {
                    let buf_len = unit.length() as usize;
                    if buf_len > 0 && buf_len < 10_000_000 {
                        let mut buf = vec![0u8; buf_len];
                        if chm.read(&unit, 0, &mut buf).is_ok() {
                            if let Ok(html_str) = std::str::from_utf8(&buf) {
                                let doc = scraper::Html::parse_document(html_str);
                                if let Some(title_el) = doc
                                    .select(&scraper::Selector::parse("title").unwrap())
                                    .next()
                                {
                                    let t = title_el
                                        .text()
                                        .collect::<Vec<_>>()
                                        .join("")
                                        .trim()
                                        .to_string();
                                    if !t.is_empty() {
                                        title = Some(t);
                                    }
                                }
                                for meta_el in
                                    doc.select(&scraper::Selector::parse("meta").unwrap())
                                {
                                    if let Some(name) = meta_el.value().attr("name") {
                                        if name.eq_ignore_ascii_case("author") {
                                            if let Some(content) = meta_el.value().attr("content")
                                            {
                                                authors.push(content.trim().to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                break; // Found a candidate, stop searching
            }
        }
    }

    // Fall back to existing heuristic if no HTML title was found.
    if title.is_none() {
        title = crate::utils::recover::recover_title(path)?;
    }

    Ok(BookMetadata {
        title,
        authors,
        ..BookMetadata::default()
    })
}
