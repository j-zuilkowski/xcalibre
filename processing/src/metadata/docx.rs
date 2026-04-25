use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use std::io::Read;
use std::path::Path;

/// DOCX (Office Open XML) is a ZIP containing word/document.xml and docProps/core.xml.
pub fn extract(path: &Path) -> Result<BookMetadata, ProcessingError> {
    let file = std::fs::File::open(path).map_err(ProcessingError::IoError)?;
    let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file))
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;

    let core_xml = read_entry(&mut archive, "docProps/core.xml").unwrap_or_default();
    if core_xml.is_empty() {
        return Ok(BookMetadata::default());
    }

    let doc = roxmltree::Document::parse(&core_xml)
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;

    let mut title = None;
    let mut authors = Vec::new();
    let mut description = None;
    let mut language = None;
    let mut tags = Vec::new();

    for node in doc.descendants().filter(|n| n.is_element()) {
        match node.tag_name().name() {
            "title" => title = node.text().map(str::trim).map(String::from),
            "creator" => {
                let author = node.text().unwrap_or("").trim().to_string();
                if !author.is_empty() {
                    authors.push(author);
                }
            }
            "subject" => {
                if let Some(subject) = node.text() {
                    let subject = subject.trim().to_string();
                    if !subject.is_empty() {
                        tags.push(subject);
                    }
                }
            }
            "description" => description = node.text().map(str::trim).map(String::from),
            "language" => language = node.text().map(str::trim).map(String::from),
            _ => {}
        }
    }

    authors.retain(|author| !author.is_empty());

    Ok(BookMetadata {
        title,
        authors,
        language,
        description,
        tags,
        ..BookMetadata::default()
    })
}

fn read_entry<R: std::io::Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    name: &str,
) -> Option<String> {
    let mut entry = archive.by_name(name).ok()?;
    let mut s = String::new();
    entry.read_to_string(&mut s).ok()?;
    Some(s)
}
