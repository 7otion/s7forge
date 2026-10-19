//! The Steam client's own record of a game's Workshop subscriptions:
//! `<library>/steamapps/workshop/appworkshop_<app_id>.acf`, plus the account
//! that is logged in, from `<steam>/config/loginusers.vdf`.

use std::fs;
use std::path::{Path, PathBuf};

use crate::core::keyvalues::{self, Table};
use crate::core::steam_install_paths::steam_install_paths;
use crate::core::steam_library::{app_library, library_paths};

/// A 64-bit Steam id minus this is the 32-bit account id Steam writes into
/// the manifest's `subscribedby`.
const STEAM_ID_BASE: u64 = 76561197960265728;

#[derive(Debug, Default, PartialEq)]
pub struct WorkshopManifest {
    /// `<library>/steamapps/workshop/content/<app_id>`, next to the manifest.
    pub content_folder: Option<PathBuf>,
    pub items: Vec<ManifestItem>,
}

#[derive(Debug, PartialEq)]
pub struct ManifestItem {
    pub item_id: u64,
    pub subscribed_by: Option<u32>,
    /// `time_updated` of the newest version Steam knows about.
    pub latest_time_updated: Option<u64>,
    pub installed: Option<InstalledItem>,
}

#[derive(Debug, PartialEq)]
pub struct InstalledItem {
    pub size: u64,
    pub time_updated: u64,
}

pub fn manifest_file(app_id: u32) -> Result<Option<PathBuf>, String> {
    let name = format!("appworkshop_{}.acf", app_id);
    let in_library = |library: &Path| library.join("steamapps").join("workshop").join(&name);
    if let Some(library) = app_library(app_id)? {
        let file = in_library(&library);
        if file.is_file() {
            return Ok(Some(file));
        }
    }
    Ok(library_paths()?
        .iter()
        .map(|library| in_library(Path::new(library)))
        .find(|file| file.is_file()))
}

pub fn read_manifest(app_id: u32, file: &Path) -> Result<WorkshopManifest, String> {
    let text = fs::read_to_string(file)
        .map_err(|e| format!("Failed to read {}: {}", file.display(), e))?;
    let mut manifest =
        parse_manifest(&text).map_err(|e| format!("Failed to parse {}: {}", file.display(), e))?;
    manifest.content_folder = file
        .parent()
        .map(|workshop| workshop.join("content").join(app_id.to_string()));
    Ok(manifest)
}

pub fn parse_manifest(text: &str) -> Result<WorkshopManifest, String> {
    let root = keyvalues::parse(text).map_err(|e| e.to_string())?;
    let app = keyvalues::table(&root, "AppWorkshop").ok_or("no AppWorkshop table")?;
    let installed = keyvalues::table(app, "WorkshopItemsInstalled");
    let details = keyvalues::table(app, "WorkshopItemDetails");
    let mut items = Vec::new();
    for (id, detail) in details.into_iter().flatten() {
        let Ok(item_id) = id.parse::<u64>() else {
            continue;
        };
        let keyvalues::Value::Table(detail) = detail else {
            continue;
        };
        items.push(ManifestItem {
            item_id,
            subscribed_by: keyvalues::number(detail, "subscribedby").map(|id| id as u32),
            latest_time_updated: keyvalues::number(detail, "latest_timeupdated"),
            installed: installed
                .and_then(|installed| keyvalues::table(installed, id))
                .and_then(installed_item),
        });
    }
    Ok(WorkshopManifest {
        content_folder: None,
        items,
    })
}

fn installed_item(entry: &Table) -> Option<InstalledItem> {
    Some(InstalledItem {
        size: keyvalues::number(entry, "size")?,
        time_updated: keyvalues::number(entry, "timeupdated")?,
    })
}

