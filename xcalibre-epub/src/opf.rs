// stub for opf — implemented in rmp05b
use crate::EpubError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EpubOPF {
    pub title:        Option<String>,
    pub authors:      Vec<String>,
    pub language:     Option<String>,
    pub publisher:    Option<String>,
    pub series:       Option<String>,
    pub series_index: Option<f32>,
    pub tags:         Vec<String>,
    pub epub_version: String,
    pub manifest:     Vec<ManifestItem>,
    pub spine:        Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestItem {
    pub id:         String,
    pub href:       String,
    pub media_type: String,
    pub properties: Option<String>,
}

impl EpubOPF {
    pub fn parse(_xml: &str) -> Result<Self, EpubError> {
        unimplemented!("rmp05b")
    }
    pub fn to_xml(&self) -> Result<String, EpubError> {
        unimplemented!("rmp05b")
    }
}
