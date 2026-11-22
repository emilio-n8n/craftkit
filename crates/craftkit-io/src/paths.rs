//! Platform application directories without an extra dependency.

use std::path::PathBuf;

/// Per-user directories for an application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    /// Configuration and settings.
    pub config: PathBuf,
    /// Application data such as documents and databases.
    pub data: PathBuf,
    /// Regenerable caches.
    pub cache: PathBuf,
}

impl AppPaths {
    /// Resolve the conventional directories for `app`.
    ///
    /// `app` must be a single path component such as `musiccraft`. Returns
    /// `None` when the environment has no home directory or the identifier is
    /// unsafe to use as a directory name.
    #[must_use]
    pub fn for_app(app: &str) -> Option<Self> {
        if !valid_id(app) {
            return None;
        }

        #[cfg(target_os = "windows")]
        {
            let base = std::env::var_os("APPDATA")
                .map(PathBuf::from)
                .or_else(|| home().map(|home| home.join("AppData").join("Roaming")))?;
            let local = std::env::var_os("LOCALAPPDATA")
                .map(PathBuf::from)
                .or_else(|| home().map(|home| home.join("AppData").join("Local")))?;
            Some(Self {
                config: base.join(app),
                data: base.join(app),
                cache: local.join(app).join("cache"),
            })
        }

        #[cfg(target_os = "macos")]
        {
            let home = home()?;
            let support = home.join("Library").join("Application Support");
            Some(Self {
                config: support.join(app),
                data: support.join(app),
                cache: home.join("Library").join("Caches").join(app),
            })
        }

        #[cfg(all(unix, not(target_os = "macos")))]
        {
            let home = home()?;
            let config = std::env::var_os("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join(".config"));
            let data = std::env::var_os("XDG_DATA_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join(".local").join("share"));
            let cache = std::env::var_os("XDG_CACHE_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join(".cache"));
            Some(Self {
                config: config.join(app),
                data: data.join(app),
                cache: cache.join(app),
            })
        }
    }

    /// Settings file inside the configuration directory.
    #[must_use]
    pub fn settings_file(&self, name: &str) -> PathBuf {
        self.config.join(name)
    }

    /// File inside the application data directory.
    #[must_use]
    pub fn data_file(&self, name: &str) -> PathBuf {
        self.data.join(name)
    }
}

fn valid_id(app: &str) -> bool {
    !app.is_empty()
        && app != "."
        && app != ".."
        && !app.contains(['/', '\\'])
        && app
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
}

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unsafe_application_ids() {
        assert!(AppPaths::for_app("").is_none());
        assert!(AppPaths::for_app("../escape").is_none());
        assert!(AppPaths::for_app("nested/app").is_none());
        assert!(AppPaths::for_app("nested\\app").is_none());
    }

    #[test]
    fn builds_named_files() {
        let paths = AppPaths {
            config: PathBuf::from("/cfg"),
            data: PathBuf::from("/data"),
            cache: PathBuf::from("/cache"),
        };
        assert_eq!(
            paths.settings_file("ui.json"),
            PathBuf::from("/cfg/ui.json")
        );
        assert_eq!(
            paths.data_file("project.db"),
            PathBuf::from("/data/project.db")
        );
    }
}
