pub mod epub;
pub mod pdf;
pub mod resize;
pub mod cbz;

#[derive(Debug, Clone)]
pub struct CoverResult {
    pub data:      Vec<u8>,
    pub mime_type: String,
    pub width:     u32,
    pub height:    u32,
}
