//! Versioned envelope around an application document.

use serde::{Deserialize, Serialize};

/// Serialized project header plus the application document.
///
/// The envelope lets every CraftKit application detect a file that belongs to
/// another application and refuse a schema version it does not understand.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectEnvelope<T> {
    /// Application identifier, for example `musiccraft`.
    pub format: String,
    /// Document schema version written by the application.
    pub schema_version: u32,
    /// Application version that wrote the file, for diagnostics.
    pub app_version: String,
    /// Application-specific document payload.
    pub document: T,
}

impl<T> ProjectEnvelope<T> {
    /// Build an envelope.
    #[must_use]
    pub fn new(
        format: impl Into<String>,
        schema_version: u32,
        app_version: impl Into<String>,
        document: T,
    ) -> Self {
        Self {
            format: format.into(),
            schema_version,
            app_version: app_version.into(),
            document,
        }
    }

    /// Check that the file belongs to this application and is not newer than
    /// the schema this build supports.
    ///
    /// Older schemas are accepted: the application is responsible for migrating
    /// them. A newer schema is rejected to avoid silent data loss.
    ///
    /// # Errors
    ///
    /// Returns [`ProjectFormatError`] for a foreign format or a future schema.
    pub fn validate(
        &self,
        expected_format: &str,
        supported_schema: u32,
    ) -> Result<(), ProjectFormatError> {
        if self.format != expected_format {
            return Err(ProjectFormatError::WrongFormat {
                expected: expected_format.to_owned(),
                actual: self.format.clone(),
            });
        }
        if self.schema_version > supported_schema {
            return Err(ProjectFormatError::UnsupportedSchema {
                found: self.schema_version,
                supported: supported_schema,
            });
        }
        Ok(())
    }
}

/// Errors detected before an application-specific migration runs.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ProjectFormatError {
    /// The file belongs to another application.
    #[error("expected a {expected} project, found {actual}")]
    WrongFormat {
        /// Format this build expects.
        expected: String,
        /// Format found in the file.
        actual: String,
    },
    /// The file was written by a newer schema.
    #[error("unsupported schema version {found} (this build supports up to {supported})")]
    UnsupportedSchema {
        /// Version found in the file.
        found: u32,
        /// Highest version this build supports.
        supported: u32,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_round_trips() {
        let envelope = ProjectEnvelope::new("demo", 1, "0.1.0", vec![1_u8, 2, 3]);
        let json = serde_json::to_string(&envelope).expect("serialize");
        let parsed: ProjectEnvelope<Vec<u8>> = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(envelope, parsed);
    }

    #[test]
    fn foreign_format_is_rejected() {
        let envelope = ProjectEnvelope::new("other", 1, "0.1.0", ());
        let error = envelope.validate("demo", 1).expect_err("wrong format");
        assert!(matches!(error, ProjectFormatError::WrongFormat { .. }));
    }

    #[test]
    fn future_schema_is_rejected_but_older_is_accepted() {
        let future = ProjectEnvelope::new("demo", 3, "0.1.0", ());
        assert!(future.validate("demo", 2).is_err());

        let older = ProjectEnvelope::new("demo", 1, "0.1.0", ());
        assert!(older.validate("demo", 2).is_ok());
    }
}
