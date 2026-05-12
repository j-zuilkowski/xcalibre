use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BookMetadata {
    pub title:        Option<String>,
    pub authors:      Vec<String>,
    pub language:     Option<String>,
    pub publisher:    Option<String>,
    pub published:    Option<String>,
    pub description:  Option<String>,
    pub isbn:         Option<String>,
    pub series:       Option<String>,
    pub series_index: Option<f32>,
    pub tags:         Vec<String>,
}

pub mod epub;
pub mod pdf;
pub mod mobi;
pub mod fb2;
pub mod html;
pub mod rtf;
pub mod docx;
pub mod odt;
pub mod chm;
pub mod lrf;
pub mod pdb;
pub mod snb;
pub mod tcr;
pub mod azw4;
pub mod djvu;
pub mod lit;
pub mod cbz;
pub mod isbn;
pub mod enrichment;
pub mod kfx;
