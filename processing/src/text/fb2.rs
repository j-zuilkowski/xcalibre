use crate::error::ProcessingError;
use crate::text::ExtractedText;
use std::io::Read;
use std::path::Path;

pub fn extract(path: &Path) -> Result<ExtractedText, ProcessingError> {
    let mut file = std::fs::File::open(path).map_err(ProcessingError::IoError)?;
    let mut xml = String::new();
    file.read_to_string(&mut xml).map_err(ProcessingError::IoError)?;

    let doc = roxmltree::Document::parse(&xml)
        .map_err(|e| ProcessingError::TextError(e.to_string()))?;

    let mut parts = vec![];
    for node in doc.descendants().filter(|n| n.is_element()) {
        match node.tag_name().name() {
            "p" | "v" | "subtitle" | "text-author" => {
                let text: String = node
                    .descendants()
                    .filter_map(|n| n.text())
                    .collect::<Vec<_>>()
                    .join(" ");
                let trimmed = text.split_whitespace().collect::<Vec<_>>().join(" ");
                if !trimmed.is_empty() {
                    parts.push(trimmed);
                }
            }
            _ => {}
        }
    }

    let full_text = parts.join("\n");
    let word_count = full_text.split_whitespace().count();
    Ok(ExtractedText { full_text, word_count })
}
