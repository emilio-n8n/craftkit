//! Bounded, deduplicated list of recently opened files.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Default number of remembered files.
pub const DEFAULT_RECENT_LIMIT: usize = 10;

/// Most-recent-first list of project paths.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecentFiles {
    paths: Vec<PathBuf>,
    limit: usize,
}

impl Default for RecentFiles {
    fn default() -> Self {
        Self::new(DEFAULT_RECENT_LIMIT)
    }
}

impl RecentFiles {
    /// Create an empty list with the given bound (at least one entry).
    #[must_use]
    pub fn new(limit: usize) -> Self {
        Self {
            paths: Vec::new(),
            limit: limit.max(1),
        }
    }

    /// Paths, most recent first.
    #[must_use]
    pub fn list(&self) -> &[PathBuf] {
        &self.paths
    }

    /// Add a path, moving it to the front when it is already present.
    pub fn add(&mut self, path: impl Into<PathBuf>) {
        let path = path.into();
        self.paths.retain(|existing| existing != &path);
        self.paths.insert(0, path);
        self.paths.truncate(self.limit);
    }

    /// Remove a path.
    pub fn remove(&mut self, path: &Path) {
        self.paths.retain(|existing| existing != path);
    }

    /// Remove every remembered path.
    pub fn clear(&mut self) {
        self.paths.clear();
    }

    /// Drop entries whose file no longer exists.
    pub fn prune_missing(&mut self) {
        self.paths.retain(|path| path.exists());
    }

    /// Maximum number of entries.
    #[must_use]
    pub fn limit(&self) -> usize {
        self.limit
    }

    /// Number of entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.paths.len()
    }

    /// Whether the list is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deduplicates_and_keeps_most_recent_first() {
        let mut recent = RecentFiles::new(3);
        recent.add("a.json");
        recent.add("b.json");
        recent.add("a.json");
        assert_eq!(
            recent.list(),
            &[PathBuf::from("a.json"), PathBuf::from("b.json")]
        );
    }

    #[test]
    fn respects_the_limit() {
        let mut recent = RecentFiles::new(2);
        recent.add("a.json");
        recent.add("b.json");
        recent.add("c.json");
        assert_eq!(
            recent.list(),
            &[PathBuf::from("c.json"), PathBuf::from("b.json")]
        );
    }

    #[test]
    fn removes_and_clears() {
        let mut recent = RecentFiles::new(3);
        recent.add("a.json");
        recent.add("b.json");
        recent.remove(Path::new("a.json"));
        assert_eq!(recent.list(), &[PathBuf::from("b.json")]);
        recent.clear();
        assert!(recent.is_empty());
    }
}
