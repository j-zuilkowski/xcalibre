use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use std::io::Read;
use std::path::Path;

/// ODT (ODF Text) is a ZIP containing content.xml and meta.xml.
pub fn extract(path: &Path) -> Result<BookMetadata, ProcessingError> {
    let file = std::fs::File::open(path).map_err(ProcessingError::IoError)?;
    let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file))
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;

    let meta_xml = read_entry(&mut archive, "meta.xml").unwrap_or_default();
    if meta_xml.is_empty() {
        return Ok(BookMetadata::default());
    }

    let doc = roxmltree::Document::parse(&meta_xml)
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;

    let mut title = None;
    let mut authors = Vec::new();
    let mut description = None;
    let mut language = None;
    let mut tags = Vec::new();

    for node in doc.descendants().filter(|node| node.is_element()) {
        match node.tag_name().name() {
            "title" => title = node.text().map(str::trim).map(String::from),
            "initial-creator" | "creator" => {
                let author = node.text().unwrap_or("").trim().to_string();
                if !author.is_empty() {
                    authors.push(author);
                }
            }
            "description" => description = node.text().map(str::trim).map(String::from),
            "language" => language = node.text().map(str::trim).map(String::from),
            "keyword" => {
                if let Some(value) = node.text() {
                    for keyword in value.split(',') {
                        let keyword = keyword.trim().to_string();
                        if !keyword.is_empty() {
                            tags.push(keyword);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    authors.retain(|author| !author.is_empty());
    authors.dedup();

    Ok(BookMetadata {
        title,
        authors,
        description,
        language,
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
