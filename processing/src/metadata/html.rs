use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use std::io::Read;
use std::path::Path;
use std::sync::OnceLock;

static SEL_TITLE: OnceLock<scraper::Selector> = OnceLock::new();
fn sel_title() -> &'static scraper::Selector {
    SEL_TITLE.get_or_init(|| scraper::Selector::parse("title").expect("selector"))
}

static SEL_META: OnceLock<scraper::Selector> = OnceLock::new();
fn sel_meta() -> &'static scraper::Selector {
    SEL_META.get_or_init(|| scraper::Selector::parse("meta").expect("selector"))
}

pub fn extract(path: &Path) -> Result<BookMetadata, ProcessingError> {
    let content = read_html(path)?;
    let document = scraper::Html::parse_document(&content);
    let mut meta = BookMetadata::default();

    let title_sel = sel_title();
    if let Some(el) = document.select(title_sel).next() {
        let t = el.text().collect::<String>().trim().to_string();
        if !t.is_empty() {
            meta.title = Some(t);
        }
    }

    let meta_sel = sel_meta();
    for el in document.select(meta_sel) {
        let name = el.value().attr("name").unwrap_or("").to_lowercase();
        let content = el.value().attr("content").unwrap_or("").trim();
        match name.as_str() {
            "author" => {
                for a in content.split(',') {
                    let a = a.trim().to_string();
                    if !a.is_empty() {
                        meta.authors.push(a);
                    }
                }
            }
            "description" => meta.description = Some(content.to_string()),
            "keywords" => {
                for k in content.split(',') {
                    let k = k.trim().to_string();
                    if !k.is_empty() {
                        meta.tags.push(k);
                    }
                }
            }
            _ => {}
        }
    }
    Ok(meta)
}

fn read_html(path: &Path) -> Result<String, ProcessingError> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    if ext == "htmlz" {
        let file = std::fs::File::open(path).map_err(ProcessingError::IoError)?;
        let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file))
            .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;
        let mut html_file = archive
            .by_name("index.html")
            .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;
        let mut s = String::new();
        html_file.read_to_string(&mut s).map_err(ProcessingError::IoError)?;
        Ok(s)
    } else {
        let mut file = std::fs::File::open(path).map_err(ProcessingError::IoError)?;
        let mut s = String::new();
        file.read_to_string(&mut s).map_err(ProcessingError::IoError)?;
        Ok(s)
    }
}
