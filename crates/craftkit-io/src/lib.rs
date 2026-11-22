//! File and settings support for CraftKit applications.
//!
//! The crate knows how to persist an application document through a versioned
//! envelope, but it never defines the document schema itself.

pub mod atomic;
pub mod document;
pub mod error;
pub mod paths;
pub mod recent;
pub mod settings;

pub use document::DocumentHost;
pub use error::IoError;
pub use paths::AppPaths;
pub use recent::RecentFiles;
pub use settings::SettingsStore;
