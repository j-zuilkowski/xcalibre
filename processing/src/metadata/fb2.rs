use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use std::io::Read;
use std::path::Path;

pub fn extract(path: &Path) -> Result<BookMetadata, ProcessingError> {
    let mut file = std::fs::File::open(path).map_err(ProcessingError::IoError)?;
    let mut xml = String::new();
    file.read_to_string(&mut xml).map_err(ProcessingError::IoError)?;

    let doc = roxmltree::Document::parse(&xml)
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;

    let mut meta = BookMetadata::default();

    if let Some(title_info) = doc.descendants().find(|n| n.is_element() && n.tag_name().name() == "title-info") {
        for child in title_info.children().filter(|n| n.is_element()) {
            match child.tag_name().name() {
                "book-title" => meta.title = child.text().map(str::trim).map(String::from),
                "lang" => meta.language = child.text().map(str::trim).map(String::from),
                "author" => {
                    let first = child.children()
                        .find(|n| n.is_element() && n.tag_name().name() == "first-name")
                        .and_then(|n| n.text())
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    let last = child.children()
                        .find(|n| n.is_element() && n.tag_name().name() == "last-name")
                        .and_then(|n| n.text())
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    let author = match (first.is_empty(), last.is_empty()) {
                        (false, false) => format!("{} {}", first, last),
                        (true, false) => last,
                        (false, true) => first,
                        (true, true) => continue,
                    };
                    meta.authors.push(author);
                }
                "genre" => {
                    if let Some(t) = child.text() {
                        let t = t.trim().to_string();
                        if !t.is_empty() {
                            meta.tags.push(t);
                        }
                    }
                }
                "sequence" => {
                    meta.series = child.attribute("name").map(String::from);
                    meta.series_index = child.attribute("number").and_then(|n| n.parse().ok());
                }
                "annotation" => {
                    meta.description = child.text().map(str::trim).map(String::from);
                }
                _ => {}
            }
        }
    }

    if let Some(pub_info) = doc.descendants().find(|n| n.is_element() && n.tag_name().name() == "publish-info") {
        for child in pub_info.children().filter(|n| n.is_element()) {
            match child.tag_name().name() {
                "publisher" => meta.publisher = child.text().map(str::trim).map(String::from),
                "year" => meta.published = child.text().map(str::trim).map(String::from),
                "isbn" => meta.isbn = child.text().map(str::trim).map(String::from),
                _ => {}
            }
        }
    }

    Ok(meta)
}
