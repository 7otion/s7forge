use std::fs;
use std::path::{Path, PathBuf};

use steamworks::AppId;

use crate::core::steam_library::library_paths;
use crate::core::steam_manager;
use crate::utils::extract_quoted_strings::extract_quoted_strings;
use crate::utils::fast_path::{Fast, with_fallback};

/// `<library>/steamapps/common/<installdir>`, read from the app's
/// `appmanifest_<id>.acf`; asked of the Steam client when the files can't be read.
pub async fn app_installation_path(app_id: u32) -> Result<String, String> {
    with_fallback(
        "app-installation-path",
        || from_files(app_id),
        || from_steam(app_id),
    )
    .await
    .map(|(_, path)| path)
}

fn from_files(app_id: u32) -> Fast<String> {
    let libraries = match library_paths() {
        Ok(libraries) => libraries,
        Err(reason) => {
            return Fast::Failed(format!("Failed to get Steam library paths: {}", reason));
        }
    };
    let manifest = format!("appmanifest_{}.acf", app_id);
    let Some(steamapps) = libraries
        .iter()
        .map(|library| Path::new(library).join("steamapps"))
        .find(|steamapps| steamapps.join(&manifest).is_file())
    else {
        return Fast::Answer(Err(format!(
            "App {} is not installed or manifest file not found",
            app_id
        )));
    };
    let content = match fs::read_to_string(steamapps.join(&manifest)) {
        Ok(content) => content,
        Err(reason) => return Fast::Failed(format!("Failed to read manifest file: {}", reason)),
    };
    let Some(install_dir) = extract_quoted_strings(&content)
        .windows(2)
        .find(|pair| pair[0] == "installdir")
        .map(|pair| pair[1].clone())
    else {
        return Fast::Failed(format!(
            "Found manifest file but couldn't parse installation directory for app {}",
            app_id
        ));
    };
    Fast::Answer(existing(steamapps.join("common").join(install_dir)))
}

async fn from_steam(app_id: u32) -> Result<String, String> {
    let steam_client = steam_manager::initialize_client(app_id).await?;
    existing(PathBuf::from(
        steam_client.apps().app_install_dir(AppId(app_id)),
    ))
}

fn existing(path: PathBuf) -> Result<String, String> {
    if path.is_dir() {
        Ok(path.to_string_lossy().into_owned())
    } else {
        Err(format!(
            "Installation directory exists in manifest but not on disk: {}",
            path.display()
        ))
    }
}