/// The 32-bit account id of the user logged in to Steam on this PC, from the
/// first readable `loginusers.vdf`; `None` when it can't be told.
pub fn logged_in_account() -> Option<u32> {
    steam_install_paths()
        .ok()?
        .iter()
        .map(|steam| Path::new(steam).join("config").join("loginusers.vdf"))
        .filter_map(|file| fs::read_to_string(file).ok())
        .find_map(|text| logged_in_steam_id(&text))
        .map(account_id)
}

/// The user marked `MostRecent`, else the one with the newest `Timestamp`.
pub fn logged_in_steam_id(text: &str) -> Option<u64> {
    let root = keyvalues::parse(text).ok()?;
    let users = keyvalues::table(&root, "users")?;
    let mut newest: Option<(u64, u64)> = None;
    for (id, user) in users {
        let (Ok(steam_id), keyvalues::Value::Table(user)) = (id.parse::<u64>(), user) else {
            continue;
        };
        if keyvalues::text(user, "MostRecent") == Some("1") {
            return Some(steam_id);
        }
        let timestamp = keyvalues::number(user, "Timestamp").unwrap_or(0);
        if newest.is_none_or(|(_, newest)| timestamp > newest) {
            newest = Some((steam_id, timestamp));
        }
    }
    newest.map(|(steam_id, _)| steam_id)
}

pub fn account_id(steam_id: u64) -> u32 {
    steam_id.wrapping_sub(STEAM_ID_BASE) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    const MANIFEST: &str = r#"
"AppWorkshop"
{
	"appid"		"1142710"
	"NeedsUpdate"		"0"
	"WorkshopItemsInstalled"
	{
		"2789844937"
		{
			"size"		"125919"
			"timeupdated"		"1701529643"
			"manifest"		"6933476854845949769"
		}
	}
	"WorkshopItemDetails"
	{
		"2789844937"
		{
			"manifest"		"6933476854845949769"
			"timeupdated"		"1701529643"
			"subscribedby"		"288138212"
			"latest_timeupdated"		"1711111111"
		}
		"3000000000"
		{
			"subscribedby"		"288138212"
			"latest_timeupdated"		"1722222222"
		}
	}
}
"#;

    const LOGIN_USERS: &str = r#"
"users"
{
	"76561198000000001"
	{
		"AccountName"		"old"
		"Timestamp"		"1700000000"
	}
	"76561198248403940"
	{
		"AccountName"		"current"
		"Timestamp"		"1791715097"
	}
}
"#;

    #[test]
    fn installed_and_pending_items_are_read() {
        let manifest = parse_manifest(MANIFEST).unwrap();
        assert_eq!(
            manifest.items,
            vec![
                ManifestItem {
                    item_id: 2789844937,
                    subscribed_by: Some(288138212),
                    latest_time_updated: Some(1711111111),
                    installed: Some(InstalledItem {
                        size: 125919,
                        time_updated: 1701529643,
                    }),
                },
                ManifestItem {
                    item_id: 3000000000,
                    subscribed_by: Some(288138212),
                    latest_time_updated: Some(1722222222),
                    installed: None,
                },
            ]
        );
    }

    #[test]
    fn manifest_without_details_is_empty() {
        let manifest = parse_manifest("\"AppWorkshop\" { \"appid\" \"1\" }").unwrap();
        assert_eq!(manifest, WorkshopManifest::default());
    }

    #[test]
    fn wrong_root_is_an_error() {
        assert!(parse_manifest("\"Other\" { }").is_err());
    }

    #[test]
    fn newest_login_wins_without_most_recent_flag() {
        assert_eq!(logged_in_steam_id(LOGIN_USERS), Some(76561198248403940));
    }

    #[test]
    fn most_recent_flag_wins() {
        let text = LOGIN_USERS.replace(
            "\"AccountName\"\t\t\"old\"",
            "\"AccountName\"\t\t\"old\"\n\t\t\"MostRecent\"\t\t\"1\"",
        );
        assert_eq!(logged_in_steam_id(&text), Some(76561198000000001));
    }

    #[test]
    fn account_id_strips_the_steam_id_base() {
        assert_eq!(account_id(76561198248403940), 288138212);
    }
}
