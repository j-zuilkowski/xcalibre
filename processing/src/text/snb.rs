use crate::error::ProcessingError;
use crate::text::ExtractedText;
use crate::utils::recover::recover_readable_text;
use std::io::Read;
use std::path::Path;

/// SNB is a ZIP container. Chapter content lives in `snbf/*.xml` entries.
/// Each XML file uses a simple `<snbf><ch>…<p>text</p>…</ch></snbf>` structure.
/// Strip XML tags and concatenate all chapter text.
/// Falls back to recover_readable_text() when the file is not a valid ZIP archive
/// (e.g. corrupt or non-standard SNB variants with SNBP magic only).
pub fn extract(path: &Path) -> Result<ExtractedText, ProcessingError> {
    let file = std::fs::File::open(path).map_err(ProcessingError::IoError)?;
    let mut archive = match zip::ZipArchive::new(std::io::BufReader::new(file)) {
        Ok(a) => a,
        Err(_) => {
            // Not a valid ZIP — fall back to heuristic readable-text recovery.
            let full_text = recover_readable_text(path)?;
            let word_count = full_text.split_whitespace().count();
            return Ok(ExtractedText { full_text, word_count });
        }
    };

    let mut full_text = String::new();

    // Collect snbf/*.xml entry names first (can't borrow archive mutably while iterating names).
    let names: Vec<String> = (0..archive.len())
        .filter_map(|i| archive.by_index(i).ok().map(|e: zip::read::ZipFile| e.name().to_owned()))
        .filter(|name: &String| {
            let lower = name.to_lowercase();
            lower.starts_with("snbf/") && lower.ends_with(".xml")
        })
        .collect();

    let mut sorted_names = names;
    sorted_names.sort();

    for name in &sorted_names {
        let mut entry = archive
            .by_name(name)
            .map_err(|e| ProcessingError::TextError(e.to_string()))?;
        let mut xml = String::new();
        entry.read_to_string(&mut xml).map_err(ProcessingError::IoError)?;
        let text = strip_xml_tags(&xml);
        if !text.is_empty() {
            if !full_text.is_empty() {
                full_text.push('\n');
            }
            full_text.push_str(&text);
        }
    }

    let word_count = full_text.split_whitespace().count();
    Ok(ExtractedText { full_text, word_count })
}

/// Remove all XML/HTML tags and collapse whitespace.
fn strip_xml_tags(xml: &str) -> String {
    let mut out = String::with_capacity(xml.len());
    let mut in_tag = false;
    for ch in xml.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    // Collapse runs of whitespace into single spaces.
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}
