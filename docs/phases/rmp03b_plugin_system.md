# RMP-03b — Plugin System (Green: Implementation)

> HOW TO USE: Write every file exactly as shown, run every shell block.
> DO NOT print code as output — write to disk using your tools.
> Prerequisite: rmp03a complete (failing tests committed).
> TDD role: GREEN — implement until all tests pass.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R03b-T01 | `xcalibre-plugin-sdk` crate: types + PLUGIN_API_VERSION | ⬜ |
| R03b-T02 | `db/plugin_queries.rs` — CRUD | ⬜ |
| R03b-T03 | `plugins/loader.rs` — ZIP install + version guard | ⬜ |
| R03b-T04 | Tauri commands: plugin management | ⬜ |
| R03b-T05 | PluginManagerModal.tsx + tests | ⬜ |
| R03b-T06 | Milestone check + visual inspection | ⬜ |

---

## R03b-T01

Add `"xcalibre-plugin-sdk"` to workspace `members` in root `Cargo.toml`.

Create directory `xcalibre-plugin-sdk/src/`.

Write `xcalibre-plugin-sdk/Cargo.toml`:
```toml
[package]
name = "xcalibre-plugin-sdk"
version = "1.0.0"
edition = "2021"

[dependencies]
serde = { version = "1", features = ["derive"] }
```

Write `xcalibre-plugin-sdk/src/lib.rs`:
```rust
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
```

Add `xcalibre-plugin-sdk` as a dependency in `processing/Cargo.toml`:
```toml
xcalibre-plugin-sdk = { path = "../xcalibre-plugin-sdk" }
```

Then run:
```bash
cargo build --workspace
cargo test --workspace -- test_plugin_sdk
git add xcalibre-plugin-sdk/ processing/Cargo.toml Cargo.toml
git commit -m "R03b-T01: xcalibre-plugin-sdk crate — SDK tests green"
```

---

## R03b-T02

In `processing/src/db/mod.rs`, add:
```rust
pub mod plugin_queries;
```

Write `processing/src/db/plugin_queries.rs`:
```rust
use crate::error::ProcessingError;
use sqlx::SqlitePool;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct InstalledPlugin {
    pub id:           String,
    pub name:         String,
    pub version:      String,
    pub api_version:  i64,
    pub plugin_type:  String,
    pub dylib_path:   String,
    pub enabled:      bool,
    pub installed_at: String,
}

pub struct NewPlugin {
    pub name:        String,
    pub version:     String,
    pub api_version: i64,
    pub plugin_type: String,
    pub dylib_path:  String,
}

pub async fn install_plugin(pool: &SqlitePool, p: &NewPlugin) -> Result<String, ProcessingError> {
    let id = uuid::Uuid::new_v4().to_string();
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

pub async fn list_plugins(pool: &SqlitePool) -> Result<Vec<InstalledPlugin>, ProcessingError> {
    sqlx::query_as::<_, InstalledPlugin>(
        "SELECT id, name, version, api_version, plugin_type, dylib_path,
                CAST(enabled AS BOOLEAN) as enabled, installed_at
         FROM installed_plugins ORDER BY name ASC",
    )
    .fetch_all(pool).await.map_err(ProcessingError::DbError)
}

pub async fn set_plugin_enabled(pool: &SqlitePool, id: &str, enabled: bool) -> Result<(), ProcessingError> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("UPDATE installed_plugins SET enabled = ?, updated_at = ? WHERE id = ?")
        .bind(enabled as i64).bind(&now).bind(id)
        .execute(pool).await.map_err(ProcessingError::DbError)?;
    Ok(())
}

pub async fn uninstall_plugin(pool: &SqlitePool, id: &str) -> Result<String, ProcessingError> {
    let row: Option<(String,)> = sqlx::query_as("SELECT dylib_path FROM installed_plugins WHERE id = ?")
        .bind(id).fetch_optional(pool).await.map_err(ProcessingError::DbError)?;
    sqlx::query("DELETE FROM installed_plugins WHERE id = ?")
        .bind(id).execute(pool).await.map_err(ProcessingError::DbError)?;
    Ok(row.map(|(p,)| p).unwrap_or_default())
}
```

Then run:
```bash
cargo test --workspace -- test_plugin_db
git add processing/src/db/plugin_queries.rs processing/src/db/mod.rs
git commit -m "R03b-T02: plugin DB queries — plugin DB tests green"
```

---

## R03b-T03

In `processing/src/plugins/mod.rs`, add `pub mod loader;`.

