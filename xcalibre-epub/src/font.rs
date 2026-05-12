use crate::{Container, EpubError};

#[derive(Debug)]
pub struct FontInfo { pub href: String, pub media_type: String }

/// List all font entries in the EPUB manifest.
pub fn list_fonts(container: &Container) -> Vec<FontInfo> {
    let font_types = [
        "font/ttf", "font/otf", "application/font-sfnt",
        "application/vnd.ms-opentype", "font/woff", "font/woff2",
    ];
    container.manifest_items()
        .into_iter()
        .filter(|(_, _, mt)| font_types.iter().any(|ft| mt.eq_ignore_ascii_case(ft)))
        .map(|(_, href, mt)| FontInfo { href, media_type: mt })
        .collect()
}

/// Embed a font into the EPUB container.
/// Writes to `Fonts/<name>` and adds a manifest entry if it doesn't already exist.
pub fn embed_font(container: &mut Container, name: &str, data: &[u8], media_type: &str)
    -> Result<(), EpubError>
{
    let href = format!("Fonts/{}", name);
    container.write_item(&href, data, media_type)?;
    Ok(())
}
