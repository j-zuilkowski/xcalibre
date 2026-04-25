use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use std::path::Path;

pub fn extract(path: &Path) -> Result<BookMetadata, ProcessingError> {
    let data = std::fs::read(path).map_err(ProcessingError::IoError)?;

    let book = mobi::Mobi::new(&data)
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;

    let mut meta = BookMetadata {
        title: Some(book.title()).filter(|s| !s.is_empty()),
        publisher: book.publisher().filter(|s| !s.is_empty()),
        language: Some(format!("{:?}", book.language())).filter(|s| !s.is_empty()),
        published: book.publish_date().filter(|s| !s.is_empty()),
        description: book.description().filter(|s| !s.is_empty()),
        isbn: book.isbn().filter(|s| !s.is_empty()),
        ..BookMetadata::default()
    };

    if let Some(author) = book.author() {
        for a in author.split(&['&', ';'][..]) {
            let a = a.trim().to_string();
            if !a.is_empty() {
                meta.authors.push(a);
            }
        }
    }

    Ok(meta)
}
