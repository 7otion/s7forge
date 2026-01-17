use bincode::{Decode, Encode};
use rustc_hash::FxHashMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::commands::steam_library_paths::steam_library_paths;
use crate::utils::extract_quoted_strings::extract_quoted_strings;
use crate::utils::get_cache_dir::get_cache_dir;

#[derive(Debug, Encode, Decode)]
struct AppInstallPathCache {
    paths: FxHashMap<u32, Result<String, String>>,
    timestamp: u64,
}

pub fn app_installation_path(app_id: u32) -> Result<String, String> {
    // Try to load from cache
    if let Ok(cache_dir) = get_cache_dir() {
        let cache_path = cache_dir.join("app_install_path_cache.bin");
        if cache_path.exists() {
            if let Ok(cache_content) = fs::read(&cache_path) {
                let config = bincode::config::standard();
                if let Ok((cache, _)) =
                    bincode::decode_from_slice::<AppInstallPathCache, _>(&cache_content, config)
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

    let library_paths =
        steam_library_paths().map_err(|e| format!("Failed to get Steam library paths: {}", e))?;

    let result = 'outer: {
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
                    break 'outer if full_path.exists() {
                        Ok(full_path.to_string_lossy().into_owned())
                    } else {
                        Err(format!(
                            "Installation directory exists in manifest but not on disk: {}",
                            full_path.display()
                        ))
                    };
                }
            }

            break 'outer Err(format!(
                "Found manifest file but couldn't parse installation directory for app {}",
                app_id
            ));
        }

        Err(format!(
            "App {} is not installed or manifest file not found",
            app_id
        ))
    };

    // Save to cache
    if let Ok(cache_dir) = get_cache_dir() {
        let _ = fs::create_dir_all(&cache_dir);
        let cache_path = cache_dir.join("app_install_path_cache.bin");

        let mut cache = if cache_path.exists() {
            if let Ok(cache_content) = fs::read(&cache_path) {
                let config = bincode::config::standard();
                bincode::decode_from_slice::<AppInstallPathCache, _>(&cache_content, config)
                    .map(|(c, _)| c)
                    .unwrap_or_else(|_| AppInstallPathCache {
                        paths: FxHashMap::default(),
                        timestamp: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs(),
                    })
            } else {
                AppInstallPathCache {
                    paths: FxHashMap::default(),
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                }
            }
        } else {
            AppInstallPathCache {
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
