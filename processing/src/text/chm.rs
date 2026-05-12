use crate::error::ProcessingError;
use crate::text::ExtractedText;
use std::path::Path;
use std::sync::OnceLock;

static RE_CHM_TAGS: OnceLock<regex::Regex> = OnceLock::new();
fn re_chm_tags() -> &'static regex::Regex {
    RE_CHM_TAGS
        .get_or_init(|| regex::Regex::new(r"<[^>]*>").expect("regex"))
}

/// CHM text extraction via raw-bytes HTML scanning.
///
/// CHM (Microsoft HTML Help / ITSF container) embeds HTML pages. This extractor
/// scans the raw bytes for recognisable HTML markup, strips tags, and returns the
/// visible text. It handles the majority of well-formed CHM files without requiring
/// a native C library (chmlib-sys has a build-script bug on Windows).
pub fn extract(path: &Path) -> Result<ExtractedText, ProcessingError> {
    let text = recover_chm_text_from_raw(path)?;
    let word_count = text.split_whitespace().count();
    Ok(ExtractedText { full_text: text, word_count })
}

fn recover_chm_text_from_raw(path: &Path) -> Result<String, ProcessingError> {
    let bytes = std::fs::read(path).map_err(ProcessingError::IoError)?;
    let raw = String::from_utf8_lossy(&bytes);

    // Path 1: embedded HTML — strip tags and return visible text.
    if raw.contains("<html") || raw.contains("<body") || raw.contains("<p>") || raw.contains("<div") {
        let stripped = strip_html_tags(&raw);
        let text: String = stripped
            .split_whitespace()
            .filter(|w| w.chars().filter(|c| c.is_alphabetic()).count() >= 2)
            .collect::<Vec<_>>()
            .join(" ");
        return Ok(text);
    }

    // Path 2: ASCII string recovery with content threshold.
    // Only return content if we find ≥ 2 words with ≥ 3 alphabetic chars, which
    // distinguishes a header-only file (just "ITSF" + symbols) from real content.
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
    stripped.split_whitespace().collect::<Vec<_>>().join(" ")
}
