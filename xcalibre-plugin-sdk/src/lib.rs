//! # xcalibre-plugin-sdk
//!
//! The plugin SDK defines the stable ABI contract between xcalibre and third-party
//! plugins distributed as native shared libraries (`.dylib` / `.so` / `.dll`).
//!
//! ## Design rationale
//!
//! Plugins are loaded at runtime via `dlopen`-style dynamic linking. This means:
//!
//! - The plugin and the host may be compiled separately, potentially with different
//!   Rust toolchain versions.
//! - Rust's internal ABI is not stable across compiler versions, so all crossing
//!   points use `extern "C"` (the C calling convention), which *is* stable.
//! - Complex Rust types (enums with fields, `String`, `Vec`) cannot be passed across
//!   the FFI boundary directly. Instead, data is serialised to JSON and passed as
//!   null-terminated C strings (`*const c_char` / `*mut c_char`).
//!
//! ## Versioning
//!
//! [`PLUGIN_API_VERSION`] is the single source of truth for ABI compatibility.
//! Every plugin ZIP must embed a `plugin.json` manifest whose `api_version` field
//! matches this constant. xcalibre refuses to install or activate a plugin whose
//! version does not match exactly — no backwards- or forwards-compatibility is
//! assumed. When any vtable field changes (order, type, or semantics), increment
//! this constant and update this doc comment.
//!
//! ## Plugin types
//!
//! | Type | Vtable | Purpose |
//! |------|--------|---------|
//! | [`PluginType::MetadataSource`] | [`MetadataSourceVtable`] | Search external databases for book metadata |
//! | [`PluginType::ConversionOutput`] | [`ConversionOutputVtable`] | Convert EPUB to a proprietary output format |
//! | [`PluginType::Store`] | *(future)* | Publish books to a reading device or service |
//!
//! ## Writing a plugin
//!
//! See `docs/DEVELOPER_GUIDE.md §Plugin Development` for a step-by-step guide.
//! The minimal requirements are:
//!
//! 1. Export [`API_VERSION_SYMBOL`] returning [`PLUGIN_API_VERSION`].
//! 2. Export [`METADATA_SYMBOL`] returning a pointer to a JSON-encoded [`PluginMetadata`].
//! 3. Export a vtable symbol matching your plugin type (e.g. `xcalibre_metadata_source_vtable`).
//! 4. Pack everything (dylib + `plugin.json` manifest) into a ZIP.

/// The current plugin ABI version.
///
/// Increment this constant whenever the vtable layout, exported symbol signatures,
/// or the JSON schema of any type exchanged across the boundary changes. Plugins
/// compiled against an older or newer version of this SDK will be rejected at
/// install time by [`xcalibre_processing::plugins::loader::install_plugin_zip`].
pub const PLUGIN_API_VERSION: u32 = 1;

/// Discriminant identifying what a plugin does.
///
/// This value is stored in `installed_plugins.plugin_type` as its `as_str()`
/// representation. It determines which vtable symbol xcalibre looks for when
/// activating the plugin.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PluginType {
    /// Queries an external service (e.g. Open Library, Google Books) for book
    /// metadata given a search string or ISBN. xcalibre calls the vtable's
    /// `search` function and merges the returned [`BookMetadata`] JSON array
    /// into the metadata editor.
    MetadataSource,

    /// Converts EPUB bytes to a target format that xcalibre does not support
    /// natively. xcalibre reads the EPUB file, passes its raw bytes to the
    /// vtable's `convert` function, and writes the returned bytes to disk.
    ConversionOutput,

    /// Reserved for future use. Will represent send-to-device and cloud-store
    /// integrations (e.g. Kindle, Kobo, Dropbox).
    Store,
}

impl PluginType {
    /// Returns the lowercase snake_case identifier stored in the database.
    pub fn as_str(&self) -> &'static str {
        match self {
            PluginType::MetadataSource   => "metadata_source",
            PluginType::ConversionOutput => "conversion_output",
            PluginType::Store            => "store",
        }
    }
}

/// Metadata that every plugin must declare in its `plugin.json` manifest.
///
/// xcalibre reads this struct from the manifest embedded inside the plugin ZIP
/// before extracting any files. If the `api_version` does not match
/// [`PLUGIN_API_VERSION`], installation is aborted without touching the filesystem.
///
/// ## plugin.json example
///
/// ```json
/// {
///   "name": "open-library",
///   "version": "1.2.0",
///   "api_version": 1,
///   "plugin_type": "MetadataSource"
/// }
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginMetadata {
    /// Human-readable plugin name, also used as the directory name under
    /// `{app_data}/plugins/` after extraction.
    pub name: String,

    /// SemVer string for the plugin's own version (independent of `api_version`).
    /// Shown in the Plugin Manager UI.
    pub version: String,

    /// Must equal [`PLUGIN_API_VERSION`]. xcalibre enforces an exact match.
    pub api_version: u32,

    /// Determines which vtable xcalibre loads after installation.
    pub plugin_type: PluginType,
}

