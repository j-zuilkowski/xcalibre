// stub — implemented in rmp05b
use crate::EpubError;
pub struct FontInfo { pub href: String, pub media_type: String }
pub fn list_fonts(_container: &crate::Container) -> Vec<FontInfo> {
    unimplemented!("rmp05b")
}
pub fn embed_font(_container: &mut crate::Container, _name: &str, _data: &[u8], _media_type: &str)
    -> Result<(), EpubError>
{
    unimplemented!("rmp05b")
}
