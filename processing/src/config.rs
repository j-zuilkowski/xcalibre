use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use crate::error::ProcessingError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryEntry {
    pub id:        String,
    pub name:      String,
    pub db_path:   PathBuf,
    pub cover_dir: PathBuf,
    pub layout:    String,
    pub xs_url:    Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Inner {
    active_id:  Option<String>,
    libraries:  Vec<LibraryEntry>,
}

pub struct LibraryConfig {
    path:  PathBuf,
    inner: Inner,
}

impl LibraryConfig {
    pub fn new(path: PathBuf) -> Self {
        Self { path, inner: Inner { active_id: None, libraries: vec![] } }
    }

    pub fn load(path: PathBuf) -> Result<Self, ProcessingError> {
        let data = std::fs::read_to_string(&path)
            .map_err(ProcessingError::IoError)?;
        let inner: Inner = serde_json::from_str(&data)
            .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;
        Ok(Self { path, inner })
    }

    pub fn save(&self) -> Result<(), ProcessingError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(ProcessingError::IoError)?;
        }
        let data = serde_json::to_string_pretty(&self.inner)
            .map_err(|e| ProcessingError::MetadataError(e.to_string()))?;
        std::fs::write(&self.path, data).map_err(ProcessingError::IoError)
    }

    pub fn add_library(&mut self, entry: LibraryEntry) {
        self.inner.libraries.push(entry);
    }

    pub fn set_active(&mut self, id: &str) {
        self.inner.active_id = Some(id.to_string());
    }

    pub fn active_id(&self) -> Option<&str> {
        self.inner.active_id.as_deref()
    }

    pub fn active_library(&self) -> Option<&LibraryEntry> {
        let id = self.inner.active_id.as_deref()?;
        self.inner.libraries.iter().find(|e| e.id == id)
    }

    pub fn libraries(&self) -> &[LibraryEntry] {
        &self.inner.libraries
    }

    pub fn remove_library(&mut self, id: &str) {
        self.inner.libraries.retain(|e| e.id != id);
        if self.inner.active_id.as_deref() == Some(id) {
            self.inner.active_id = self.inner.libraries.first().map(|e| e.id.clone());
        }
    }
}
