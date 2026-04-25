use crate::cover::CoverResult;
use crate::error::ProcessingError;
use std::io::Read;
use std::path::Path;

pub fn extract(path: &Path) -> Result<Option<CoverResult>, ProcessingError> {
    let file = std::fs::File::open(path).map_err(ProcessingError::IoError)?;
    let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file))
        .map_err(|e| ProcessingError::CoverError(e.to_string()))?;

    let opf_path = {
        let mut c = archive
            .by_name("META-INF/container.xml")
            .map_err(|e| ProcessingError::CoverError(e.to_string()))?;
        let mut xml = String::new();
        c.read_to_string(&mut xml).map_err(ProcessingError::IoError)?;
        let doc = roxmltree::Document::parse(&xml)
            .map_err(|e| ProcessingError::CoverError(e.to_string()))?;
        doc.descendants()
            .find(|n| n.tag_name().name() == "rootfile")
            .and_then(|n| n.attribute("full-path"))
            .map(String::from)
            .ok_or_else(|| ProcessingError::CoverError("no rootfile".into()))?
    };

    let opf_xml = {
        let mut f = archive
            .by_name(&opf_path)
            .map_err(|e| ProcessingError::CoverError(e.to_string()))?;
        let mut s = String::new();
        f.read_to_string(&mut s).map_err(ProcessingError::IoError)?;
        s
    };

    let opf_dir = std::path::Path::new(&opf_path)
        .parent()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();

    let doc = roxmltree::Document::parse(&opf_xml)
        .map_err(|e| ProcessingError::CoverError(e.to_string()))?;

    let cover_id = doc.descendants()
        .find(|n| n.tag_name().name() == "meta" && n.attribute("name") == Some("cover"))
        .and_then(|n| n.attribute("content"))
        .map(String::from);

    let cover_href = doc.descendants()
        .filter(|n| n.tag_name().name() == "item")
        .find(|n| {
            let matches_id = cover_id.as_deref().is_some_and(|id| n.attribute("id") == Some(id));
            let has_prop   = n.attribute("properties").is_some_and(|p| p.contains("cover-image"));
            matches_id || has_prop
        })
        .and_then(|n| n.attribute("href"))
        .map(|h| {
            if opf_dir.is_empty() { h.to_string() } else { format!("{}/{}", opf_dir, h) }
        });

    let href = match cover_href {
        Some(h) => h,
        None    => return Ok(None),
    };

    let mut bytes = Vec::new();
    archive
        .by_name(&href)
        .map_err(|e| ProcessingError::CoverError(e.to_string()))?
        .read_to_end(&mut bytes)
        .map_err(ProcessingError::IoError)?;

    let mime_type = match href.rsplit('.').next().unwrap_or("").to_lowercase().as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png"          => "image/png",
        "gif"          => "image/gif",
        "webp"         => "image/webp",
        _              => "image/jpeg",
    }.to_string();

    let img = image::load_from_memory(&bytes)
        .map_err(|e| ProcessingError::CoverError(e.to_string()))?;

    Ok(Some(CoverResult {
        data: bytes,
        mime_type,
        width:  img.width(),
        height: img.height(),
    }))
}