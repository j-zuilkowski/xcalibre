use crate::cover::CoverResult;
use crate::error::ProcessingError;
use std::path::Path;

/// Attempt to extract the first embedded XObject image from a PDF.
/// Returns None if no suitable image is found (common — PDFs rarely embed covers as raw images).
pub fn extract(path: &Path) -> Result<Option<CoverResult>, ProcessingError> {
    let doc = match lopdf::Document::load(path) {
        Ok(doc) => doc,
        Err(_) => return Ok(None),
    };

    if let Some(page_id) = doc.page_iter().next() {
        if let Ok(resources) = doc.get_page_resources(page_id) {
            if let Some(xobjects) = resources.0 {
                for (_, xobj_ref) in xobjects.iter() {
                    let xobj_ref = match xobj_ref.as_reference() {
                        Ok(reference) => reference,
                        Err(e) => return Err(ProcessingError::CoverError(e.to_string())),
                    };
                    if let Ok(lopdf::Object::Stream(stream)) = doc.get_object(xobj_ref) {
                            let subtype = stream.dict.get(b"Subtype")
                                .ok()
                                .and_then(|o| o.as_name_str().ok())
                                .unwrap_or("");
                            if subtype == "Image" {
                                let data = stream.decompressed_content()
                                    .map_err(|e| ProcessingError::CoverError(e.to_string()))?;
                                if let Ok(img) = image::load_from_memory(&data) {
                                    let (w, h) = (img.width(), img.height());
                                    if w < 100 || h < 100 { continue; }
                                    return Ok(Some(CoverResult {
                                        data,
                                        mime_type: "image/jpeg".to_string(),
                                        width: w,
                                        height: h,
                                    }));
                                }
                            }
                    }
                }
            }
        }
    }
    Ok(None)
}
