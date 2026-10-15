use crate::core::steam_library::library_paths;

pub fn steam_library_paths() -> Result<Vec<String>, String> {
    library_paths()
}
