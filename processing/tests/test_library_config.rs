//! Tests for LibraryConfig persistence (RMP-01).
//! Tests for LibraryConfig persistence (RMP-01).

use std::path::PathBuf;
use xcalibre_processing::config::{LibraryConfig, LibraryEntry};

#[test]
fn test_library_config_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("config.json");

    let mut cfg = LibraryConfig::new(config_path.clone());
    cfg.add_library(LibraryEntry {
        id: "lib-1".into(),
        name: "Main Library".into(),
        db_path: PathBuf::from("/tmp/main.db"),
        cover_dir: PathBuf::from("/tmp/covers"),
        layout: "in_place".into(),
        xs_url: None,
    });
    cfg.set_active("lib-1");
    cfg.save().expect("save");

    let loaded = LibraryConfig::load(config_path).expect("load");
    assert_eq!(loaded.libraries().len(), 1);
    assert_eq!(loaded.active_id().unwrap(), "lib-1");
    assert_eq!(loaded.libraries()[0].name, "Main Library");
}

#[test]
fn test_library_config_active_switches() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("config.json");

    let mut cfg = LibraryConfig::new(config_path.clone());
    cfg.add_library(LibraryEntry {
        id: "a".into(), name: "A".into(),
        db_path: PathBuf::from("/tmp/a.db"),
        cover_dir: PathBuf::from("/tmp/ac"),
        layout: "in_place".into(), xs_url: None,
    });
    cfg.add_library(LibraryEntry {
        id: "b".into(), name: "B".into(),
        db_path: PathBuf::from("/tmp/b.db"),
        cover_dir: PathBuf::from("/tmp/bc"),
        layout: "managed".into(), xs_url: None,
    });
    cfg.set_active("b");
    cfg.save().unwrap();

    let loaded = LibraryConfig::load(config_path).unwrap();
    assert_eq!(loaded.active_id().unwrap(), "b");
    assert_eq!(loaded.active_library().unwrap().layout, "managed");
}
