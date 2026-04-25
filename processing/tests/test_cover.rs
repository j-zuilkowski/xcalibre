use xcalibre_processing::cover::{epub, resize, CoverResult};
use std::path::PathBuf;

#[test]
fn test_epub_cover_extract() {
    let path = PathBuf::from("tests/fixtures/fixture_epub.epub");
    let result = epub::extract(&path);
    assert!(result.is_ok());
}

#[test]
fn test_resize_cover() {
    let img = image::DynamicImage::new_rgb8(1000, 1500);
    let mut bytes = std::io::Cursor::new(Vec::new());
    img.write_to(&mut bytes, image::ImageFormat::Jpeg).unwrap();
    let cover = CoverResult {
        data:      bytes.into_inner(),
        mime_type: "image/jpeg".to_string(),
        width:     1000,
        height:    1500,
    };
    let resized = resize::resize_cover(cover).unwrap();
    assert!(resized.width  <= 500);
    assert!(resized.height <= 750);
}