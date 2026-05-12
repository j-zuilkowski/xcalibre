use crate::EpubError;
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};

/// In-memory EPUB container.
pub struct Container {
    /// Path to the on-disk ZIP file.
    path: PathBuf,
    /// All entries: zip entry name → raw bytes.
    entries: HashMap<String, Vec<u8>>,
    /// OPF file path within the ZIP (from META-INF/container.xml).
    opf_path: String,
    /// Manifest: id → (href, media_type)
    manifest: Vec<(String, String, String)>,
    /// Spine hrefs in reading order.
    spine: Vec<String>,
}

impl Container {
    pub fn open(path: &Path) -> Result<Self, EpubError> {
        let file = std::fs::File::open(path).map_err(EpubError::Io)?;
        let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file))
            .map_err(|e| EpubError::Zip(e.to_string()))?;

        let mut entries: HashMap<String, Vec<u8>> = HashMap::new();
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)
                .map_err(|e| EpubError::Zip(e.to_string()))?;
            if entry.is_dir() { continue; }
            let name = entry.name().to_string();
            let mut data = Vec::new();
            entry.read_to_end(&mut data).map_err(EpubError::Io)?;
            entries.insert(name, data);
        }

        let opf_path = {
            let container_xml = entries.get("META-INF/container.xml")
                .ok_or_else(|| EpubError::MissingElement("META-INF/container.xml".into()))?;
            let xml = std::str::from_utf8(container_xml)
                .map_err(|e| EpubError::Xml(e.to_string()))?;
            let doc = roxmltree::Document::parse(xml)
                .map_err(|e| EpubError::Xml(e.to_string()))?;
            doc.descendants()
                .find(|n| n.tag_name().name() == "rootfile")
                .and_then(|n| n.attribute("full-path"))
                .map(String::from)
                .ok_or_else(|| EpubError::MissingElement("rootfile full-path".into()))?
        };

        let (manifest, spine) = parse_opf_for_manifest(&entries, &opf_path)?;

        Ok(Self { path: path.to_path_buf(), entries, opf_path, manifest, spine })
    }

    pub fn manifest_items(&self) -> Vec<(String, String, String)> {
        self.manifest.clone()
    }

    pub fn opf_path(&self) -> &str { &self.opf_path }

    pub fn spine_hrefs(&self) -> Vec<String> { self.spine.clone() }

    pub fn read_item(&self, href: &str) -> Result<Vec<u8>, EpubError> {
        let full = resolve_href(&self.opf_path, href);
        self.entries.get(&full)
            .or_else(|| self.entries.get(href))
            .cloned()
            .ok_or_else(|| EpubError::ItemNotFound(href.to_string()))
    }

    pub fn write_item(&mut self, href: &str, data: &[u8], _media_type: &str) -> Result<(), EpubError> {
        let full = resolve_href(&self.opf_path, href);
        self.entries.insert(full, data.to_vec());
        Ok(())
    }

    pub fn remove_item(&mut self, href: &str) -> Result<(), EpubError> {
        let full = resolve_href(&self.opf_path, href);
        self.entries.remove(&full)
            .or_else(|| self.entries.remove(href))
            .map(|_| ())
            .ok_or_else(|| EpubError::ItemNotFound(href.to_string()))
    }

    pub fn save(&mut self) -> Result<(), EpubError> {
        self.write_zip(&self.path.clone())
    }

    pub fn save_as(&self, path: &Path) -> Result<(), EpubError> {
        self.write_zip(path)
    }

    fn write_zip(&self, path: &Path) -> Result<(), EpubError> {
        let tmp = path.with_extension("epub.tmp");
        {
            let f = std::fs::File::create(&tmp).map_err(EpubError::Io)?;
            let mut z = zip::ZipWriter::new(f);

            // mimetype MUST be first and stored (uncompressed)
            let stored = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);
            let deflated = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);

            if let Some(mt) = self.entries.get("mimetype") {
                z.start_file("mimetype", stored).map_err(|e| EpubError::Zip(e.to_string()))?;
                use std::io::Write;
                z.write_all(mt).map_err(EpubError::Io)?;
            }
            for (name, data) in &self.entries {
                if name == "mimetype" { continue; }
                let opts = if name.ends_with(".xhtml") || name.ends_with(".html")
                    || name.ends_with(".css") || name.ends_with(".opf")
                    || name.ends_with(".xml") || name.ends_with(".ncx")
                { deflated } else { deflated };
                z.start_file(name, opts).map_err(|e| EpubError::Zip(e.to_string()))?;
                use std::io::Write;
                z.write_all(data).map_err(EpubError::Io)?;
            }
            z.finish().map_err(|e| EpubError::Zip(e.to_string()))?;
        }
        std::fs::rename(&tmp, path).map_err(EpubError::Io)
    }
}

fn resolve_href(opf_path: &str, href: &str) -> String {
    if href.starts_with('/') || href.contains("://") { return href.to_string(); }
    let opf_dir = Path::new(opf_path).parent()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();
    if opf_dir.is_empty() { href.to_string() } else { format!("{}/{}", opf_dir, href) }
}

fn parse_opf_for_manifest(
    entries: &HashMap<String, Vec<u8>>, opf_path: &str,
) -> Result<(Vec<(String, String, String)>, Vec<String>), EpubError> {
    let opf_data = entries.get(opf_path)
        .ok_or_else(|| EpubError::MissingElement(format!("OPF file: {}", opf_path)))?;
    let xml = std::str::from_utf8(opf_data).map_err(|e| EpubError::Xml(e.to_string()))?;
    let doc = roxmltree::Document::parse(xml).map_err(|e| EpubError::Xml(e.to_string()))?;

    let manifest: Vec<(String, String, String)> = doc.descendants()
        .filter(|n| n.tag_name().name() == "item")
        .filter_map(|n| {
            let id = n.attribute("id")?.to_string();
            let href = n.attribute("href")?.to_string();
            let mt = n.attribute("media-type").unwrap_or("application/octet-stream").to_string();
            Some((id, href, mt))
        })
        .collect();

    let id_to_href: HashMap<_, _> = manifest.iter()
        .map(|(id, href, _)| (id.clone(), href.clone())).collect();

    let spine: Vec<String> = doc.descendants()
        .filter(|n| n.tag_name().name() == "itemref")
        .filter_map(|n| {
            let idref = n.attribute("idref")?;
            id_to_href.get(idref).cloned()
        })
        .collect();

    Ok((manifest, spine))
}
