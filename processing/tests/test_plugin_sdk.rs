//! Tests for xcalibre-plugin-sdk types and PLUGIN_API_VERSION guard.
//! Tests for the plugin SDK crate (RMP-03).

use xcalibre_plugin_sdk::{PLUGIN_API_VERSION, PluginType, PluginMetadata};

#[test]
fn test_api_version_is_nonzero() {
    assert!(PLUGIN_API_VERSION > 0, "API version must be > 0");
}

#[test]
fn test_plugin_metadata_fields() {
    let meta = PluginMetadata {
        name: "test-plugin".into(),
        version: "1.0.0".into(),
        api_version: PLUGIN_API_VERSION,
        plugin_type: PluginType::MetadataSource,
    };
    assert_eq!(meta.name, "test-plugin");
    assert_eq!(meta.api_version, PLUGIN_API_VERSION);
}

#[test]
fn test_plugin_type_display() {
    assert_eq!(PluginType::MetadataSource.as_str(), "metadata_source");
    assert_eq!(PluginType::ConversionOutput.as_str(), "conversion_output");
    assert_eq!(PluginType::Store.as_str(), "store");
}
