// stub — implemented in rmp05b
use crate::EpubError;
pub struct CoverInfo { pub href: String, pub media_type: String, pub data: Vec<u8> }
pub fn extract_cover(_container: &crate::Container) -> Result<Option<CoverInfo>, EpubError> {
    unimplemented!("rmp05b")
}
pub fn replace_cover(_container: &mut crate::Container, _jpeg_data: &[u8]) -> Result<(), EpubError> {
    unimplemented!("rmp05b")
}
