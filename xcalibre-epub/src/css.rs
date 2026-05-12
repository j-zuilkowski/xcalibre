use crate::{Container, EpubError};

/// Replace a CSS property value across all stylesheets in the container.
pub fn replace_css_property(
    container: &mut Container,
    property: &str,
    new_value: &str,
) -> Result<u32, EpubError> {
    let css_hrefs: Vec<String> = container.manifest_items().into_iter()
        .filter(|(_, _, mt)| mt == "text/css")
        .map(|(_, href, _)| href)
        .collect();
    let mut count = 0u32;
    let pattern = format!("{}:", property);
    for href in css_hrefs {
        if let Ok(data) = container.read_item(&href) {
            if let Ok(css) = std::str::from_utf8(&data) {
                let updated = rewrite_property(css, &pattern, new_value);
                if updated != css {
                    container.write_item(&href, updated.as_bytes(), "text/css")?;
                    count += 1;
                }
            }
        }
    }
    Ok(count)
}

fn rewrite_property(css: &str, property_colon: &str, new_value: &str) -> String {
    let prop_name = property_colon.trim_end_matches(':');
    let re = regex::Regex::new(&format!(
        r"(?i)({})\s*:[^;}}]+([;}}])", regex::escape(prop_name)
    )).unwrap();
    re.replace_all(css, |caps: &regex::Captures| {
        format!("{}: {}{}", &caps[1], new_value, &caps[2])
    }).into_owned()
}
