use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use std::io::Read;
use std::path::Path;

/// Parse a standalone OPF sidecar file (Calibre's metadata.opf).
/// Reuses the same OPF field mapping as the EPUB extractor.
pub fn parse_opf(path: &Path) -> Result<BookMetadata, ProcessingError> {
    let mut file = std::fs::File::open(path).map_err(ProcessingError::IoError)?;
    let mut xml = String::new();
    file.read_to_string(&mut xml).map_err(ProcessingError::IoError)?;

    let doc = roxmltree::Document::parse(&xml)
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;

    let mut meta = BookMetadata::default();
    for node in doc.descendants() {
        match node.tag_name().name() {
            "title" => meta.title = node.text().map(str::trim).map(String::from),
            "creator" => meta.authors.push(node.text().unwrap_or("").trim().to_string()),
            "language" => meta.language = node.text().map(str::trim).map(String::from),
            "publisher" => meta.publisher = node.text().map(str::trim).map(String::from),
            "date" => meta.published = node.text().map(str::trim).map(String::from),
            "description" => meta.description = node.text().map(str::trim).map(String::from),
            "subject" => {
                if let Some(t) = node.text() {
                    let t = t.trim().to_string();
                    if !t.is_empty() {
                        meta.tags.push(t);
                    }
                }
            }
            "identifier" => {
                let scheme = node
                    .attribute("opf:scheme")
                    .or_else(|| node.attribute("scheme"))
                    .unwrap_or("");
                if scheme.eq_ignore_ascii_case("isbn") {
                    meta.isbn = node.text().map(str::trim).map(String::from);
                }
            }
            "meta" => {
                let name = node.attribute("name").unwrap_or("");
                let content = node.attribute("content").unwrap_or("");
                match name {
                    "calibre:series" => meta.series = Some(content.to_string()),
                    "calibre:series_index" => meta.series_index = content.parse().ok(),
                    _ => {}
                }
            }
            _ => {}
        }
    }
    meta.authors.retain(|a| !a.is_empty());
    Ok(meta)
}
