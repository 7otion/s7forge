use bincode::{Decode, Encode};
use rustc_hash::FxHashMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::steam_install_paths::steam_install_paths;
use crate::utils::extract_quoted_strings::extract_quoted_strings;
use crate::utils::get_cache_dir::get_cache_dir;

#[derive(Debug, Encode, Decode)]
struct WorkshopPathCache {
    paths: FxHashMap<u32, Option<String>>,
    timestamp: u64,
}

pub fn workshop_path(app_id: u32) -> Option<String> {
    // Try to load from cache
    if let Ok(cache_dir) = get_cache_dir() {
        let cache_path = cache_dir.join("workshop_path_cache.bin");
        if cache_path.exists() {
            if let Ok(cache_content) = fs::read(&cache_path) {
                let config = bincode::config::standard();
                if let Ok((cache, _)) =
                    bincode::decode_from_slice::<WorkshopPathCache, _>(&cache_content, config)
                {
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    let cache_duration_secs = 60 * 60; // 1 hour

                    if now.saturating_sub(cache.timestamp) < cache_duration_secs {
                        if let Some(cached_result) = cache.paths.get(&app_id) {
                            return cached_result.clone();
                        }
                    }
                }
            }
        }
    }

    // Compute the result
    let result = 'search: {
        match steam_install_paths() {
            Ok(paths) => {
                for steam_install_path in paths {
                    let library_meta_file = Path::new(&steam_install_path)
                        .join("steamapps")
                        .join("libraryfolders.vdf");

                    if !library_meta_file.exists() {
                        continue;
                    }

                    let file_data = match fs::read_to_string(&library_meta_file) {
                        Ok(data) => data,
                        Err(_) => continue,
                    };

                    let quoted_strings = extract_quoted_strings(&file_data);

                    let mut library_folder_paths = Vec::new();
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

                    for lib_path in &library_folder_paths {
                        let workshop_path = Path::new(lib_path)
                            .join("steamapps")
                            .join("workshop")
                            .join("content")
                            .join(app_id.to_string());

                        if workshop_path.exists() {
                            break 'search Some(workshop_path.to_string_lossy().into_owned());
                        }
                    }
                }
                None
            }
            Err(_) => None,
        }
    };

    // Save to cache
    if let Ok(cache_dir) = get_cache_dir() {
        let _ = fs::create_dir_all(&cache_dir);
        let cache_path = cache_dir.join("workshop_path_cache.bin");

        let mut cache = if cache_path.exists() {
            if let Ok(cache_content) = fs::read(&cache_path) {
                let config = bincode::config::standard();
                bincode::decode_from_slice::<WorkshopPathCache, _>(&cache_content, config)
                    .map(|(c, _)| c)
                    .unwrap_or_else(|_| WorkshopPathCache {
                        paths: FxHashMap::default(),
                        timestamp: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs(),
                    })
            } else {
                WorkshopPathCache {
                    paths: FxHashMap::default(),
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                }
            }
        } else {
            WorkshopPathCache {
                paths: FxHashMap::default(),
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            }
        };

        cache.paths.insert(app_id, result.clone());
        cache.timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let config = bincode::config::standard();
        if let Ok(encoded) = bincode::encode_to_vec(&cache, config) {
            let _ = fs::write(&cache_path, encoded);
        }
    }

    result
}
