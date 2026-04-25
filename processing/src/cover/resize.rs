use crate::cover::CoverResult;
use crate::error::ProcessingError;

pub const COVER_MAX_W: u32 = 500;
pub const COVER_MAX_H: u32 = 750;

pub fn resize_cover(cover: CoverResult) -> Result<CoverResult, ProcessingError> {
    let img = image::load_from_memory(&cover.data)
        .map_err(|e| ProcessingError::CoverError(e.to_string()))?;

    let resized = img.resize(COVER_MAX_W, COVER_MAX_H, image::imageops::FilterType::Lanczos3);
    let (w, h)  = (resized.width(), resized.height());

    let mut out = std::io::Cursor::new(Vec::new());
    resized
        .write_to(&mut out, image::ImageFormat::Jpeg)
        .map_err(|e| ProcessingError::CoverError(e.to_string()))?;

    Ok(CoverResult {
        data:      out.into_inner(),
        mime_type: "image/jpeg".to_string(),
        width:     w,
        height:    h,
    })
}