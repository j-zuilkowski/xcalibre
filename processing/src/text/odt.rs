use crate::error::ProcessingError;
use crate::text::ExtractedText;
use std::io::Read;
use std::path::Path;

pub fn extract(path: &Path) -> Result<ExtractedText, ProcessingError> {
    let file = std::fs::File::open(path).map_err(ProcessingError::IoError)?;
    let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file))
        .map_err(|e| ProcessingError::TextError(e.to_string()))?;

    let mut xml_str = String::new();
    if let Ok(mut entry) = archive.by_name("content.xml") {
        entry
            .read_to_string(&mut xml_str)
            .map_err(ProcessingError::IoError)?;
    } else {
        return Ok(ExtractedText {
            full_text: String::new(),
            word_count: 0,
        });
    }

    let doc = roxmltree::Document::parse(&xml_str)
        .map_err(|e| ProcessingError::TextError(e.to_string()))?;

    let mut parts = Vec::new();
    for paragraph in doc
        .descendants()
        .filter(|node| node.is_element() && node.tag_name().name() == "p")
    {
        let text = paragraph
            .descendants()
            .filter_map(|node| node.text())
            .collect::<Vec<_>>()
            .join(" ");
        let trimmed = text.split_whitespace().collect::<Vec<_>>().join(" ");
        if !trimmed.is_empty() {
            parts.push(trimmed);
        }
    }

    let full_text = parts.join("\n");
    let word_count = full_text.split_whitespace().count();
    Ok(ExtractedText {
        full_text,
        word_count,
    })
}
