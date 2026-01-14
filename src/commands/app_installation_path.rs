use std::fs;
use std::path::Path;

use crate::commands::steam_library_paths::steam_library_paths;
use crate::utils::extract_quoted_strings::extract_quoted_strings;

pub fn app_installation_path(app_id: u32) -> Result<String, String> {
    let library_paths =
        steam_library_paths().map_err(|e| format!("Failed to get Steam library paths: {}", e))?;

    for library_path in library_paths {
        let steamapps_path = Path::new(&library_path).join("steamapps");

        let manifest_file = steamapps_path.join(format!("appmanifest_{}.acf", app_id));
        if !manifest_file.exists() {
            continue;
        }
        let manifest_content = fs::read_to_string(&manifest_file)
            .map_err(|e| format!("Failed to read manifest file: {}", e))?;

        let quoted_strings = extract_quoted_strings(&manifest_content);
        for i in 0..quoted_strings.len() {
            if quoted_strings[i] == "installdir" && i + 1 < quoted_strings.len() {
                let install_dir = &quoted_strings[i + 1];

                let full_path = steamapps_path.join("common").join(install_dir);
                if full_path.exists() {
                    return Ok(full_path.to_string_lossy().into_owned());
                } else {
                    return Err(format!(
                        "Installation directory exists in manifest but not on disk: {}",
                        full_path.display()
                    ));
                }
            }
        }

        return Err(format!(
            "Found manifest file but couldn't parse installation directory for app {}",
            app_id
        ));
    }

    Err(format!(
        "App {} is not installed or manifest file not found",
        app_id
    ))
}
