use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use std::path::Path;

pub fn extract(path: &Path) -> Result<BookMetadata, ProcessingError> {
    let doc = match lopdf::Document::load(path) {
        Ok(doc) => doc,
        Err(_) => return Ok(BookMetadata::default()),
    };

    let mut meta = BookMetadata::default();

    if let Ok(info_obj) = doc.trailer.get(b"Info") {
        if let Ok(info_ref) = info_obj.as_reference() {
            if let Ok(info_obj) = doc.get_object(info_ref) {
                if let Ok(info) = info_obj.as_dict() {
                    let get_str = |dict: &lopdf::Dictionary, key: &[u8]| -> Option<String> {
                        dict.get(key).ok().and_then(|obj| match obj {
                            lopdf::Object::String(bytes, _) => {
                                Some(String::from_utf8_lossy(bytes).to_string())
                            }
                            lopdf::Object::Name(bytes) => Some(String::from_utf8_lossy(bytes).to_string()),
                            _ => None,
                        })
                    };
                    meta.title = get_str(info, b"Title");
                    meta.publisher = get_str(info, b"Creator");
                    if let Some(author) = get_str(info, b"Author") {
                        for a in author.split([';', ',']) {
                            let a = a.trim().to_string();
                            if !a.is_empty() {
                                meta.authors.push(a);
                            }
                        }
                    }
                    meta.published = get_str(info, b"CreationDate");
                }
            }
        }
    }

    Ok(meta)
}
