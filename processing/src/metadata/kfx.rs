use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use rusqlite::Connection;
use std::path::Path;

pub fn extract(path: &Path) -> Result<BookMetadata, ProcessingError> {
    let conn = Connection::open(path)
        .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;

    let mut meta = BookMetadata::default();

    // KFX stores metadata in fragment_properties table
    // Standard keys: title, author, language, publisher, publication_date, description
    let mut stmt = conn.prepare(
        "SELECT key, value FROM fragment_properties
         WHERE fragment_id IN (
             SELECT id FROM fragments WHERE ftype = '$270'
         )"
    ).map_err(|e| ProcessingError::MetadataError(e.to_string()))?;

    let rows = stmt.query_map([], |row| {
        let key: String = row.get(0)?;
        let value: Option<String> = row.get(1)?;
        Ok((key, value))
    }).map_err(|e| ProcessingError::MetadataError(e.to_string()))?;

    for row in rows {
        let (key, value) = row.map_err(|e| ProcessingError::MetadataError(e.to_string()))?;
        let value = match value { Some(v) if !v.is_empty() => v, _ => continue };
        match key.as_str() {
            "title"            => meta.title       = Some(value),
            "author"           => meta.authors.push(value),
            "language"         => meta.language    = Some(value),
            "publisher"        => meta.publisher   = Some(value),
            "publication_date" => meta.published   = Some(value),
            "description"      => meta.description = Some(value),
            "isbn"             => meta.isbn        = Some(value),
            _                  => {}
        }
    }
    Ok(meta)
}
