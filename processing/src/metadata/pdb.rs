use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use std::io::Read;
use std::path::Path;

/// PDB (Palm Database) / PML / RB — 32-byte header with name at offset 0.
/// The first 32 bytes contain a null-terminated book name.
pub fn extract(path: &Path) -> Result<BookMetadata, ProcessingError> {
    let mut file = std::fs::File::open(path).map_err(ProcessingError::IoError)?;
    let mut header = [0u8; 32];
    file.read_exact(&mut header).map_err(ProcessingError::IoError)?;

    let name_bytes = header.split(|&byte| byte == 0).next().unwrap_or(&header);
    let title = std::str::from_utf8(name_bytes)
        .ok()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(String::from);

    Ok(BookMetadata {
        title,
        ..BookMetadata::default()
    })
}
