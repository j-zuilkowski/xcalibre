//! Plugin SDK for xCalibre (S1-A: extern "C" vtables + semver guard).

/// Increment this when any vtable interface changes.
/// Plugins must be compiled against the matching SDK version.
pub const PLUGIN_API_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PluginType {
    MetadataSource,
    ConversionOutput,
    Store,
}

impl PluginType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PluginType::MetadataSource  => "metadata_source",
            PluginType::ConversionOutput => "conversion_output",
            PluginType::Store            => "store",
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginMetadata {
    pub name:        String,
    pub version:     String,
    pub api_version: u32,
    pub plugin_type: PluginType,
}

/// Every plugin dylib must export this symbol returning PLUGIN_API_VERSION.
/// Signature: `extern "C" fn xcalibre_plugin_api_version() -> u32`
pub const API_VERSION_SYMBOL: &str = "xcalibre_plugin_api_version";

/// Every plugin dylib must export this symbol returning a pointer to
/// a null-terminated JSON string containing PluginMetadata.
pub const METADATA_SYMBOL: &str = "xcalibre_plugin_metadata";

/// Vtable for MetadataSource plugins.
/// Fields are C function pointers for ABI stability.
#[repr(C)]
pub struct MetadataSourceVtable {
    /// Search for books by query string. Returns JSON array of BookMetadata.
    pub search: extern "C" fn(query: *const std::os::raw::c_char) -> *mut std::os::raw::c_char,
    /// Free a string returned by `search`.
    pub free_str: extern "C" fn(ptr: *mut std::os::raw::c_char),
}

/// Vtable for ConversionOutput plugins.
#[repr(C)]
pub struct ConversionOutputVtable {
    /// Convert EPUB bytes to target format. Returns output bytes length.
    pub convert: extern "C" fn(
        epub_data: *const u8, epub_len: usize,
        out_buf: *mut *mut u8, out_len: *mut usize,
    ) -> i32,
    pub free_buf: extern "C" fn(ptr: *mut u8, len: usize),
}
