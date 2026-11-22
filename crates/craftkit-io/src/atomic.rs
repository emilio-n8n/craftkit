//! Atomic file replacement used for project and settings files.

use std::io::Write;
use std::path::Path;

/// Write `bytes` to `path` through a temporary file in the same directory.
///
/// The temporary file is flushed and synced before the rename, so a crash
/// leaves either the previous file or the complete new one, never a truncated
/// project.
///
/// # Errors
///
/// Returns the underlying I/O error when the directory, temporary file, sync or
/// rename fails.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent)?;

    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    temporary.write_all(bytes)?;
    temporary.as_file().sync_all()?;
    temporary.persist(path).map_err(|error| error.error)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_and_replaces_a_file() {
        let directory = tempfile::tempdir().expect("tempdir");
        let path = directory.path().join("nested").join("data.bin");

        write_atomic(&path, b"first").expect("write first");
        assert_eq!(std::fs::read(&path).expect("read first"), b"first");

        write_atomic(&path, b"second").expect("write second");
        assert_eq!(std::fs::read(&path).expect("read second"), b"second");
    }
}
