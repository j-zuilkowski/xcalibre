use crate::{Container, EpubError};

/// Upgrade an EPUB 2 container to EPUB 3.
/// 
/// Changes:
/// 1. OPF version attribute: 2.0 → 3.0
/// 2. Add xmlns:epub namespace to package element
/// 3. If NCX exists, generate a nav.xhtml TOC and add to manifest with properties="nav"
/// 4. Keep NCX for backwards compatibility
pub fn epub2_to_epub3(container: &mut Container) -> Result<(), EpubError> {
    let opf_path = container.opf_path().to_string();
    let opf_data = container.read_item(&opf_path)?;
    let opf_str = std::str::from_utf8(&opf_data)
        .map_err(|e| EpubError::Xml(e.to_string()))?;
    
    // 1. Change version attribute
    let updated = opf_str
        .replace("version=\"2.0\"", "version=\"3.0\"")
        .replace("version=\"2.01\"", "version=\"3.0\"");
    
    // 2. Add EPUB namespace
    let updated = if !updated.contains("xmlns:epub") {
        updated.replacen(
            "<package",
            "<package xmlns:epub=\"http://www.idpf.org/2007/ops\"",
            1,
        )
    } else {
        updated
    };
    
    // 3. Handle NCX → nav conversion
    let nav_xhtml = generate_nav_xhtml_from_ncx(container)?;
    
    // Write the nav.xhtml file
    container.write_item("nav.xhtml", nav_xhtml.as_bytes(), "application/xhtml+xml")?;
    
    // Add nav to the OPF manifest and spine
    let updated = if updated.contains("nav.xhtml") {
        updated
    } else {
        // Insert nav manifest item before </manifest>
        let nav_item = "\n    <item id=\"nav\" href=\"nav.xhtml\" media-type=\"application/xhtml+xml\" properties=\"nav\"/>";
        updated.replacen("</manifest>", &format!("{}</manifest>", nav_item), 1)
    };
    
    // Write updated OPF
    container.write_item(&opf_path, updated.as_bytes(), "application/oebps-package+xml")?;
    
    Ok(())
}

fn generate_nav_xhtml_from_ncx(container: &Container) -> Result<String, EpubError> {
    // Try to find an NCX file in the manifest
    let ncx_href = container.manifest_items()
        .into_iter()
        .find(|(_, _, mt)| mt == "application/x-dtbncx+xml")
        .map(|(_, href, _)| href);
    
    // Build a basic nav.xhtml with or without NCX content
    let mut nav = String::from(
        r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" lang="en">
<head><title>Table of Contents</title></head>
<body>
  <nav epub:type="toc">
    <h1>Table of Contents</h1>
    <ol>
"#);

    if let Some(href) = ncx_href {
        if let Ok(data) = container.read_item(&href) {
            if let Ok(ncx_str) = std::str::from_utf8(&data) {
                let doc = roxmltree::Document::parse(ncx_str)
                    .map_err(|e| EpubError::Xml(e.to_string()))?;
                
                // Walk navMap/navPoint elements
                for nav_point in doc.descendants().filter(|n| n.tag_name().name() == "navPoint") {
                    let label = nav_point.descendants()
                        .find(|n| n.tag_name().name() == "text")
                        .and_then(|n| n.text())
                        .unwrap_or("Untitled");
                    let content_src = nav_point.descendants()
                        .find(|n| n.tag_name().name() == "content")
                        .and_then(|n| n.attribute("src"))
                        .unwrap_or("#");
                    nav.push_str(&format!(
                        "      <li><a href=\"{}\">{}</a></li>\n",
                        content_src, label
                    ));
                }
            }
        }
    }
    
    nav.push_str(
r#"    </ol>
  </nav>
</body>
</html>"#);
    
    Ok(nav)
}
