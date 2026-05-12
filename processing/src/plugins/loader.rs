use std::io::Read;
use std::path::Path;
use xcalibre_plugin_sdk::{PluginMetadata, PLUGIN_API_VERSION};

#[derive(Debug, thiserror::Error)]
pub enum PluginLoadError {
    #[error("missing plugin.json manifest in ZIP")]
    MissingManifest,
    #[error("API version mismatch: plugin={plugin}, host={host}")]
    ApiVersionMismatch { plugin: u32, host: u32 },
    #[error("invalid manifest JSON: {0}")]
    InvalidManifest(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("ZIP error: {0}")]
    Zip(String),
}

/// Extract a plugin ZIP into `dest_dir` after verifying the API version.
/// Returns the PluginMetadata on success.
pub fn install_plugin_zip(zip_path: &Path, dest_dir: &Path) -> Result<PluginMetadata, PluginLoadError> {
    let file = std::fs::File::open(zip_path).map_err(PluginLoadError::Io)?;
    let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file))
        .map_err(|e| PluginLoadError::Zip(e.to_string()))?;

    // Step 1: read manifest
    let manifest: PluginMetadata = {
        let mut entry = archive.by_name("plugin.json")
            .map_err(|_| PluginLoadError::MissingManifest)?;
        let mut json = String::new();
        entry.read_to_string(&mut json).map_err(PluginLoadError::Io)?;
        serde_json::from_str(&json)
            .map_err(|e| PluginLoadError::InvalidManifest(e.to_string()))?
    };

    // Step 2: version guard
    if manifest.api_version != PLUGIN_API_VERSION {
        return Err(PluginLoadError::ApiVersionMismatch {
            plugin: manifest.api_version,
            host:   PLUGIN_API_VERSION,
        });
    }

    // Step 3: extract all files
    std::fs::create_dir_all(dest_dir).map_err(PluginLoadError::Io)?;
    let archive_len = archive.len();
    for i in 0..archive_len {
        let mut entry = archive.by_index(i)
            .map_err(|e| PluginLoadError::Zip(e.to_string()))?;
        if entry.is_dir() { continue; }
        let name = entry.name().to_string();
        let out_path = dest_dir.join(&name);
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent).map_err(PluginLoadError::Io)?;
        }
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).map_err(PluginLoadError::Io)?;
        std::fs::write(&out_path, bytes).map_err(PluginLoadError::Io)?;
    }

    Ok(manifest)
}
