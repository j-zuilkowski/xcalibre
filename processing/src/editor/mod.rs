use crate::error::ProcessingError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use xcalibre_epub::Container;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestItem {
    pub id:         String,
    pub href:       String,
    pub media_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorMetadata {
    pub title:       Option<String>,
    pub authors:     Vec<String>,
    pub language:    Option<String>,
    pub publisher:   Option<String>,
    pub description: Option<String>,
}

pub struct EpubEditor {
    container: Container,
    _path:     PathBuf,
}

impl EpubEditor {
    pub fn open(path: &Path) -> Result<Self, ProcessingError> {
        let container = Container::open(path)
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))?;
        Ok(Self { container, _path: path.to_path_buf() })
    }

    pub fn spine_items(&self) -> Vec<String> {
        self.container.spine_hrefs()
    }

    pub fn manifest_items(&self) -> Vec<ManifestItem> {
        self.container.manifest_items()
            .into_iter()
            .map(|(id, href, media_type)| ManifestItem { id, href, media_type })
            .collect()
    }

    pub fn read_item(&self, href: &str) -> Result<Vec<u8>, ProcessingError> {
        self.container.read_item(href)
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))
    }

    pub fn write_item(&mut self, href: &str, content: &[u8]) -> Result<(), ProcessingError> {
        let mime = guess_mime(href);
        self.container.write_item(href, content, mime)
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))
    }

    pub fn metadata(&self) -> EditorMetadata {
        let opf_path = self.container.opf_path();
        let opf_bytes = self.container.read_item(opf_path).unwrap_or_default();
        let opf_str = String::from_utf8_lossy(&opf_bytes);
        let opf = xcalibre_epub::opf::EpubOPF::parse(&opf_str).unwrap_or_default();
        EditorMetadata {
            title:       opf.title,
            authors:     opf.authors,
            language:    opf.language,
            publisher:   opf.publisher,
            description: None,
        }
    }

    pub fn set_title(&mut self, title: &str) {
        let opf_path = self.container.opf_path().to_string();
        let opf_bytes = self.container.read_item(&opf_path).unwrap_or_default();
        let opf_str = String::from_utf8_lossy(&opf_bytes);
        if let Ok(mut opf) = xcalibre_epub::opf::EpubOPF::parse(&opf_str) {
            opf.title = Some(title.to_string());
            if let Ok(xml) = opf.to_xml() {
                let _ = self.container.write_item(&opf_path, xml.as_bytes(), "application/oebps-package+xml");
            }
        }
    }

    pub fn set_authors(&mut self, authors: &[&str]) {
        let opf_path = self.container.opf_path().to_string();
        let opf_bytes = self.container.read_item(&opf_path).unwrap_or_default();
        let opf_str = String::from_utf8_lossy(&opf_bytes);
        if let Ok(mut opf) = xcalibre_epub::opf::EpubOPF::parse(&opf_str) {
            opf.authors = authors.iter().map(|s| s.to_string()).collect();
            if let Ok(xml) = opf.to_xml() {
                let _ = self.container.write_item(&opf_path, xml.as_bytes(), "application/oebps-package+xml");
            }
        }
    }

    pub fn cover_bytes(&self) -> Result<Vec<u8>, ProcessingError> {
        // Try manifest items first
        let items = self.container.manifest_items();
        for (_, href, mt) in &items {
            if mt.starts_with("image/") && (href.to_lowercase().contains("cover")) {
                if let Ok(data) = self.container.read_item(href) {
                    if !data.is_empty() { return Ok(data); }
                }
            }
        }
        // Try reading cover.jpeg directly
        if let Ok(data) = self.container.read_item("cover.jpeg") {
            if !data.is_empty() { return Ok(data); }
        }
        // Fallback: return first image from manifest
        for (_, href, mt) in &items {
            if mt.starts_with("image/") {
                if let Ok(data) = self.container.read_item(href) {
                    if !data.is_empty() { return Ok(data); }
                }
            }
        }
        Ok(vec![])
    }

    pub fn set_cover(&mut self, data: &[u8], mime: &str) -> Result<(), ProcessingError> {
        // Try to find existing cover item, otherwise create new
        let items = self.container.manifest_items();
        let mut cover_href = "cover.jpeg".to_string();
        for (_, href, mt) in &items {
            if mt.starts_with("image/") && (href.contains("cover") || href.contains("Cover")) {
                cover_href = href.clone();
                break;
            }
        }
        self.container.write_item(&cover_href, data, mime)
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))
    }

    pub fn save(&mut self) -> Result<(), ProcessingError> {
        self.container.save()
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))
    }

    pub fn save_as(&self, dest: &Path) -> Result<(), ProcessingError> {
        self.container.save_as(dest)
            .map_err(|e| ProcessingError::ConversionError(e.to_string()))
    }
}

fn guess_mime(href: &str) -> &str {
    if href.ends_with(".xhtml") || href.ends_with(".html") { "application/xhtml+xml" }
    else if href.ends_with(".css") { "text/css" }
    else if href.ends_with(".jpg") || href.ends_with(".jpeg") { "image/jpeg" }
    else if href.ends_with(".png") { "image/png" }
    else { "application/octet-stream" }
}
