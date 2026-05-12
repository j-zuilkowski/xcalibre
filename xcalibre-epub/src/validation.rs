use crate::{Container, EpubError};

#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub severity: Severity,
    pub message:  String,
    pub location: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Severity { Error, Warning }

/// Run all five validation checks on an EPUB container:
/// 1. Link checker
/// 2. CSS validator  
/// 3. Font validator
/// 4. Image validator
/// 5. OPF validator
pub fn validate(container: &Container) -> Result<Vec<ValidationIssue>, EpubError> {
    let mut issues = Vec::new();
    
    // 1. Link checker: verify href/src targets exist
    let all_hrefs: Vec<String> = container.manifest_items()
        .into_iter()
        .map(|(_, href, _)| href)
        .collect();
    let spine = container.spine_hrefs();
    
    for href in &spine {
        if let Ok(data) = container.read_item(href) {
            if let Ok(html) = std::str::from_utf8(&data) {
                check_links(html, href, &all_hrefs, &mut issues);
            }
        }
    }
    
    // 2. CSS validator
    let css_items: Vec<String> = container.manifest_items()
        .into_iter()
        .filter(|(_, _, mt)| mt == "text/css")
        .map(|(_, href, _)| href)
        .collect();
    for href in &css_items {
        if let Ok(data) = container.read_item(href) {
            if let Ok(css_text) = std::str::from_utf8(&data) {
                let opens = css_text.matches('{').count();
                let closes = css_text.matches('}').count();
                if opens != closes {
                    issues.push(ValidationIssue {
                        severity: Severity::Error,
                        message: format!("Unbalanced braces in CSS ({0} vs {1})", opens, closes),
                        location: Some(href.clone()),
                    });
                }
            } else {
                issues.push(ValidationIssue {
                    severity: Severity::Error,
                    message: "CSS file is not valid UTF-8".into(),
                    location: Some(href.clone()),
                });
            }
        }
    }
    
    // 3. Font validator — pre-compile regexes
    let fonts: Vec<String> = container.manifest_items()
        .into_iter()
        .filter(|(_, _, mt)| {
            mt == "font/ttf" || mt == "font/otf" || mt == "application/font-sfnt"
                || mt == "application/vnd.ms-opentype" || mt == "font/woff" || mt == "font/woff2"
        })
        .map(|(_, href, _)| href)
        .collect();
    let font_face_re = regex::Regex::new(r"@font-face\s*\{[^}]*\}").unwrap();
    let url_re = regex::Regex::new(r#"url\(["']?([^)"']+)["']?\)"#).unwrap();
    
    for href in &css_items {
        if let Ok(data) = container.read_item(href) {
            if let Ok(css_text) = std::str::from_utf8(&data) {
                for cap in font_face_re.find_iter(css_text) {
                    let block = cap.as_str();
                    if let Some(url_match) = url_re.find(block) {
                        let url = url_match.as_str();
                        let path = url.trim_start_matches("url(")
                            .trim_start_matches('"')
                            .trim_start_matches('\'')
                            .trim_end_matches('"')
                            .trim_end_matches('\'')
                            .trim_end_matches(')');
                        if !fonts.iter().any(|f| path.contains(f.as_str()) || f.contains(path)) {
                            issues.push(ValidationIssue {
                                severity: Severity::Warning,
                                message: format!("Font {} referenced in CSS not found in manifest", path),
                                location: Some(href.clone()),
                            });
                        }
                    }
                }
            }
        }
    }
    
    // 4. Image validator: magic bytes check
    let image_items: Vec<(String, String)> = container.manifest_items()
        .into_iter()
        .filter(|(_, _, mt)| mt.starts_with("image/"))
        .map(|(_, href, mt)| (href, mt))
        .collect();
    for (href, mt) in &image_items {
        if let Ok(data) = container.read_item(href) {
            let valid = match mt.as_str() {
                "image/jpeg" => data.len() >= 2 && data[0] == 0xFF && data[1] == 0xD8,
                "image/png" => data.len() >= 8 && &data[0..8] == b"\x89PNG\r\n\x1a\n",
                "image/gif" => data.len() >= 6 && (&data[0..6] == b"GIF89a" || &data[0..6] == b"GIF87a"),
                "image/webp" => data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP",
                "image/svg+xml" => data.len() > 5 && (&data[0..5] == b"<?xml" || &data[0..5] == b"<svg " 
                    || &data[0..4] == b"<svg"),
                _ => true,
            };
            if !valid {
                issues.push(ValidationIssue {
                    severity: Severity::Error,
                    message: format!("Image magic bytes do not match declared type {}", mt),
                    location: Some(href.clone()),
                });
            }
        }
    }
    
    // 5. OPF validator
    if let Ok(opf_data) = container.read_item(container.opf_path()) {
        if let Ok(opf_xml) = std::str::from_utf8(&opf_data) {
            if !opf_xml.contains("<dc:title>") {
                issues.push(ValidationIssue {
                    severity: Severity::Error,
                    message: "OPF missing required <dc:title>".into(),
                    location: Some(container.opf_path().to_string()),
                });
            }
            let spine_count = opf_xml.matches("<itemref").count();
            if spine_count == 0 {
                issues.push(ValidationIssue {
                    severity: Severity::Error,
                    message: "OPF spine has no <itemref> entries".into(),
                    location: Some(container.opf_path().to_string()),
                });
            }
        }
    }
    
    Ok(issues)
}

fn check_links(html: &str, source: &str, all_hrefs: &[String], issues: &mut Vec<ValidationIssue>) {
    let re = regex::Regex::new(r#"(?:href|src)\s*=\s*["']([^"']+)["']"#).unwrap();
    for cap in re.captures_iter(html) {
        if let Some(link) = cap.get(1) {
            let target = link.as_str();
            if target.starts_with("http://") || target.starts_with("https://")
                || target.starts_with('#') || target.starts_with("data:")
            {
                continue;
            }
            let resolved = if target.starts_with('/') {
                target.to_string()
            } else {
                let base_dir = std::path::Path::new(source).parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();
                if base_dir.is_empty() { target.to_string() }
                else { format!("{}/{}", base_dir, target) }
            };
            if !all_hrefs.iter().any(|h| h == &resolved || h.ends_with(&format!("/{}", target))) {
                issues.push(ValidationIssue {
                    severity: Severity::Warning,
                    message: format!("Broken link: {} -> {}", source, target),
                    location: Some(source.to_string()),
                });
            }
        }
    }
}
