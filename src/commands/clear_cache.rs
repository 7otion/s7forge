use serde::{Deserialize, Serialize};
use std::fs;

use crate::utils::get_cache_dir::get_cache_dir;

#[derive(Serialize, Deserialize)]
pub struct ClearCacheResult {
    pub success: bool,
    pub message: String,
    pub files_cleared: usize,
    pub files: Vec<String>,
}

pub fn clear_cache() -> Result<ClearCacheResult, String> {
    let cache_dir = get_cache_dir();

    if !cache_dir.exists() {
        return Ok(ClearCacheResult {
            success: true,
            message: "Cache directory does not exist, nothing to clear".to_string(),
            files_cleared: 0,
            files: Vec::new(),
        });
    }

    let mut cleared_files = Vec::new();
    let mut errors = Vec::new();

    let entries =
        fs::read_dir(&cache_dir).map_err(|e| format!("Failed to read cache directory: {:?}", e))?;

    for entry in entries {
        match entry {
            Ok(file_entry) => {
                let file_path = file_entry.path();
                if file_path.is_file() {
                    match fs::remove_file(&file_path) {
                        Ok(_) => {
                            if let Some(file_name) = file_path.file_name() {
                                cleared_files.push(file_name.to_string_lossy().to_string());
                            }
                        }
                        Err(e) => {
                            errors.push(format!("Failed to remove {}: {}", file_path.display(), e));
                        }
                    }
                }
            }
            Err(e) => {
                errors.push(format!("Failed to read directory entry: {}", e));
            }
        }
    }

    if !errors.is_empty() {
        return Err(format!(
            "Errors occurred while clearing cache: {}",
            errors.join(", ")
        ));
    }

    let result = if cleared_files.is_empty() {
        ClearCacheResult {
            success: true,
            message: "Cache directory was already empty".to_string(),
            files_cleared: 0,
            files: Vec::new(),
        }
    } else {
        ClearCacheResult {
            success: true,
            message: format!("Successfully cleared {} cache files", cleared_files.len()),
            files_cleared: cleared_files.len(),
            files: cleared_files,
        }
    };

    Ok(result)
}
