//! Plugin subsystem: format detection types and plugin loader.
//!
//! This module owns two distinct concerns that share a home here because format
//! detection is the foundation on which plugin-based conversion is built:
//!
//! - **[`DetectedFormat`]** — the canonical enum of every ebook format xcalibre
//!   understands. Format detection happens via magic-byte inspection (not file
//!   extension), implemented in [`crate::pipeline::ingest::detect_format`].
//!
//! - **[`loader`]** — plugin ZIP extraction and ABI version verification.
//!   See [`loader::install_plugin_zip`] for the full installation flow.
//!
//! ## Format detection precedence
//!
//! `detect_format` inspects the first 4096 bytes of a file in this priority order:
//!
//! 1. **ZIP-based formats** — `PK\x03\x04` magic → probe inner entries to
//!    distinguish EPUB, DOCX, ODT, HTMLZ, and CBZ.
//! 2. **Specific binary magic** — `%PDF-`, `BOOKMOBI`, `{\\rtf`, `AT&TFORM`
//!    (DjVu), `ITSF` (CHM), `ITOLITLS` (LIT), `SQLite format 3\0` (KFX probe),
//!    `SNBP`, `L\0R\0F\0` (LRF), RAR signatures.
//! 3. **XML/text probes** — FictionBook XML (`FictionBook` in first 4096 bytes),
//!    HTML (`<html`, `<!DOCTYPE`).
//! 4. **Extension fallback** — for formats with no reliable magic (PDB, TCR, RB,
//!    PML, LRX, AZW3/AZW4 disambiguation).
//! 5. **Printable-character heuristic** — ≥50 % printable bytes → TXT.
//! 6. **Error** — `ProcessingError::UnsupportedFormat`.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Every ebook and document format that xcalibre can ingest, display, or convert.
///
/// Variants are detected by [`crate::pipeline::ingest::detect_format`] from file
/// content (not filename). The same enum is stored as its [`DetectedFormat::to_string`]
/// representation in `jobs.format` and `local_books.format`.
///
/// ## Adding a new format
///
/// 1. Add a variant here with a doc comment explaining its magic bytes / detection
///    method.
/// 2. Add `extension()` and `display_name()` arms.
/// 3. Add magic-byte detection logic in `pipeline/ingest.rs::detect_format`.
/// 4. Add integrity validation in `pipeline/ingest.rs::validate_integrity`
///    (or fall through to the no-op arm if there is no useful check).
/// 5. Add conversion support in the appropriate `convert/` file and wire it into
///    `src-tauri/src/commands.rs::convert_book`.
/// 6. Add the format to migration `0020_jobs_format_any.sql` is already permissive,
///    so no migration is needed — just add the UI option in `ConversionDialog.tsx`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DetectedFormat {
    /// Open Container Format EPUB 2/3. Detected by `PK` magic + `mimetype` entry
    /// starting with `application/epub`.
    Epub,
    /// PDF. Detected by `%PDF-` header. Also validates `%%EOF` marker at tail.
    Pdf,
    /// Mobipocket. Detected by `BOOKMOBI` at bytes 60–68.
    Mobi,
    /// Kindle Format 8 (AZW3). Same `BOOKMOBI` magic as MOBI; disambiguated by
    /// `.azw3` extension fallback.
    Azw3,
    /// Kindle Print Replica (AZW4). Detected by extension after `BOOKMOBI` magic.
    Azw4,
    /// Comic Book ZIP. Any ZIP that is not EPUB/DOCX/ODT/HTMLZ falls through to CBZ.
    Cbz,
    /// Comic Book RAR. Detected by RAR v4 (`Rar!\x1a\x07\x00`) or RAR v5
    /// (`Rar!\x1a\x07\x01\x00`) magic.
    Cbr,
    /// Plain text. Detected by printable-character heuristic (≥50 % printable,
    /// no null bytes) after all other probes fail.
    Txt,
    /// FictionBook 2 XML. Detected by `<?xml` or `<FictionBook` at file start,
    /// confirmed by `FictionBook` string within first 4096 bytes.
    Fb2,
    /// HTML document. Detected by `<html`, `<HTML`, `<!DOCTYPE`, or `<!doctype`
    /// at the start of the file content.
    Html,
    /// HTML ZIP (Calibre's HTMLZ format). A ZIP containing `index.html` at the root.
    Htmlz,
    /// Rich Text Format. Detected by `{\rtf` magic at file start.
    Rtf,
    /// Office Open XML Word document. ZIP + `word/document.xml` + `docProps/core.xml`.
    Docx,
    /// OpenDocument Text. ZIP + `content.xml` + `meta.xml`, or ODF mimetype entry.
    Odt,
    /// Microsoft Compiled HTML Help. Detected by `ITSF` magic.
    Chm,
    /// Sony BBeB Book Format (LRF). Detected by `L\0R\0F\0` magic (UTF-16LE "LRF").
    Lrf,
    /// Sony LRX (LRF encrypted). No reliable magic; detected by `.lrx` extension.
    Lrx,
    /// Palm Database (PalmDOC / various). No reliable magic; detected by `.pdb` extension.
    Pdb,
    /// Palm Markup Language. No reliable magic; detected by `.pml` extension.
    Pml,
    /// Rocket Book / RocketEdition. No reliable magic; detected by `.rb` extension.
    Rb,
    /// Shanda Bambook SNB. Detected by `SNBP` magic at file start.
    Snb,
    /// Compressed text format for Psion/EPOC devices. Detected by `.tcr` extension.
    Tcr,
    /// DjVu document. Detected by `AT&TFORM` magic at file start.
    Djvu,
    /// Microsoft LIT (Reader). Detected by `ITOLITLS` magic at file start.
    Lit,
    /// Kindle Format X (Amazon's SQLite-based format). Detected by SQLite magic +
    /// presence of a `fragments` table (see `is_kfx_database`).
    Kfx,
}

impl DetectedFormat {
    /// Returns the canonical file extension including the leading dot.
    ///
    /// Used when constructing output file paths during conversion. Note that MOBI
    /// and AZW3 share the `.mobi` extension because most readers accept both under
    /// that name.
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
            Self::Kfx => ".kfx",
        }
    }

    /// Returns the user-facing uppercase format name shown in the UI.
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
            Self::Kfx => "KFX",
        }
    }

    /// Returns `true` for image-only container formats (CBZ/CBR) that contain no
    /// extractable text and therefore skip the FTS indexing step.
    pub fn is_image_only(&self) -> bool {
        matches!(self, Self::Cbz | Self::Cbr)
    }
}

impl fmt::Display for DetectedFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

pub mod loader;

// Re-export detect_format here so callers can write `plugins::detect_format`
// instead of `pipeline::ingest::detect_format`.
pub use crate::pipeline::ingest::detect_format;