Write `processing/src/plugins/loader.rs`:
```rust
use std::io::Read;
use std::path::Path;
use xcalibre_plugin_sdk::{PluginMetadata, PLUGIN_API_VERSION};
use crate::error::ProcessingError;

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
```

Then run:
```bash
cargo test --workspace -- test_plugin_loader
cargo build --workspace
cargo clippy --workspace -- -D warnings
git add processing/src/plugins/loader.rs processing/src/plugins/mod.rs
git commit -m "R03b-T03: plugin ZIP loader with ABI version guard — loader tests green"
```

---

## R03b-T04

In `src-tauri/src/commands.rs`, append:
```rust
use xcalibre_processing::db::plugin_queries::{
    install_plugin, list_plugins, set_plugin_enabled, uninstall_plugin,
    InstalledPlugin, NewPlugin,
};

#[tauri::command]
pub async fn list_plugins_cmd(
    pool: tauri::State<'_, std::sync::Arc<sqlx::SqlitePool>>,
) -> Result<Vec<InstalledPlugin>, String> {
    list_plugins(pool.inner().as_ref()).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn install_plugin_from_zip(
    pool: tauri::State<'_, std::sync::Arc<sqlx::SqlitePool>>,
    app: tauri::AppHandle,
    zip_path: String,
) -> Result<String, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let plugins_dir = data_dir.join("plugins");
    let zip = std::path::Path::new(&zip_path);
    let meta = xcalibre_processing::plugins::loader::install_plugin_zip(zip, &plugins_dir)
        .map_err(|e| e.to_string())?;
    let dylib_path = plugins_dir
        .join(&meta.name)
        .to_string_lossy().into_owned();
    let p = NewPlugin {
        name: meta.name.clone(), version: meta.version.clone(),
        api_version: meta.api_version as i64,
        plugin_type: meta.plugin_type.as_str().to_string(),
        dylib_path,
    };
    install_plugin(pool.inner().as_ref(), &p).await.map_err(|e| e.to_string())?;
    Ok(meta.name)
}

#[tauri::command]
pub async fn set_plugin_enabled_cmd(
    pool: tauri::State<'_, std::sync::Arc<sqlx::SqlitePool>>,
    id: String, enabled: bool,
) -> Result<(), String> {
    set_plugin_enabled(pool.inner().as_ref(), &id, enabled).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn uninstall_plugin_cmd(
    pool: tauri::State<'_, std::sync::Arc<sqlx::SqlitePool>>,
    id: String,
) -> Result<(), String> {
    let dylib_path = uninstall_plugin(pool.inner().as_ref(), &id).await.map_err(|e| e.to_string())?;
    let _ = std::fs::remove_file(&dylib_path); // best-effort cleanup
    Ok(())
}
```

Register all four in `generate_handler!`. Then run:
```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
git add src-tauri/src/commands.rs src-tauri/src/main.rs
git commit -m "R03b-T04: plugin management Tauri commands"
```

---

## R03b-T05

Write `ui/src/components/PluginManagerModal.tsx`:
```tsx
import { useEffect, useState } from "react"
import { invoke, open as openDialog } from "@tauri-apps/api/core"

interface Plugin {
  id: string; name: string; version: string
  plugin_type: string; enabled: boolean
}

interface Props { onClose: () => void }

export function PluginManagerModal({ onClose }: Props) {
  const [plugins, setPlugins] = useState<Plugin[]>([])
  const [error, setError] = useState<string | null>(null)

  const reload = () =>
    invoke<Plugin[]>("list_plugins_cmd").then(setPlugins).catch(e => setError(String(e)))

  useEffect(() => { reload() }, [])

  const toggle = async (id: string, enabled: boolean) => {
    await invoke("set_plugin_enabled_cmd", { id, enabled: !enabled })
    reload()
  }

  const uninstall = async (id: string) => {
    await invoke("uninstall_plugin_cmd", { id })
    reload()
  }

  const installZip = async () => {
    const path = await openDialog({ filters: [{ name: "Plugin ZIP", extensions: ["zip"] }] })
    if (!path) return
    try {
      await invoke("install_plugin_from_zip", { zipPath: path })
      reload()
    } catch (e) { setError(String(e)) }
  }

  return (
    <div data-testid="plugin-manager" className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-white dark:bg-gray-800 rounded-xl shadow-2xl w-full max-w-lg p-6">
        <div className="flex justify-between items-center mb-4">
          <h2 className="text-lg font-semibold dark:text-white">Plugins</h2>
          <button data-testid="plugin-manager-close" onClick={onClose} className="text-gray-400 hover:text-gray-600 text-xl">×</button>
        </div>
        {error && <p className="text-red-500 text-sm mb-3">{error}</p>}
        {plugins.length === 0 && <p className="text-gray-400 text-sm mb-4">No plugins installed.</p>}
        <ul className="space-y-2 mb-4">
          {plugins.map(p => (
            <li key={p.id} data-testid={`plugin-row-${p.id}`}
                className="flex items-center justify-between px-3 py-2 rounded border dark:border-gray-700">
              <div>
                <span className="font-medium text-sm dark:text-white">{p.name}</span>
                <span className="ml-2 text-xs text-gray-400">{p.version} · {p.plugin_type}</span>
              </div>
              <div className="flex gap-2">
                <button onClick={() => toggle(p.id, p.enabled)}
                  className={`text-xs px-2 py-1 rounded ${p.enabled ? "bg-green-100 text-green-700" : "bg-gray-100 text-gray-500"}`}>
                  {p.enabled ? "Enabled" : "Disabled"}
                </button>
                <button onClick={() => uninstall(p.id)}
                  className="text-xs px-2 py-1 rounded bg-red-100 text-red-600 hover:bg-red-200">
                  Remove
                </button>
              </div>
            </li>
          ))}
        </ul>
        <button data-testid="install-plugin-btn" onClick={installZip}
          className="w-full text-sm border-2 border-dashed border-gray-300 dark:border-gray-600
                     rounded-lg py-2 text-gray-500 hover:border-blue-400 hover:text-blue-500">
          + Install plugin from ZIP…
        </button>
      </div>
    </div>
  )
}
```

