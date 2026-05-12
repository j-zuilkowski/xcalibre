//! Database layer for the `installed_plugins` table.
//!
//! Each installed plugin gets one row in this table. The row stores everything
//! xcalibre needs to display the plugin in the UI and to load its dylib at runtime:
//!
//! - Identity fields (`id`, `name`, `version`, `api_version`, `plugin_type`)
//! - `dylib_path` — absolute filesystem path to the extracted `.dylib`/`.so`/`.dll`
//! - `enabled` — whether xcalibre should activate this plugin on startup
//! - Audit timestamps (`installed_at`, `updated_at`)
//!
//! ## Schema
//!
//! ```sql
//! CREATE TABLE installed_plugins (
//!     id           TEXT PRIMARY KEY,
//!     name         TEXT NOT NULL,
//!     version      TEXT NOT NULL,
//!     api_version  INTEGER NOT NULL,
//!     plugin_type  TEXT NOT NULL,
//!     dylib_path   TEXT NOT NULL,
//!     enabled      INTEGER NOT NULL DEFAULT 1,
//!     installed_at TEXT NOT NULL,
//!     updated_at   TEXT NOT NULL
//! );
//! ```
//!
//! The `enabled` column is stored as SQLite INTEGER (0/1) but mapped to `bool`
//! via `CAST(enabled AS BOOLEAN)` in [`list_plugins`].
//!
//! ## Tauri command surface
//!
//! All functions here are called exclusively from `src-tauri/src/commands.rs`
//! through thin wrappers:
//!
//! | DB function | Tauri command |
//! |-------------|---------------|
//! | [`install_plugin`] | `install_plugin_from_zip` |
//! | [`list_plugins`] | `list_plugins_cmd` |
//! | [`set_plugin_enabled`] | `set_plugin_enabled_cmd` |
//! | [`uninstall_plugin`] | `uninstall_plugin_cmd` |

use crate::error::ProcessingError;
use sqlx::SqlitePool;

/// A plugin row as read from the database, including all fields needed by the UI.
///
/// Serialisable to JSON so the Tauri command can return it directly to the
/// React frontend without an intermediate DTO.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct InstalledPlugin {
    /// UUID primary key, generated at install time.
    pub id: String,

    /// Human-readable plugin name from `plugin.json`.
    pub name: String,

    /// SemVer string from `plugin.json` (e.g. `"1.2.0"`).
    pub version: String,

    /// The `api_version` value from `plugin.json`. Stored for display and future
    /// re-validation if the SDK version is bumped.
    pub api_version: i64,

    /// One of `"metadata_source"`, `"conversion_output"`, or `"store"`.
    /// Matches [`xcalibre_plugin_sdk::PluginType::as_str`].
    pub plugin_type: String,

    /// Absolute path to the extracted `.dylib`/`.so`/`.dll` file.
    /// Used by the runtime loader to call `dlopen` / `LoadLibrary`.
    pub dylib_path: String,

    /// Whether the plugin is active. Disabled plugins are listed in the UI
    /// but their vtables are never loaded.
    pub enabled: bool,

    /// RFC-3339 timestamp of when the plugin was first installed.
    pub installed_at: String,
}

/// Data required to register a new plugin after its ZIP has been extracted.
///
/// Constructed by `src-tauri/src/commands.rs::install_plugin_from_zip` from the
/// [`xcalibre_plugin_sdk::PluginMetadata`] returned by
/// [`crate::plugins::loader::install_plugin_zip`].
pub struct NewPlugin {
    pub name:        String,
    pub version:     String,
    pub api_version: i64,
    /// The `plugin_type` string value (e.g. `"metadata_source"`).
    pub plugin_type: String,
    /// Absolute path where the dylib was extracted.
    pub dylib_path:  String,
}

/// Register a new plugin in the database.
///
/// Plugins are enabled by default (`enabled = 1`). The UUID `id` is generated
/// here and returned so the caller can display it or use it for further queries.
///
/// This function does **not** verify the dylib or check the API version — that
/// responsibility belongs to [`crate::plugins::loader::install_plugin_zip`], which
/// must be called first.
pub async fn install_plugin(pool: &SqlitePool, p: &NewPlugin) -> Result<String, ProcessingError> {
    let id  = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO installed_plugins
         (id, name, version, api_version, plugin_type, dylib_path, installed_at, updated_at)
         VALUES (?,?,?,?,?,?,?,?)",
    )
    .bind(&id).bind(&p.name).bind(&p.version).bind(p.api_version)
    .bind(&p.plugin_type).bind(&p.dylib_path).bind(&now).bind(&now)
    .execute(pool).await.map_err(ProcessingError::DbError)?;
    Ok(id)
}

/// Return all installed plugins sorted alphabetically by name.
///
/// Both enabled and disabled plugins are included — the caller decides whether
/// to filter. The `CAST(enabled AS BOOLEAN)` ensures the `bool` field in
/// [`InstalledPlugin`] is set correctly regardless of whether SQLite stored the
/// value as an integer or a textual boolean.
pub async fn list_plugins(pool: &SqlitePool) -> Result<Vec<InstalledPlugin>, ProcessingError> {
    sqlx::query_as::<_, InstalledPlugin>(
        "SELECT id, name, version, api_version, plugin_type, dylib_path,
                CAST(enabled AS BOOLEAN) as enabled, installed_at
         FROM installed_plugins ORDER BY name ASC",
    )
    .fetch_all(pool).await.map_err(ProcessingError::DbError)
}

/// Enable or disable a plugin by its UUID.
///
/// Disabling a plugin prevents its vtable from being loaded on subsequent
/// xcalibre launches. The change takes effect the next time the plugin would
/// be invoked — running sessions are not interrupted.
///
/// `updated_at` is bumped so audit trails remain accurate.
pub async fn set_plugin_enabled(pool: &SqlitePool, id: &str, enabled: bool) -> Result<(), ProcessingError> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("UPDATE installed_plugins SET enabled = ?, updated_at = ? WHERE id = ?")
        .bind(enabled as i64).bind(&now).bind(id)
        .execute(pool).await.map_err(ProcessingError::DbError)?;
    Ok(())
}

/// Remove a plugin from the database and return its dylib path.
///
/// The caller is responsible for deleting the dylib file from disk. This
/// two-step design means the database row is removed even if the filesystem
/// delete fails (e.g. the file was already deleted manually), avoiding orphaned
/// database entries.
///
/// Returns the `dylib_path` that was stored, or an empty string if the plugin
/// row was not found (idempotent behaviour for double-uninstall).
pub async fn uninstall_plugin(pool: &SqlitePool, id: &str) -> Result<String, ProcessingError> {
    // Fetch the dylib path before deletion so we can tell the caller what to clean up.
    let row: Option<(String,)> = sqlx::query_as("SELECT dylib_path FROM installed_plugins WHERE id = ?")
        .bind(id).fetch_optional(pool).await.map_err(ProcessingError::DbError)?;
    sqlx::query("DELETE FROM installed_plugins WHERE id = ?")
        .bind(id).execute(pool).await.map_err(ProcessingError::DbError)?;
    Ok(row.map(|(p,)| p).unwrap_or_default())
}
