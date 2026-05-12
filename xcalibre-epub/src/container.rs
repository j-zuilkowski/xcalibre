use crate::EpubError;
use std::path::Path;

pub struct Container {
    _private: (),
}

impl Container {
    pub fn open(_path: &Path) -> Result<Self, EpubError> {
        unimplemented!("rmp05b")
    }
    pub fn manifest_items(&self) -> Vec<(String, String, String)> {
        unimplemented!("rmp05b")
    }
    pub fn read_item(&self, _href: &str) -> Result<Vec<u8>, EpubError> {
        unimplemented!("rmp05b")
    }
    pub fn write_item(&mut self, _href: &str, _data: &[u8], _media_type: &str) -> Result<(), EpubError> {
        unimplemented!("rmp05b")
    }
    pub fn remove_item(&mut self, _href: &str) -> Result<(), EpubError> {
        unimplemented!("rmp05b")
    }
    pub fn save(&mut self) -> Result<(), EpubError> {
        unimplemented!("rmp05b")
    }
    pub fn save_as(&self, _path: &Path) -> Result<(), EpubError> {
        unimplemented!("rmp05b")
    }
    pub fn opf_path(&self) -> &str {
        unimplemented!("rmp05b")
    }
    pub fn spine_hrefs(&self) -> Vec<String> {
        unimplemented!("rmp05b")
    }
}
