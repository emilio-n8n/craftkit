//! UI-free foundations shared by CraftKit applications.
//!
//! The crate deliberately knows nothing about files, windows or a particular
//! document schema. An application defines its own document type and wraps it
//! in [`History`]; [`ProjectEnvelope`] adds the metadata needed to persist it.

pub mod history;
pub mod project;

pub use history::History;
pub use project::{ProjectEnvelope, ProjectFormatError};
