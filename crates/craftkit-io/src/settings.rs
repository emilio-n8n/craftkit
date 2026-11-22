//! Persisted application settings with a tolerant loader.

use std::path::{Path, PathBuf};

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::atomic::write_atomic;
use crate::error::{IoError, io_error, json_error};

/// JSON-backed settings value.
///
/// A missing file is normal. A corrupt file falls back to defaults and logs a
/// warning: losing UI preferences is preferable to refusing to start.
#[derive(Debug)]
pub struct SettingsStore<T> {
    path: Option<PathBuf>,
    value: T,
}

impl<T: Default + Serialize + DeserializeOwned> SettingsStore<T> {
    /// Load settings from `path`, or keep defaults when the path is `None`.
    #[must_use]
    pub fn load(path: Option<PathBuf>) -> Self {
        let value = path.as_deref().map(load_value).unwrap_or_default();
        Self { path, value }
    }

    /// Read the current settings.
    #[must_use]
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Mutate the settings in memory. Call [`Self::save`] to persist them.
    pub fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }

    /// Persist the current settings.
    ///
    /// # Errors
    ///
    /// Returns an I/O or serialization error. A store without a path is
    /// memory-only and always succeeds.
    pub fn save(&self) -> Result<(), IoError> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        let bytes =
            serde_json::to_vec_pretty(&self.value).map_err(|error| json_error(path, error))?;
        write_atomic(path, &bytes).map_err(|error| io_error(path, error))
    }

    /// Path backing this store, if any.
    #[must_use]
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }
}

fn load_value<T: Default + DeserializeOwned>(path: &Path) -> T {
    match std::fs::read(path) {
        Ok(bytes) => match serde_json::from_slice(&bytes) {
            Ok(value) => value,
            Err(error) => {
                log::warn!("ignoring corrupt settings {}: {error}", path.display());
                T::default()
            }
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => T::default(),
        Err(error) => {
            log::warn!("cannot read settings {}: {error}", path.display());
            T::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    struct TestSettings {
        theme: String,
        limit: usize,
    }

    #[test]
    fn missing_file_loads_defaults() {
        let directory = tempfile::tempdir().expect("tempdir");
        let store =
            SettingsStore::<TestSettings>::load(Some(directory.path().join("settings.json")));
        assert_eq!(*store.value(), TestSettings::default());
    }

    #[test]
    fn settings_round_trip() {
        let directory = tempfile::tempdir().expect("tempdir");
        let path = directory.path().join("settings.json");
        let mut store = SettingsStore::<TestSettings>::load(Some(path.clone()));
        store.value_mut().theme = "light".into();
        store.value_mut().limit = 12;
        store.save().expect("save");

        let loaded = SettingsStore::<TestSettings>::load(Some(path));
        assert_eq!(loaded.value().theme, "light");
        assert_eq!(loaded.value().limit, 12);
    }

    #[test]
    fn corrupt_file_falls_back_to_defaults() {
        let directory = tempfile::tempdir().expect("tempdir");
        let path = directory.path().join("settings.json");
        std::fs::write(&path, b"{not json").expect("write");
        let store = SettingsStore::<TestSettings>::load(Some(path));
        assert_eq!(*store.value(), TestSettings::default());
    }

    #[test]
    fn memory_store_does_not_fail() {
        let store = SettingsStore::<TestSettings>::load(None);
        store.save().expect("memory save");
        assert!(store.path().is_none());
    }
}
