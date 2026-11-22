//! Native file dialogs.

use std::path::PathBuf;

/// Ask the user to choose an existing project file.
#[must_use]
pub fn open_project(title: &str, extensions: &[&str]) -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title(title)
        .add_filter("CraftKit project", extensions)
        .pick_file()
}

/// Ask the user where to save a project.
///
/// The first extension is appended when the chosen name has none.
#[must_use]
pub fn save_project(title: &str, default_name: &str, extension: &str) -> Option<PathBuf> {
    let path = rfd::FileDialog::new()
        .set_title(title)
        .add_filter("CraftKit project", &[extension])
        .set_file_name(default_name)
        .save_file()?;
    if path.extension().is_some() {
        Some(path)
    } else {
        let mut path = path;
        path.set_extension(extension);
        Some(path)
    }
}
