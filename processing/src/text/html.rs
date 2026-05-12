use crate::error::ProcessingError;
use crate::text::ExtractedText;
use std::io::Read;
use std::path::Path;
use std::sync::OnceLock;

static SEL_CONTENT: OnceLock<scraper::Selector> = OnceLock::new();
fn sel_content() -> &'static scraper::Selector {
    SEL_CONTENT.get_or_init(|| scraper::Selector::parse("p, h1, h2, h3, h4, li, td, th").expect("selector"))
}

pub fn extract(path: &Path) -> Result<ExtractedText, ProcessingError> {
    let content = read_html(path)?;
    let document = scraper::Html::parse_document(&content);

    let sel = sel_content();
    let mut parts = vec![];
    for el in document.select(sel) {
        let text = el.text().collect::<String>();
        let trimmed = text.split_whitespace().collect::<Vec<_>>().join(" ");
        if !trimmed.is_empty() {
            parts.push(trimmed);
        }
    }

    let full_text = parts.join("\n");
    let word_count = full_text.split_whitespace().count();
    Ok(ExtractedText { full_text, word_count })
}

fn read_html(path: &Path) -> Result<String, ProcessingError> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    if ext == "htmlz" {
        let file = std::fs::File::open(path).map_err(ProcessingError::IoError)?;
        let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file))
            .map_err(|e| ProcessingError::TextError(e.to_string()))?;
        let mut html_file = archive
            .by_name("index.html")
            .map_err(|e| ProcessingError::TextError(e.to_string()))?;
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
