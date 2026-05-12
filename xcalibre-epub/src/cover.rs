use crate::{Container, EpubError};

#[derive(Debug)]
pub struct CoverInfo { pub href: String, pub media_type: String, pub data: Vec<u8> }

/// Extract the cover image from an EPUB container.
/// Looks for: (1) manifest item with properties="cover-image"
///            (2) <meta name="cover" content="..."/> pointing to a manifest id
pub fn extract_cover(container: &Container) -> Result<Option<CoverInfo>, EpubError> {
    // Read and parse the OPF to find cover references
    let opf_bytes = container.read_item(container.opf_path())?;
    let opf_xml = std::str::from_utf8(&opf_bytes)
        .map_err(|e| EpubError::Xml(e.to_string()))?;
    let doc = roxmltree::Document::parse(opf_xml)
        .map_err(|e| EpubError::Xml(e.to_string()))?;

    // Build a map: id → (href, media_type)
    let id_map: std::collections::HashMap<&str, (&str, &str)> = doc
        .descendants()
        .filter(|n| n.tag_name().name() == "item")
        .filter_map(|n| {
            let id = n.attribute("id")?;
            let href = n.attribute("href")?;
            let mt = n.attribute("media-type").unwrap_or("application/octet-stream");
            Some((id, (href, mt)))
        })
        .collect();

    // Strategy 1: item with properties="cover-image"
    for node in doc.descendants().filter(|n| n.tag_name().name() == "item") {
        if let Some(props) = node.attribute("properties") {
            if props.contains("cover-image") {
                if let (Some(_id), Some(href)) = (node.attribute("id"), node.attribute("href")) {
                    let mt = node.attribute("media-type").unwrap_or("image/jpeg");
                    let data = container.read_item(href)?;
                    return Ok(Some(CoverInfo {
                        href: href.to_string(),
                        media_type: mt.to_string(),
                        data,
                    }));
                }
            }
        }
    }

    // Strategy 2: <meta name="cover" content="id"/>
    for node in doc.descendants().filter(|n| n.tag_name().name() == "meta") {
        if node.attribute("name") == Some("cover") {
            if let Some(id) = node.attribute("content") {
                if let Some(&(href, mt)) = id_map.get(id) {
                    let data = container.read_item(href)?;
                    return Ok(Some(CoverInfo {
                        href: href.to_string(),
                        media_type: mt.to_string(),
                        data,
                    }));
                }
            }
        }
    }

    Ok(None)
}

/// Replace the cover image in the EPUB. Writes JPEG data to an existing cover
/// item or creates a new `cover.jpeg` entry if none exists.
pub fn replace_cover(container: &mut Container, jpeg_data: &[u8]) -> Result<(), EpubError> {
    let existing = extract_cover(container)?;
    if let Some(cov) = existing {
        container.write_item(&cov.href, jpeg_data, "image/jpeg")?;
    } else {
        container.write_item("cover.jpeg", jpeg_data, "image/jpeg")?;
    }
    Ok(())
}
