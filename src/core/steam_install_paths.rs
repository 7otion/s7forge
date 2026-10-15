use winreg::RegKey;
use winreg::enums::*;

use crate::core::steam_library::dedupe_paths;

/// Folders of every Steam installation found in the registry.
///
/// Steam writes `HKCU\...\Valve\Steam\SteamPath` for the current user and
/// `HKLM\...\Valve\Steam\InstallPath` at install time (under WOW6432Node on 64-bit Windows).
pub fn steam_install_paths() -> Result<Vec<String>, String> {
    let lookups = [
        (HKEY_CURRENT_USER, "SOFTWARE\\Valve\\Steam", "SteamPath"),
        (
            HKEY_LOCAL_MACHINE,
            "SOFTWARE\\WOW6432Node\\Valve\\Steam",
            "InstallPath",
        ),
        (HKEY_LOCAL_MACHINE, "SOFTWARE\\Valve\\Steam", "InstallPath"),
    ];

    let paths = lookups
        .iter()
        .filter_map(|(hive, key, value)| {
            RegKey::predef(*hive)
                .open_subkey(key)
                .and_then(|key| key.get_value::<String, _>(value))
                .ok()
        })
        .collect();

    Ok(dedupe_paths(paths))
}
