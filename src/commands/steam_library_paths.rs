use bincode::{Decode, Encode};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, path::Path};

use crate::core::steam_install_paths::steam_install_paths;
use crate::utils::extract_quoted_strings::extract_quoted_strings;
use crate::utils::get_cache_dir::get_cache_dir;

#[derive(Debug, Encode, Decode)]
struct LibraryPathsCache {
    paths: Vec<String>,
    timestamp: u64,
}

pub fn steam_library_paths() -> Result<Vec<String>, String> {
    // Try to load from cache
    if let Ok(cache_dir) = get_cache_dir() {
        let cache_path = cache_dir.join("library_paths_cache.bin");
        if cache_path.exists() {
            if let Ok(cache_content) = fs::read(&cache_path) {
                let config = bincode::config::standard();
                if let Ok((cache, _)) =
                    bincode::decode_from_slice::<LibraryPathsCache, _>(&cache_content, config)
                {
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    let cache_duration_secs = 60 * 60; // 1 hour

                    if now.saturating_sub(cache.timestamp) < cache_duration_secs {
                        return Ok(cache.paths);
                    }
                }
            }
        }
    }

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

    // Save to cache
    if let Ok(cache_dir) = get_cache_dir() {
        let _ = fs::create_dir_all(&cache_dir);
        let cache_path = cache_dir.join("library_paths_cache.bin");

        let cache = LibraryPathsCache {
            paths: library_folder_paths.clone(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        let config = bincode::config::standard();
        if let Ok(encoded) = bincode::encode_to_vec(&cache, config) {
            let _ = fs::write(&cache_path, encoded);
        }
    }

    Ok(library_folder_paths)
}
