//! Project lifecycle around an application document.

use std::path::{Path, PathBuf};

use craftkit_core::{History, ProjectEnvelope};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::atomic::write_atomic;
use crate::error::{IoError, io_error, json_error};

/// Owns a document, its undo history and its file path.
///
/// Applications mutate the document through [`DocumentHost::apply`] or the
/// exposed [`DocumentHost::history_mut`]. Every save writes a versioned
/// [`ProjectEnvelope`] atomically and marks the current state as clean.
#[derive(Debug)]
pub struct DocumentHost<T> {
    history: History<T>,
    path: Option<PathBuf>,
    format: String,
    schema_version: u32,
    app_version: String,
}

impl<T> DocumentHost<T>
where
    T: Clone + PartialEq + Serialize + DeserializeOwned,
{
    /// Create an unsaved document.
    #[must_use]
    pub fn new(
        document: T,
        format: impl Into<String>,
        schema_version: u32,
        app_version: impl Into<String>,
    ) -> Self {
        Self {
            history: History::new(document),
            path: None,
            format: format.into(),
            schema_version,
            app_version: app_version.into(),
        }
    }

    /// Read and validate a project file.
    ///
    /// # Errors
    ///
    /// Returns an error for unreadable files, invalid JSON, a project from
    /// another application or a newer schema version.
    pub fn from_file(
        path: impl AsRef<Path>,
        format: impl Into<String>,
        schema_version: u32,
        app_version: impl Into<String>,
    ) -> Result<Self, IoError> {
        let path = path.as_ref();
        let bytes = std::fs::read(path).map_err(|error| io_error(path, error))?;
        let envelope: ProjectEnvelope<T> =
            serde_json::from_slice(&bytes).map_err(|error| json_error(path, error))?;
        let format = format.into();
        envelope.validate(&format, schema_version)?;
        Ok(Self {
            history: History::new(envelope.document),
            path: Some(path.to_path_buf()),
            format,
            schema_version,
            app_version: app_version.into(),
        })
    }

    /// Current document.
    #[must_use]
    pub fn document(&self) -> &T {
        self.history.current()
    }

    /// Undo/redo and save-point state.
    #[must_use]
    pub fn history(&self) -> &History<T> {
        &self.history
    }

    /// Mutable history for direct gestures or custom workflow.
    pub fn history_mut(&mut self) -> &mut History<T> {
        &mut self.history
    }

    /// Apply a document mutation and record one undo step.
    ///
    /// Returns `true` when the document changed.
    pub fn apply(&mut self, mutate: impl FnOnce(&mut T)) -> bool {
        self.history.apply(mutate)
    }

    /// Undo one step.
    pub fn undo(&mut self) -> bool {
        self.history.undo()
    }

    /// Redo one step.
    pub fn redo(&mut self) -> bool {
        self.history.redo()
    }

    /// Whether undo is available.
    #[must_use]
    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    /// Whether redo is available.
    #[must_use]
    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    /// Whether the document differs from the last saved state.
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.history.is_dirty()
    }

    /// Path of the loaded project, if any.
    #[must_use]
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Display name for the title bar.
    #[must_use]
    pub fn display_name(&self) -> String {
        self.path
            .as_deref()
            .and_then(Path::file_name)
            .and_then(std::ffi::OsStr::to_str)
            .map_or_else(|| "Untitled".to_owned(), str::to_owned)
    }

    /// Replace the document, for example after New or an import.
    ///
    /// The file path is cleared and the history is reset.
    pub fn reset(&mut self, document: T) {
        self.history.replace(document);
        self.path = None;
    }

    /// Open a project into this host, replacing the current document.
    ///
    /// # Errors
    ///
    /// See [`DocumentHost::from_file`].
    pub fn open(&mut self, path: impl AsRef<Path>) -> Result<(), IoError> {
        let loaded = Self::from_file(
            path,
            self.format.clone(),
            self.schema_version,
            self.app_version.clone(),
        )?;
        *self = loaded;
        Ok(())
    }

    /// Save to the current path.
    ///
    /// # Errors
    ///
    /// Returns [`IoError::NoPath`] when the document has never been saved.
    pub fn save(&mut self) -> Result<PathBuf, IoError> {
        let Some(path) = self.path.clone() else {
            return Err(IoError::NoPath);
        };
        self.write_to(&path)?;
        self.history.mark_saved();
        Ok(path)
    }

    /// Save to a new path and remember it.
    ///
    /// # Errors
    ///
    /// Returns an I/O or serialization error. On failure the previous path and
    /// dirty state are preserved.
    pub fn save_as(&mut self, path: impl AsRef<Path>) -> Result<PathBuf, IoError> {
        let path = path.as_ref().to_path_buf();
        self.write_to(&path)?;
        self.path = Some(path.clone());
        self.history.mark_saved();
        Ok(path)
    }

    /// Format identifier written to the envelope.
    #[must_use]
    pub fn format(&self) -> &str {
        &self.format
    }

    /// Schema version written to the envelope.
    #[must_use]
    pub fn schema_version(&self) -> u32 {
        self.schema_version
    }

    fn write_to(&self, path: &Path) -> Result<(), IoError> {
        #[derive(Serialize)]
        struct EnvelopeRef<'a, T> {
            format: &'a str,
            schema_version: u32,
            app_version: &'a str,
            document: &'a T,
        }

        let mut bytes = serde_json::to_vec_pretty(&EnvelopeRef {
            format: &self.format,
            schema_version: self.schema_version,
            app_version: &self.app_version,
            document: self.history.current(),
        })
        .map_err(|error| json_error(path, error))?;
        bytes.push(b'\n');
        write_atomic(path, &bytes).map_err(|error| io_error(path, error))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    struct Document {
        title: String,
        value: i32,
    }

    fn document() -> Document {
        Document {
            title: "Demo".into(),
            value: 1,
        }
    }

    fn host() -> DocumentHost<Document> {
        DocumentHost::new(document(), "demo", 1, "0.1.0")
    }

    #[test]
    fn save_and_load_round_trip() {
        let directory = tempfile::tempdir().expect("tempdir");
        let path = directory.path().join("project.json");

        let mut host = host();
        host.apply(|document| document.value = 7);
        host.save_as(&path).expect("save");
        assert!(!host.is_dirty());

        let loaded = DocumentHost::<Document>::from_file(&path, "demo", 1, "0.1.0").expect("load");
        assert_eq!(loaded.document().value, 7);
        assert_eq!(loaded.path(), Some(path.as_path()));
        assert!(!loaded.is_dirty());
    }

    #[test]
    fn save_without_path_is_an_error() {
        let mut host = host();
        assert!(matches!(host.save(), Err(IoError::NoPath)));
    }

    #[test]
    fn rejects_a_foreign_project() {
        let directory = tempfile::tempdir().expect("tempdir");
        let path = directory.path().join("other.json");
        let envelope = ProjectEnvelope::new("other", 1, "0.1.0", document());
        std::fs::write(&path, serde_json::to_vec(&envelope).expect("json")).expect("write");

        let error = DocumentHost::<Document>::from_file(&path, "demo", 1, "0.1.0")
            .expect_err("foreign format");
        assert!(matches!(
            error,
            IoError::Format(craftkit_core::ProjectFormatError::WrongFormat { .. })
        ));
    }

    #[test]
    fn rejects_a_newer_schema() {
        let directory = tempfile::tempdir().expect("tempdir");
        let path = directory.path().join("future.json");
        let envelope = ProjectEnvelope::new("demo", 99, "0.1.0", document());
        std::fs::write(&path, serde_json::to_vec(&envelope).expect("json")).expect("write");

        let error =
            DocumentHost::<Document>::from_file(&path, "demo", 1, "0.1.0").expect_err("future");
        assert!(matches!(
            error,
            IoError::Format(craftkit_core::ProjectFormatError::UnsupportedSchema { .. })
        ));
    }

    #[test]
    fn undo_and_redo_round_trip() {
        let mut host = host();
        host.apply(|document| document.value = 2);
        assert!(host.is_dirty());
        assert!(host.undo());
        assert_eq!(host.document().value, 1);
        assert!(host.redo());
        assert_eq!(host.document().value, 2);
    }

    #[test]
    fn reset_clears_path_and_history() {
        let directory = tempfile::tempdir().expect("tempdir");
        let path = directory.path().join("project.json");
        let mut host = host();
        host.save_as(&path).expect("save");
        host.apply(|document| document.value = 2);
        host.reset(document());
        assert!(host.path().is_none());
        assert!(!host.can_undo());
        assert!(!host.is_dirty());
    }
}
