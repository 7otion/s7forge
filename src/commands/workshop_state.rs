use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use steamworks::{ItemState, PublishedFileId};
use tokio::task;

use crate::commands::workshop_path::workshop_path;
use crate::core::steam_manager;
use crate::core::workshop_manifest::{self, ManifestItem};
use crate::utils::fast_path::{Fast, Route, with_fallback};

#[derive(Debug, Serialize)]
pub struct WorkshopState {
    pub route: Route,
    pub account_id: Option<u32>,
    pub items: Vec<WorkshopStateItem>,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct WorkshopStateItem {
    pub item_id: u64,
    pub installed: bool,
    pub folder: Option<String>,
    pub size_on_disk: Option<u64>,
    pub installed_time_updated: Option<u64>,
    pub latest_time_updated: Option<u64>,
    pub needs_update: bool,
    pub download_pending: bool,
}

pub async fn workshop_state(steam_game_id: u32) -> Result<WorkshopState, String> {
    let steam_error = match with_fallback(
        "workshop-state",
        || from_files(steam_game_id),
        || from_steam(steam_game_id),
    )
    .await
    {
        Ok((route, (account_id, items))) => {
            return Ok(WorkshopState {
                route,
                account_id,
                items,
            });
        }
        Err(error) => error,
    };
    let content_folder = workshop_path(steam_game_id).ok_or(steam_error)?;
    Ok(WorkshopState {
        route: Route::Folder,
        account_id: None,
        items: from_folder(Path::new(&content_folder))?,
    })
}

fn from_folder(content_folder: &Path) -> Result<Vec<WorkshopStateItem>, String> {
    let entries = fs::read_dir(content_folder)
        .map_err(|e| format!("Failed to list {}: {}", content_folder.display(), e))?;
    let mut items = Vec::new();
    for entry in entries.flatten() {
        let folder = entry.path();
        if let Some(item_id) = item_id_of(&folder) {
            items.push(state_from_folder(item_id, folder));
        }
    }
    items.sort_by_key(|item| item.item_id);
    Ok(items)
}

fn item_id_of(folder: &Path) -> Option<u64> {
    if !folder.is_dir() {
        return None;
    }
    folder.file_name()?.to_str()?.parse().ok()
}

fn state_from_folder(item_id: u64, folder: PathBuf) -> WorkshopStateItem {
    WorkshopStateItem {
        item_id,
        installed: true,
        folder: Some(folder.to_string_lossy().into_owned()),
        size_on_disk: None,
        installed_time_updated: None,
        latest_time_updated: None,
        needs_update: false,
        download_pending: false,
    }
}

type Subscriptions = (Option<u32>, Vec<WorkshopStateItem>);

fn from_files(app_id: u32) -> Fast<Subscriptions> {
    let file = match workshop_manifest::manifest_file(app_id) {
        Ok(Some(file)) => file,
        Ok(None) => return Fast::Answer(Ok((workshop_manifest::logged_in_account(), Vec::new()))),
        Err(reason) => return Fast::Failed(reason),
    };
    let manifest = match workshop_manifest::read_manifest(app_id, &file) {
        Ok(manifest) => manifest,
        Err(reason) => return Fast::Failed(reason),
    };
    let account_id = workshop_manifest::logged_in_account();
    let content_folder = manifest.content_folder;
    let items = manifest
        .items
        .into_iter()
        .filter(|item| belongs_to(item, account_id))
        .map(|item| state_from_manifest(item, content_folder.as_deref()))
        .collect();
    Fast::Answer(Ok((account_id, items)))
}

fn belongs_to(item: &ManifestItem, account_id: Option<u32>) -> bool {
    match (item.subscribed_by, account_id) {
        (Some(owner), Some(account)) => owner == account,
        _ => true,
    }
}

fn state_from_manifest(item: ManifestItem, content_folder: Option<&Path>) -> WorkshopStateItem {
    let installed_time_updated = item
        .installed
        .as_ref()
        .map(|installed| installed.time_updated);
    let folder = item.installed.as_ref().and_then(|_| {
        content_folder.map(|folder| {
            folder
                .join(item.item_id.to_string())
                .to_string_lossy()
                .into_owned()
        })
    });
    WorkshopStateItem {
        item_id: item.item_id,
        installed: item.installed.is_some(),
        folder,
        size_on_disk: item.installed.as_ref().map(|installed| installed.size),
        installed_time_updated,
        latest_time_updated: item.latest_time_updated,
        needs_update: match (installed_time_updated, item.latest_time_updated) {
            (Some(installed), Some(latest)) => latest > installed,
            _ => false,
        },
        download_pending: item.installed.is_none(),
    }
}

async fn from_steam(steam_game_id: u32) -> Result<Subscriptions, String> {
    let steam_client = steam_manager::initialize_client(steam_game_id).await?;
    let account_id = Some(steam_client.user().steam_id().account_id().raw());
    let items = task::spawn_blocking(move || {
        let ugc = steam_client.ugc();
        ugc.subscribed_items()
            .into_iter()
            .map(|id| {
                let state = ugc.item_state(id);
                let install = ugc.item_install_info(id);
                state_from_client(
                    id,
                    state,
                    install.map(|info| (info.folder, info.size_on_disk, info.timestamp as u64)),
                )
            })
            .collect()
    })
    .await
    .map_err(|e| format!("Failed to read subscriptions: {:?}", e))?;
    Ok((account_id, items))
}

fn state_from_client(
    id: PublishedFileId,
    state: ItemState,
    install: Option<(String, u64, u64)>,
) -> WorkshopStateItem {
    WorkshopStateItem {
        item_id: id.0,
        installed: state.contains(ItemState::INSTALLED),
        folder: install.as_ref().map(|(folder, _, _)| folder.clone()),
        size_on_disk: install.as_ref().map(|(_, size, _)| *size),
        installed_time_updated: install.map(|(_, _, time_updated)| time_updated),
        latest_time_updated: None,
        needs_update: state.contains(ItemState::NEEDS_UPDATE),
        download_pending: state.intersects(ItemState::DOWNLOADING | ItemState::DOWNLOAD_PENDING),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::workshop_manifest::InstalledItem;

    const CONTENT: &str = "C:\\w";

    fn item(
        installed: Option<(u64, u64)>,
        latest: Option<u64>,
        owner: Option<u32>,
    ) -> ManifestItem {
        ManifestItem {
            item_id: 1,
            subscribed_by: owner,
            latest_time_updated: latest,
            installed: installed.map(|(size, time_updated)| InstalledItem { size, time_updated }),
        }
    }

    #[test]
    fn newer_published_version_needs_update() {
        let state = state_from_manifest(
            item(Some((10, 100)), Some(200), None),
            Some(Path::new(CONTENT)),
        );
        assert!(state.installed);
        assert!(state.needs_update);
        assert!(!state.download_pending);
        assert_eq!(state.size_on_disk, Some(10));
        assert_eq!(state.folder.as_deref(), Some("C:\\w\\1"));
    }

    #[test]
    fn not_downloaded_item_is_pending() {
        let state = state_from_manifest(item(None, Some(200), None), Some(Path::new(CONTENT)));
        assert!(!state.installed);
        assert!(state.download_pending);
        assert!(!state.needs_update);
        assert_eq!(state.folder, None);
    }

    #[test]
    fn other_accounts_subscriptions_are_dropped_only_when_both_known() {
        assert!(belongs_to(&item(None, None, Some(7)), Some(7)));
        assert!(!belongs_to(&item(None, None, Some(7)), Some(8)));
        assert!(belongs_to(&item(None, None, None), Some(8)));
        assert!(belongs_to(&item(None, None, Some(7)), None));
    }

    #[test]
    fn folder_names_that_are_ids_become_installed_items() {
        let root =
            std::env::temp_dir().join(format!("s7forge-folder-route-{}", std::process::id()));
        for name in ["2858004741", "notanid", "12"] {
            fs::create_dir_all(root.join(name)).unwrap();
        }
        fs::write(root.join("99.txt"), b"").unwrap();
        let items = from_folder(&root).unwrap();
        fs::remove_dir_all(&root).unwrap();
        let ids: Vec<u64> = items.iter().map(|item| item.item_id).collect();
        assert_eq!(ids, vec![12, 2858004741]);
        assert!(
            items
                .iter()
                .all(|item| item.installed && item.folder.is_some())
        );
        assert!(items.iter().all(|item| item.size_on_disk.is_none()));
    }

    #[test]
    fn client_state_maps_flags() {
        let state = state_from_client(
            PublishedFileId(5),
            ItemState::SUBSCRIBED | ItemState::INSTALLED | ItemState::NEEDS_UPDATE,
            Some(("C:\\w\\5".to_owned(), 42, 123)),
        );
        assert_eq!(state.item_id, 5);
        assert!(state.installed && state.needs_update && !state.download_pending);
        assert_eq!(state.installed_time_updated, Some(123));
        assert_eq!(state.latest_time_updated, None);
        assert_eq!(state.folder.as_deref(), Some("C:\\w\\5"));
    }
}
