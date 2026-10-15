use std::path::{Path, PathBuf};

use crate::core::steam_library::{app_library, library_paths};

/// `<library>/steamapps/workshop/content/<app_id>`, if that folder exists.
///
/// Steam keeps workshop content in the library the game is installed in, so
/// that library is checked first; other libraries are a fallback.
pub fn workshop_path(app_id: u32) -> Option<String> {
    let content_folder = |library: &Path| -> PathBuf {
        library
            .join("steamapps")
            .join("workshop")
            .join("content")
            .join(app_id.to_string())
    };

    let from_app_library = app_library(app_id)
        .ok()
        .flatten()
        .map(|library| content_folder(&library));

    let from_any_library = || {
        library_paths()
            .ok()?
            .iter()
            .map(|library| content_folder(Path::new(library)))
            .find(|folder| folder.is_dir())
    };

    from_app_library
        .filter(|folder| folder.is_dir())
        .or_else(from_any_library)
        .map(|folder| folder.to_string_lossy().into_owned())
}
