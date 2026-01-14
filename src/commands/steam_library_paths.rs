use std::{fs, path::Path};

use crate::core::steam_install_paths::steam_install_paths;
use crate::utils::extract_quoted_strings::extract_quoted_strings;

pub fn steam_library_paths() -> Result<Vec<String>, String> {
    let steam_install_paths = steam_install_paths()?;
    let mut library_folder_paths = Vec::new();

    for steam_install_path in steam_install_paths {
        let library_meta_file = Path::new(&steam_install_path)
            .join("steamapps")
            .join("libraryfolders.vdf");

        if !library_meta_file.exists() {
            continue;
        }

        let file_data = fs::read_to_string(&library_meta_file)
            .map_err(|e| format!("Failed to read library metadata file: {:?}", e))?;

        let quoted_strings = extract_quoted_strings(&file_data);

        for i in 0..quoted_strings.len() {
            let current_string = &quoted_strings[i];
            if current_string == "path" && i + 1 < quoted_strings.len() {
                let lib_path = Path::new(&quoted_strings[i + 1])
                    .to_str()
                    .unwrap_or("")
                    .to_string();
                library_folder_paths.push(lib_path.replace("\\\\", "\\"));
            }
        }
    }

    Ok(library_folder_paths)
}
