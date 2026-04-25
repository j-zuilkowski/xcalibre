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

#[derive(Debug, Clone)]
pub struct ExtractedText {
    pub full_text:  String,
    pub word_count: usize,
}
