use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DetectedFormat {
    Epub,
    Pdf,
    Mobi,
    Azw3,
    Azw4,
    Cbz,
    Cbr,
    Txt,
    Fb2,
    Html,
    Htmlz,
    Rtf,
    Docx,
    Odt,
    Chm,
    Lrf,
    Lrx,
    Pdb,
    Pml,
    Rb,
    Snb,
    Tcr,
    Djvu,
    Lit,
}

impl DetectedFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Epub => ".epub",
            Self::Pdf => ".pdf",
            Self::Mobi | Self::Azw3 => ".mobi",
            Self::Azw4 => ".azw4",
            Self::Cbz => ".cbz",
            Self::Cbr => ".cbr",
            Self::Txt => ".txt",
            Self::Fb2 => ".fb2",
            Self::Html => ".html",
            Self::Htmlz => ".htmlz",
            Self::Rtf => ".rtf",
            Self::Docx => ".docx",
            Self::Odt => ".odt",
            Self::Chm => ".chm",
            Self::Lrf => ".lrf",
            Self::Lrx => ".lrx",
            Self::Pdb => ".pdb",
            Self::Pml => ".pml",
            Self::Rb => ".rb",
            Self::Snb => ".snb",
            Self::Tcr => ".tcr",
            Self::Djvu => ".djvu",
            Self::Lit => ".lit",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Epub => "EPUB",
            Self::Pdf => "PDF",
            Self::Mobi => "MOBI",
            Self::Azw3 => "AZW3",
            Self::Azw4 => "AZW4",
            Self::Cbz => "CBZ",
            Self::Cbr => "CBR",
            Self::Txt => "TXT",
            Self::Fb2 => "FB2",
            Self::Html => "HTML",
            Self::Htmlz => "HTMLZ",
            Self::Rtf => "RTF",
            Self::Docx => "DOCX",
            Self::Odt => "ODT",
            Self::Chm => "CHM",
            Self::Lrf => "LRF",
            Self::Lrx => "LRX",
            Self::Pdb => "PDB",
            Self::Pml => "PML",
            Self::Rb => "RB",
            Self::Snb => "SNB",
            Self::Tcr => "TCR",
            Self::Djvu => "DJVU",
            Self::Lit => "LIT",
        }
    }

    pub fn is_image_only(&self) -> bool {
        matches!(self, Self::Cbz | Self::Cbr)
    }
}

impl fmt::Display for DetectedFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}
