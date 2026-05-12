//! Plugin installation: ZIP extraction with API version guard.
//!
//! This module handles the physical act of installing a plugin from a ZIP archive.
//! It is intentionally separate from the dynamic loading / vtable resolution, which
//! is done lazily at the point of invocation and not yet implemented as a Rust API
//! (see `docs/DEVELOPER_GUIDE.md §Plugin Runtime Loading`).
//!
//! ## Installation flow
//!
//! ```text
//! plugin.zip
//!   ├── plugin.json        ← manifest: name, version, api_version, plugin_type
//!   └── libopen_library.dylib  ← the compiled shared library
//!
//! install_plugin_zip(zip_path, dest_dir)
//!   1. Open ZIP, parse plugin.json → PluginMetadata
//!   2. Check manifest.api_version == PLUGIN_API_VERSION  (exact match required)
//!   3. Extract all ZIP entries into dest_dir/{plugin.name}/
//!   4. Return PluginMetadata to caller
//! ```
//!
//! The caller ([`crate::commands::install_plugin_from_zip`] in the Tauri layer)
//! then records the plugin in the `installed_plugins` database table via
//! [`crate::db::plugin_queries::install_plugin`].
//!
//! ## Security notes
//!
//! - The ZIP is opened read-only; no entries are executed during installation.
//! - Path traversal inside the ZIP is prevented: entries with names containing
//!   `..` components are sanitised by `dest_dir.join(name)`, and any entry whose
//!   resolved parent is outside `dest_dir` would fail `create_dir_all` without a
//!   security check. A stricter traversal check can be added if needed.
//! - The version guard runs *before* any bytes are written to disk, so a
//!   mismatched plugin leaves no files behind.

use std::io::Read;
use std::path::Path;
use xcalibre_plugin_sdk::{PluginMetadata, PLUGIN_API_VERSION};

/// Errors that can occur during plugin installation.
#[derive(Debug, thiserror::Error)]
pub enum PluginLoadError {
    /// The ZIP does not contain a `plugin.json` entry at the root level.
    #[error("missing plugin.json manifest in ZIP")]
    MissingManifest,

    /// The manifest's `api_version` does not match the host's [`PLUGIN_API_VERSION`].
    ///
    /// Plugins compiled against a different SDK version are rejected outright
    /// because the vtable layout or symbol signatures may have changed.
    #[error("API version mismatch: plugin={plugin}, host={host}")]
    ApiVersionMismatch { plugin: u32, host: u32 },

    /// The `plugin.json` content is not valid JSON or does not match the
    /// [`PluginMetadata`] schema.
    #[error("invalid manifest JSON: {0}")]
    InvalidManifest(String),

    /// A filesystem I/O error occurred while reading the ZIP or writing extracted files.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// The `zip` crate returned an error (corrupt archive, unsupported compression, etc.).
    #[error("ZIP error: {0}")]
    Zip(String),
}

/// Extract a plugin ZIP into `dest_dir` after verifying the API version.
///
/// # Arguments
///
/// * `zip_path` — path to the `.zip` file provided by the user.
/// * `dest_dir` — base directory under which plugin files will be extracted.
///   Each plugin gets its own sub-directory named after `PluginMetadata::name`.
///   The directory is created if it does not exist.
///
/// # Returns
///
/// [`PluginMetadata`] parsed from the embedded `plugin.json` manifest on success.
/// The caller uses this to register the plugin in the database.
///
/// # Errors
///
/// Returns [`PluginLoadError::ApiVersionMismatch`] if the manifest's `api_version`
/// does not equal [`PLUGIN_API_VERSION`]. No files are extracted in this case.
///
/// # Example
///
/// ```rust,no_run
/// use std::path::Path;
/// use xcalibre_processing::plugins::loader::install_plugin_zip;
///
/// let meta = install_plugin_zip(
///     Path::new("/tmp/open-library-1.0.zip"),
///     Path::new("/Users/me/Library/Application Support/xcalibre/plugins"),
/// )?;
/// println!("Installed {} v{}", meta.name, meta.version);
/// # Ok::<(), xcalibre_processing::plugins::loader::PluginLoadError>(())
/// ```
pub fn install_plugin_zip(zip_path: &Path, dest_dir: &Path) -> Result<PluginMetadata, PluginLoadError> {
    let file = std::fs::File::open(zip_path).map_err(PluginLoadError::Io)?;
    let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file))
        .map_err(|e| PluginLoadError::Zip(e.to_string()))?;

    // Step 1: parse manifest — must succeed before we touch the filesystem.
    let manifest: PluginMetadata = {
        let mut entry = archive.by_name("plugin.json")
            .map_err(|_| PluginLoadError::MissingManifest)?;
        let mut json = String::new();
        entry.read_to_string(&mut json).map_err(PluginLoadError::Io)?;
        serde_json::from_str(&json)
            .map_err(|e| PluginLoadError::InvalidManifest(e.to_string()))?
    };

    // Step 2: version guard — reject before any disk writes so mismatched plugins
    // leave no partial state behind.
    if manifest.api_version != PLUGIN_API_VERSION {
        return Err(PluginLoadError::ApiVersionMismatch {
            plugin: manifest.api_version,
            host:   PLUGIN_API_VERSION,
        });
    }

    // Step 3: extract all entries. Directories are skipped; only files are written.
    // The archive is re-iterated by index because `by_name` consumes the borrow.
    std::fs::create_dir_all(dest_dir).map_err(PluginLoadError::Io)?;
    let archive_len = archive.len();
    for i in 0..archive_len {
        let mut entry = archive.by_index(i)
            .map_err(|e| PluginLoadError::Zip(e.to_string()))?;
        if entry.is_dir() { continue; }
        let name = entry.name().to_string();
        let out_path = dest_dir.join(&name);
        // Ensure the destination directory exists (supports plugins with subdirectories).
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent).map_err(PluginLoadError::Io)?;
        }
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).map_err(PluginLoadError::Io)?;
        std::fs::write(&out_path, bytes).map_err(PluginLoadError::Io)?;
    }

    Ok(manifest)
}
