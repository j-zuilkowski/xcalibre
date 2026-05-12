use crate::error::ProcessingError;
use crate::text::ExtractedText;
use std::path::Path;
use std::sync::OnceLock;

static RE_CHM_TAGS: OnceLock<regex::Regex> = OnceLock::new();
fn re_chm_tags() -> &'static regex::Regex {
    RE_CHM_TAGS
        .get_or_init(|| regex::Regex::new(r"<[^>]*>").expect("regex"))
}

/// CHM text extraction via `chmlib` ITSF container traversal.
/// Iterates all objects in the CHM with `.htm` or `.html` extensions,
/// strips HTML tags, and concatenates the result.
/// Falls back to recover_readable_text() on any parse failure.
pub fn extract(path: &Path) -> Result<ExtractedText, ProcessingError> {
    let mut all_text = String::new();

    // Attempt ITSF container parse.
    if let Ok(mut chm) = chmlib::ChmFile::open(path) {
        // Collect HTML file paths first (can't mutate chm during for_each iteration
        // since the callback receives &mut ChmFile).
        let mut html_paths: Vec<std::path::PathBuf> = Vec::new();

        let _ = chm.for_each(chmlib::Filter::all(), |_chm, unit| {
            if unit.is_file() {
                if let Some(p) = unit.path() {
                    let name = p.to_string_lossy().to_lowercase();
                    if name.ends_with(".htm") || name.ends_with(".html") {
                        html_paths.push(p.to_path_buf());
                    }
                }
            }
            chmlib::Continuation::Continue
        });

        // Now read each HTML file
        for ref html_path in &html_paths {
            if let Some(unit) = chm.find(html_path) {
                let buf_len = unit.length() as usize;
                if buf_len > 0 && buf_len < 10_000_000 {
                    let mut buf = vec![0u8; buf_len];
                    if chm.read(&unit, 0, &mut buf).is_ok() {
                        if let Ok(html_str) = std::str::from_utf8(&buf) {
                            let stripped = strip_html_tags(html_str);
                            if !stripped.trim().is_empty() {
                                if !all_text.is_empty() {
                                    all_text.push('\n');
                                }
                                all_text.push_str(&stripped);
                            }
                        }
                    }
                }
            }
        }
    }

    // Fall back to embedded-HTML scan if container traversal produced nothing.
    // CHM is a container format so the generic recover_readable_text() heuristic
    // (which tries UTF-16 LE and full-bytes interpretations) produces garbage for
    // CHM headers. Instead we scan the raw bytes for recognisable HTML fragments
    // only — this handles malformed or partially-valid files that embed raw HTML.
    if all_text.trim().is_empty() {
        all_text = recover_chm_text_from_raw(path)?;
    }

    let word_count = all_text.split_whitespace().count();
    Ok(ExtractedText {
        full_text: all_text,
        word_count,
    })
}

/// CHM-specific raw-bytes fallback. Avoids the generic recover_readable_text()
/// which tries UTF-16 LE decoding and produces garbage CJK output for CHM headers.
///
/// Strategy:
/// 1. If the bytes contain recognisable HTML markup, strip tags and return the text.
/// 2. Otherwise, extract ASCII strings and return them only if enough real words
///    are present — this filters out files that only contain binary magic bytes
///    such as "ITSF" without any actual content.
fn recover_chm_text_from_raw(path: &Path) -> Result<String, ProcessingError> {
    let bytes = std::fs::read(path).map_err(ProcessingError::IoError)?;
    let raw = String::from_utf8_lossy(&bytes);

    // Path 1: embedded HTML — strip tags and return.
    if raw.contains("<html") || raw.contains("<body") || raw.contains("<p>") || raw.contains("<div") {
        let stripped = strip_html_tags(&raw);
        let text: String = stripped
            .split_whitespace()
            .filter(|w| w.chars().filter(|c| c.is_alphabetic()).count() >= 2)
            .collect::<Vec<_>>()
            .join(" ");
        return Ok(text);
    }

    // Path 2: ASCII string recovery with a content threshold.
    // Collect words that look like real text (≥ 3 alphabetic chars).
    // Only return content if we find ≥ 2 such words, which distinguishes
    // a header-only file (just "ITSF" + symbols) from a file with real content.
    let words: Vec<&str> = raw
        .split(|c: char| !c.is_ascii_graphic() && !c.is_ascii_whitespace())
        .flat_map(|chunk| chunk.split_whitespace())
        .filter(|w| w.chars().filter(|c| c.is_alphabetic()).count() >= 3)
        .collect();

    if words.len() >= 2 {
        return Ok(words.join(" "));
    }

    Ok(String::new())
}

fn strip_html_tags(html: &str) -> String {
    let stripped = re_chm_tags().replace_all(html, " ");
    stripped
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
