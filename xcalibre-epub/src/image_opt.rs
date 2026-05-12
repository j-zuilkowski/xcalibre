use crate::{Container, EpubError};
use image::imageops::FilterType;

pub struct ImageOptConfig { pub max_width: u32, pub max_height: u32, pub jpeg_quality: u8 }
impl Default for ImageOptConfig {
    fn default() -> Self { Self { max_width: 1920, max_height: 1080, jpeg_quality: 85 } }
}

/// Rescale all images in the container exceeding max_width × max_height.
pub fn optimize_images(container: &mut Container, cfg: &ImageOptConfig) -> Result<u32, EpubError> {
    let image_hrefs: Vec<String> = container.manifest_items().into_iter()
        .filter(|(_, _, mt)| mt.starts_with("image/"))
        .map(|(_, href, _)| href)
        .collect();
    let mut count = 0u32;
    for href in image_hrefs {
        if let Ok(data) = container.read_item(&href) {
            if let Ok(img) = image::load_from_memory(&data) {
                if img.width() > cfg.max_width || img.height() > cfg.max_height {
                    let resized = img.resize(cfg.max_width, cfg.max_height, FilterType::Lanczos3);
                    let mut buf = std::io::Cursor::new(Vec::new());
                    resized.write_to(&mut buf, image::ImageFormat::Jpeg)
                        .map_err(|e| EpubError::Io(std::io::Error::other(e)))?;
                    container.write_item(&href, &buf.into_inner(), "image/jpeg")?;
                    count += 1;
                }
            }
        }
    }
    Ok(count)
}
