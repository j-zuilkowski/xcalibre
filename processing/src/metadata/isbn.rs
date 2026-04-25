use std::io::Read;
use std::path::Path;

pub fn from_epub(path: &Path) -> Option<String> {
    let file = std::fs::File::open(path).ok()?;
    let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file)).ok()?;

    let container_xml = {
        let mut entry = archive.by_name("META-INF/container.xml").ok()?;
        let mut s = String::new();
        entry.read_to_string(&mut s).ok()?;
        s
    };

    let opf_path = {
        let doc = roxmltree::Document::parse(&container_xml).ok()?;
        doc.descendants()
            .find(|n| n.tag_name().name() == "rootfile")
            .and_then(|n| n.attribute("full-path"))
            .map(String::from)?
    };

    let opf_xml = {
        let mut entry = archive.by_name(&opf_path).ok()?;
        let mut s = String::new();
        entry.read_to_string(&mut s).ok()?;
        s
    };

    let doc = roxmltree::Document::parse(&opf_xml).ok()?;
    for node in doc.descendants() {
        if node.tag_name().name() == "identifier" {
            let scheme = node
                .attribute("opf:scheme")
                .or_else(|| node.attribute("scheme"))
                .unwrap_or("");
            if let Some(raw) = node.text() {
                if scheme.eq_ignore_ascii_case("isbn")
                    || raw.to_ascii_lowercase().contains("isbn")
                {
                    return validate_isbn(raw);
                }
            }
        }
    }
    None
}

pub fn from_pdf(path: &Path) -> Option<String> {
    let mut file = std::fs::File::open(path).ok()?;
    let mut buf = [0u8; 8192];
    let n = std::io::Read::read(&mut file, &mut buf).ok()?;
    let text = String::from_utf8_lossy(&buf[..n]);

    let re = regex::Regex::new(r"ISBN[:\s\-]*([0-9\-]{10,17})").ok()?;
    for cap in re.captures_iter(&text) {
        if let Some(raw) = cap.get(1) {
            if let Some(isbn) = validate_isbn(raw.as_str()) {
                return Some(isbn);
            }
        }
    }
    None
}

fn validate_isbn(raw: &str) -> Option<String> {
    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() == 10 || digits.len() == 13 {
        Some(digits)
    } else {
        None
    }
}
