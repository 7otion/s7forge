use serde::Serialize;
use steamworks::{ItemState, PublishedFileId};

use crate::core::steam_manager;

#[derive(Debug, PartialEq, Serialize)]
pub struct DownloadInfo {
    pub is_downloading: bool,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub progress_percentage: f32,
    pub download_complete: bool,
}

pub async fn check_item_download(steam_game_id: u32, item_id: u64) -> Result<DownloadInfo, String> {
    let steam_client = steam_manager::initialize_client(steam_game_id).await?;
    let ugc = steam_client.ugc();
    let item = PublishedFileId(item_id);

    // These are synchronous reads of the local Steam client state; no callbacks are involved.
    Ok(download_info(
        ugc.item_state(item),
        ugc.item_install_info(item).map(|info| info.size_on_disk),
        ugc.item_download_info(item),
    ))
}

fn download_info(
    state: ItemState,
    installed_size: Option<u64>,
    download_progress: Option<(u64, u64)>,
) -> DownloadInfo {
    let is_installed = state.contains(ItemState::INSTALLED);
    let needs_update = state.contains(ItemState::NEEDS_UPDATE);
    let is_downloading = state.intersects(ItemState::DOWNLOADING | ItemState::DOWNLOAD_PENDING);

    if is_downloading {
        let (downloaded_bytes, total_bytes) = download_progress.unwrap_or((0, 0));
        let progress_percentage = if total_bytes > 0 {
            (downloaded_bytes as f32 / total_bytes as f32) * 100.0
        } else {
            0.0
        };
        DownloadInfo {
            is_downloading: true,
            downloaded_bytes,
            total_bytes,
            progress_percentage,
            download_complete: false,
        }
    } else if is_installed && !needs_update {
        // Install info can be missing even when the state says INSTALLED (e.g. a deleted folder).
        let size = installed_size.unwrap_or(0);
        DownloadInfo {
            is_downloading: false,
            downloaded_bytes: size,
            total_bytes: size,
            progress_percentage: 100.0,
            download_complete: true,
        }
    } else {
        DownloadInfo {
            is_downloading: false,
            downloaded_bytes: 0,
            total_bytes: 0,
            progress_percentage: 0.0,
            download_complete: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SUBSCRIBED: ItemState = ItemState::SUBSCRIBED;

    #[test]
    fn installed_item_reports_size_on_disk() {
        let info = download_info(SUBSCRIBED | ItemState::INSTALLED, Some(42), None);
        assert!(info.download_complete);
        assert_eq!(info.total_bytes, 42);
        assert_eq!(info.progress_percentage, 100.0);
    }

    #[test]
    fn installed_item_without_install_info_does_not_panic() {
        // Steam reports INSTALLED but GetItemInstallInfo fails, e.g. the folder was deleted.
        let info = download_info(SUBSCRIBED | ItemState::INSTALLED, None, None);
        assert!(info.download_complete);
        assert_eq!(info.total_bytes, 0);
    }

    #[test]
    fn downloading_item_reports_progress() {
        let info = download_info(SUBSCRIBED | ItemState::DOWNLOADING, None, Some((25, 100)));
        assert!(info.is_downloading);
        assert_eq!(info.progress_percentage, 25.0);
    }

    #[test]
    fn downloading_item_without_progress_info_still_reports_downloading() {
        let info = download_info(SUBSCRIBED | ItemState::DOWNLOADING, None, None);
        assert!(info.is_downloading);
        assert!(!info.download_complete);
    }

    #[test]
    fn download_pending_counts_as_downloading() {
        let info = download_info(SUBSCRIBED | ItemState::DOWNLOAD_PENDING, None, None);
        assert!(info.is_downloading);
        assert!(!info.download_complete);
    }

    #[test]
    fn update_downloading_reports_progress_not_completion() {
        let state =
            SUBSCRIBED | ItemState::INSTALLED | ItemState::NEEDS_UPDATE | ItemState::DOWNLOADING;
        let info = download_info(state, Some(42), Some((10, 20)));
        assert!(info.is_downloading);
        assert!(!info.download_complete);
        assert_eq!(info.progress_percentage, 50.0);
    }

    #[test]
    fn not_subscribed_item_is_not_complete() {
        let info = download_info(ItemState::NONE, None, None);
        assert!(!info.is_downloading);
        assert!(!info.download_complete);
    }
}
