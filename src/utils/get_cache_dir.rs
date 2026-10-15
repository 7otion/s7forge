use std::fs;
use std::path::{Path, PathBuf};

pub const CACHE_DIR_ENV: &str = "S7FORGE_CACHE_DIR";

/// `S7FORGE_CACHE_DIR` when set, otherwise `%LOCALAPPDATA%\s7forge\cache`.
///
/// The directory is not created here. Every cache is optional: a cache that
/// cannot be read or written behaves like an empty cache.
pub fn get_cache_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os(CACHE_DIR_ENV).filter(|dir| !dir.is_empty()) {
        return PathBuf::from(dir);
    }

    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("s7forge")
        .join("cache")
}

/// Bump whenever a cached struct changes shape. bincode files written with an
/// older layout are then ignored instead of being decoded into wrong values.
const CACHE_FORMAT_VERSION: u32 = 3;

pub fn cache_file(name: &str) -> PathBuf {
    get_cache_dir().join(format!("v{}_{}", CACHE_FORMAT_VERSION, name))
}

/// Best-effort write; failures are ignored because the cache is optional.
pub fn write_cache_file(path: &Path, contents: &[u8]) {
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    let _ = fs::write(path, contents);
}