Write `ui/src/components/PluginManagerModal.test.tsx`:
```tsx
import { render, screen, fireEvent, waitFor } from "@testing-library/react"
import { describe, it, expect, vi, beforeEach } from "vitest"
import { PluginManagerModal } from "./PluginManagerModal"
import { mockInvoke } from "../test/setup"

const mockPlugins = [
  { id: "p1", name: "open-library", version: "1.0.0", plugin_type: "metadata_source", enabled: true },
]

beforeEach(() => {
  mockInvoke("list_plugins_cmd", mockPlugins)
  mockInvoke("set_plugin_enabled_cmd", undefined)
  mockInvoke("uninstall_plugin_cmd", undefined)
})

describe("PluginManagerModal", () => {
  it("renders installed plugins", async () => {
    render(<PluginManagerModal onClose={vi.fn()} />)
    await waitFor(() => screen.getByTestId("plugin-row-p1"))
    expect(screen.getByTestId("plugin-row-p1")).toHaveTextContent("open-library")
  })

  it("shows empty state when no plugins", async () => {
    mockInvoke("list_plugins_cmd", [])
    render(<PluginManagerModal onClose={vi.fn()} />)
    await waitFor(() => screen.getByText(/No plugins installed/))
  })

  it("calls onClose when × is clicked", async () => {
    const onClose = vi.fn()
    render(<PluginManagerModal onClose={onClose} />)
    fireEvent.click(screen.getByTestId("plugin-manager-close"))
    expect(onClose).toHaveBeenCalled()
  })
})
```

Then run:
```bash
cd ui && npm test -- --reporter=verbose 2>&1 | tail -20 && cd ..
cargo build --workspace
git add ui/src/components/PluginManagerModal.tsx ui/src/components/PluginManagerModal.test.tsx
git commit -m "R03b-T05: PluginManagerModal component + tests"
```

---

## R03b-T06 — Milestone Check + Visual Inspection

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cd ui && npm run build && npm test && cd ..
```

**Visual inspection:**
```bash
pkill -x xcalibre 2>/dev/null || true
cargo tauri dev &>/tmp/xcalibre_tauri_dev.log &
# Poll until xcalibre process appears — first-run compilation can take 3-5 min
for i in $(seq 1 30); do
  sleep 10
  if pgrep -x xcalibre > /dev/null 2>&1; then
    echo "xcalibre running after $((i*10))s"
    sleep 3
    break
  fi
  echo "Waiting for xcalibre… $((i*10))s elapsed"
  [ "$i" -eq 30 ] && echo "ERROR: xcalibre did not launch within 5 minutes" && exit 1
done
```

Verify:
- [ ] Plugin manager accessible from menu (add a menu item to App.tsx if not already wired)
- [ ] Empty state shows "No plugins installed"
- [ ] "Install plugin from ZIP…" button opens file picker
- [ ] Installing a ZIP with wrong API version shows an error toast (not a crash)
- [ ] Enable/Disable toggle visually flips

```bash
pkill -x xcalibre 2>/dev/null || true
```

```bash
git add -A
git commit -m "R03b-T06: RMP-03 plugin system — all tests green, visual verified"
```
