use crate::error::ProcessingError;
use crate::text::ExtractedText;
use regex::Regex;
use std::io::Read;
use std::path::Path;

pub fn extract(path: &Path) -> Result<ExtractedText, ProcessingError> {
    let file = std::fs::File::open(path).map_err(ProcessingError::IoError)?;
    let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file))
        .map_err(|e| ProcessingError::TextError(e.to_string()))?;

    let opf_path = {
        let mut c = archive
            .by_name("META-INF/container.xml")
            .map_err(|e| ProcessingError::TextError(e.to_string()))?;
        let mut xml = String::new();
        c.read_to_string(&mut xml).map_err(ProcessingError::IoError)?;
        let doc = roxmltree::Document::parse(&xml)
            .map_err(|e| ProcessingError::TextError(e.to_string()))?;
        doc.descendants()
            .find(|n| n.tag_name().name() == "rootfile")
            .and_then(|n| n.attribute("full-path"))
            .map(String::from)
            .ok_or_else(|| ProcessingError::TextError("no rootfile".into()))?
    };

    let opf_xml = {
        let mut f = archive
            .by_name(&opf_path)
            .map_err(|e| ProcessingError::TextError(e.to_string()))?;
        let mut s = String::new();
        f.read_to_string(&mut s).map_err(ProcessingError::IoError)?;
        s
    };

    let opf_doc = roxmltree::Document::parse(&opf_xml)
        .map_err(|e| ProcessingError::TextError(e.to_string()))?;

    let opf_dir = std::path::Path::new(&opf_path)
        .parent()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();

    let manifest: std::collections::HashMap<String, String> = opf_doc
        .descendants()
        .filter(|n| n.tag_name().name() == "item")
        .filter_map(|n| {
            let id   = n.attribute("id")?.to_string();
            let href = n.attribute("href")?.to_string();
            Some((id, href))
        })
        .collect();

    let spine_hrefs: Vec<String> = opf_doc
        .descendants()
        .filter(|n| n.tag_name().name() == "itemref")
        .filter_map(|n| {
            let idref = n.attribute("idref")?;
            let href  = manifest.get(idref)?;
            let full  = if opf_dir.is_empty() {
                href.clone()
            } else {
                format!("{}/{}", opf_dir, href)
            };
            Some(full)
        })
        .collect();

    let tag_re = Regex::new(r"<[^>]+>")
        .map_err(|e| ProcessingError::TextError(e.to_string()))?;

    let mut parts = Vec::new();
    for href in &spine_hrefs {
        if let Ok(mut f) = archive.by_name(href) {
            let mut html = String::new();
            if f.read_to_string(&mut html).is_ok() {
                let plain = tag_re.replace_all(&html, " ");
                let collapsed: String = plain.split_whitespace().collect::<Vec<_>>().join(" ");
                if !collapsed.is_empty() {
                    parts.push(collapsed);
                }
            }
        }
    }

    let full_text  = parts.join("\n\n");
    let word_count = full_text.split_whitespace().count();
    Ok(ExtractedText { full_text, word_count })
}