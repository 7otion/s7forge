use std::fs;

use crate::core::steam_library::app_library;
use crate::utils::extract_quoted_strings::extract_quoted_strings;

/// `<library>/steamapps/common/<installdir>`, read from the app's `appmanifest_<id>.acf`.
pub fn app_installation_path(app_id: u32) -> Result<String, String> {
    let library = app_library(app_id)
        .map_err(|e| format!("Failed to get Steam library paths: {}", e))?
        .ok_or_else(|| format!("App {} is not installed or manifest file not found", app_id))?;

    let steamapps_path = library.join("steamapps");
    let manifest_content =
        fs::read_to_string(steamapps_path.join(format!("appmanifest_{}.acf", app_id)))
            .map_err(|e| format!("Failed to read manifest file: {}", e))?;

    let install_dir = extract_quoted_strings(&manifest_content)
        .windows(2)
        .find(|pair| pair[0] == "installdir")
        .map(|pair| pair[1].clone())
        .ok_or_else(|| {
            format!(
                "Found manifest file but couldn't parse installation directory for app {}",
                app_id
            )
        })?;

    let full_path = steamapps_path.join("common").join(install_dir);
    if full_path.exists() {
        Ok(full_path.to_string_lossy().into_owned())
    } else {
        Err(format!(
            "Installation directory exists in manifest but not on disk: {}",
            full_path.display()
        ))
    }
}
