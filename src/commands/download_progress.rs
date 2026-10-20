use serde::Serialize;
use steamworks::PublishedFileId;

use super::workshop_items::LocalState;
use crate::core::steam_manager;

#[derive(Debug, Serialize)]
pub struct DownloadProgress {
    pub item_id: u64,
    pub state: LocalState,
    pub downloaded_bytes: Option<u64>,
    pub total_bytes: Option<u64>,
    pub installed_time_updated: Option<u64>,
}

pub async fn download_progress(
    steam_game_id: u32,
    item_ids: Vec<u64>,
) -> Result<Vec<DownloadProgress>, String> {
    let steam_client = steam_manager::initialize_client(steam_game_id).await?;
    let ugc = steam_client.ugc();
    Ok(item_ids
        .into_iter()
        .map(|item_id| {
            let item = PublishedFileId(item_id);
            let (downloaded_bytes, total_bytes) = ugc.item_download_info(item).unzip();
            DownloadProgress {
                item_id,
                state: ugc.item_state(item).into(),
                downloaded_bytes,
                total_bytes,
                installed_time_updated: ugc
                    .item_install_info(item)
                    .map(|info| info.timestamp as u64),
            }
        })
        .collect())
}
