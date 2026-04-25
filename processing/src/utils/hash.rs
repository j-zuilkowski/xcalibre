use crate::error::ProcessingError;
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::Path;

pub fn sha256_file(path: &Path) -> Result<String, ProcessingError> {
    let mut file = std::fs::File::open(path).map_err(ProcessingError::IoError)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = file.read(&mut buf).map_err(ProcessingError::IoError)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}