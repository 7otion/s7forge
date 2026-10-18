use std::fs;
use std::path::{Path, PathBuf};

use crate::core::steam_install_paths::steam_install_paths;
use crate::utils::extract_quoted_strings::extract_quoted_strings;

/// Library folder paths listed in a `libraryfolders.vdf`.
///
/// `extract_quoted_strings` already unescapes `\\`, so values are used as-is.
pub fn parse_library_folders(vdf: &str) -> Vec<String> {
    let quoted_strings = extract_quoted_strings(vdf);
    quoted_strings
        .windows(2)
        .filter(|pair| pair[0] == "path")
        .map(|pair| pair[1].clone())
        .collect()
}

/// Removes paths that point to the same folder, keeping the first spelling.
pub fn dedupe_paths(paths: Vec<String>) -> Vec<String> {
    let mut seen = Vec::new();
    paths
        .into_iter()
        .filter(|path| {
            let key = path_key(path);
            if seen.contains(&key) {
                false
            } else {
                seen.push(key);
                true
            }
        })
        .collect()
}

/// Windows paths are case-insensitive, and Steam writes some with forward slashes.
#[cfg(windows)]
fn path_key(path: &str) -> String {
    path.replace('/', "\\")
        .trim_end_matches('\\')
        .to_lowercase()
}

#[cfg(not(windows))]
fn path_key(_path: &str) -> String {
    unimplemented!("comparing Steam library paths on this platform")
}

/// Every Steam library folder, read from each Steam installation's `libraryfolders.vdf`.
pub fn library_paths() -> Result<Vec<String>, String> {
    let mut paths = Vec::new();
    for steam_path in steam_install_paths()? {
        let vdf = Path::new(&steam_path)
            .join("steamapps")
            .join("libraryfolders.vdf");
        if let Ok(content) = fs::read_to_string(&vdf) {
            paths.extend(parse_library_folders(&content));
        }
    }
    Ok(dedupe_paths(paths))
}

/// The library whose `steamapps` folder holds the app's manifest, i.e. where it is installed.
pub fn app_library(app_id: u32) -> Result<Option<PathBuf>, String> {
    let manifest = format!("appmanifest_{}.acf", app_id);
    Ok(library_paths()?
        .into_iter()
        .map(PathBuf::from)
        .find(|library| library.join("steamapps").join(&manifest).is_file()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const VDF: &str = r#"
"libraryfolders"
{
	"0"
	{
		"path"		"C:\\Program Files (x86)\\Steam"
		"label"		""
		"apps"
		{
			"228980"		"370237915"
		}
	}
	"1"
	{
		"path"		"\\\\nas\\games\\SteamLibrary"
		"apps"
		{
		}
	}
}
"#;

    #[test]
    fn parses_local_library_paths() {
        assert_eq!(
            parse_library_folders(VDF)[0],
            r"C:\Program Files (x86)\Steam"
        );
    }

    #[test]
    fn keeps_unc_library_paths_intact() {
        assert_eq!(parse_library_folders(VDF)[1], r"\\nas\games\SteamLibrary");
    }

    #[test]
    fn dedupes_paths_that_differ_only_in_case_or_separators() {
        let paths = vec![
            "c:/program files (x86)/steam".to_string(),
            r"C:\Program Files (x86)\Steam\".to_string(),
            r"D:\SteamLibrary".to_string(),
        ];
        assert_eq!(
            dedupe_paths(paths),
            vec![
                "c:/program files (x86)/steam".to_string(),
                r"D:\SteamLibrary".to_string()
            ]
        );
    }
}
