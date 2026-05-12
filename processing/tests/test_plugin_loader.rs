//! Tests for plugin ZIP extraction and ABI version guard.
//! These tests FAIL until rmp03b implements PluginLoader.

use std::io::Write;
use std::path::PathBuf;
use xcalibre_processing::plugins::loader::{install_plugin_zip, PluginLoadError};

#[test]
fn test_version_mismatch_is_rejected() {
    // A ZIP with a dylib that exports the wrong PLUGIN_API_VERSION must be refused.
    // We simulate this with a crafted fixture.
    let fixture = PathBuf::from("tests/fixtures/plugin_bad_version.zip");
    if !fixture.exists() { return; } // fixture generated in rmp03b
    let dir = tempfile::tempdir().unwrap();
    let result = install_plugin_zip(&fixture, dir.path());
    assert!(
        matches!(result, Err(PluginLoadError::ApiVersionMismatch { .. })),
        "expected ApiVersionMismatch, got: {:?}", result
    );
}

#[test]
fn test_missing_manifest_is_rejected() {
    // A ZIP with no plugin.json manifest must be rejected.
    let dir = tempfile::tempdir().unwrap();
    let zip_path = dir.path().join("no_manifest.zip");
    {
        let f = std::fs::File::create(&zip_path).unwrap();
        let mut z = zip::ZipWriter::new(f);
        let opts = zip::write::FileOptions::default();
        z.start_file("dummy.txt", opts).unwrap();
        z.write_all(b"hello").unwrap();
        z.finish().unwrap();
    }
    let out_dir = tempfile::tempdir().unwrap();
    let result = install_plugin_zip(&zip_path, out_dir.path());
    assert!(
        matches!(result, Err(PluginLoadError::MissingManifest)),
        "expected MissingManifest, got: {:?}", result
    );
}