/// Symbol name that every plugin dylib must export.
///
/// xcalibre resolves this symbol first. The function must have the signature:
///
/// ```c
/// uint32_t xcalibre_plugin_api_version(void);
/// ```
///
/// It must return the value of [`PLUGIN_API_VERSION`] that the plugin was compiled
/// against. xcalibre double-checks this at runtime even though the manifest already
/// carries the version — the dylib export is the authoritative live check.
pub const API_VERSION_SYMBOL: &str = "xcalibre_plugin_api_version";

/// Symbol name that every plugin dylib must export.
///
/// The function must have the signature:
///
/// ```c
/// const char *xcalibre_plugin_metadata(void);
/// ```
///
/// It returns a pointer to a **static**, null-terminated UTF-8 string containing
/// a JSON-serialised [`PluginMetadata`]. xcalibre does *not* call `free` on this
/// pointer; the plugin owns the memory for the lifetime of the process.
pub const METADATA_SYMBOL: &str = "xcalibre_plugin_metadata";

/// Vtable for [`PluginType::MetadataSource`] plugins.
///
/// xcalibre resolves the symbol `xcalibre_metadata_source_vtable` from the dylib
/// and reads this struct directly. Because the struct is `#[repr(C)]`, its memory
/// layout is deterministic across compilers and toolchain versions.
///
/// ## Safety contract
///
/// - `search` receives a null-terminated UTF-8 query string. It must return either
///   a null-terminated UTF-8 JSON array of `BookMetadata` objects, or `null` on
///   error.
/// - The caller (xcalibre) is responsible for calling `free_str` on every non-null
///   pointer returned by `search`. The plugin allocates the string; the plugin frees
///   it. Never pass a pointer allocated by xcalibre to `free_str`.
/// - Both function pointers must be non-null; xcalibre does not check for null
///   function pointers before calling them.
#[repr(C)]
pub struct MetadataSourceVtable {
    /// Search for books matching `query`.
    ///
    /// `query` is a null-terminated UTF-8 string (title keywords, ISBN, etc.).
    /// Returns a pointer to a null-terminated UTF-8 JSON array, e.g.:
    ///
    /// ```json
    /// [{"title":"Dune","authors":["Frank Herbert"],"isbn":"0441013597"}]
    /// ```
    ///
    /// Returns null on failure. The caller must pass the returned pointer to
    /// [`MetadataSourceVtable::free_str`] when done.
    pub search: extern "C" fn(query: *const std::os::raw::c_char) -> *mut std::os::raw::c_char,

    /// Frees a string previously returned by [`MetadataSourceVtable::search`].
    ///
    /// Must be called exactly once per non-null pointer returned by `search`.
    /// Calling with a null pointer is a no-op (matching `free(NULL)` semantics).
    pub free_str: extern "C" fn(ptr: *mut std::os::raw::c_char),
}

/// Vtable for [`PluginType::ConversionOutput`] plugins.
///
/// xcalibre resolves the symbol `xcalibre_conversion_output_vtable` from the dylib.
///
/// ## Safety contract
///
/// - `convert` receives a pointer to EPUB bytes and their length. It must write
///   the converted output into a newly-allocated buffer and set `*out_buf` and
///   `*out_len` accordingly. Returns 0 on success, a negative error code on failure.
/// - xcalibre is responsible for calling `free_buf` on every non-null `*out_buf`
///   after a successful conversion.
/// - Both pointers must remain valid for the duration of the `convert` call.
#[repr(C)]
pub struct ConversionOutputVtable {
    /// Convert `epub_len` bytes of EPUB data at `epub_data` to the plugin's
    /// target format.
    ///
    /// On success: sets `*out_buf` to a newly-allocated buffer containing the
    /// output bytes, sets `*out_len` to the buffer length, and returns 0.
    ///
    /// On failure: returns a negative error code. `*out_buf` and `*out_len` are
    /// undefined and must not be read or freed.
    pub convert: extern "C" fn(
        epub_data: *const u8,
        epub_len:  usize,
        out_buf:   *mut *mut u8,
        out_len:   *mut usize,
    ) -> i32,

    /// Frees a buffer previously returned by [`ConversionOutputVtable::convert`].
    ///
    /// Must be called exactly once per successful conversion. `len` must match the
    /// value written into `*out_len` by `convert`.
    pub free_buf: extern "C" fn(ptr: *mut u8, len: usize),
}
