use futures_util::FutureExt;
use serde::Serialize;
use steamworks::{PublishedFileId, SteamId};
use tokio::sync::mpsc;

use super::workshop_items::{EnhancedWorkshopItem, workshop_items};
use crate::core::steam_manager;
use crate::utils::fetch_creator_names::{UNKNOWN_NAME, fetch_creator_names};

#[derive(Debug, Serialize)]
pub struct CollectionInfo {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub creator_id: String,
    pub creator_name: String,
    pub preview_url: Option<String>,
    pub time_created: u64,
    pub time_updated: u64,
    pub num_upvotes: u32,
    pub num_downvotes: u32,
}

#[derive(Debug, Serialize)]
pub struct CollectionDetails {
    pub details: CollectionInfo,
    pub items: Vec<EnhancedWorkshopItem>,
}

pub async fn collection_items(
    steam_game_id: u32,
    item_id: u64,
) -> Result<CollectionDetails, String> {
    let steam_client = steam_manager::initialize_client(steam_game_id).await?;

    let (tx, mut rx) = mpsc::channel(32);

    let collection_task = tokio::task::spawn_blocking(move || {
        let ugc = steam_client.ugc();
        let collection_id = PublishedFileId(item_id);

        let (tx_inner, rx_inner) = std::sync::mpsc::channel();

        let query_handle = ugc
            .query_items(vec![collection_id])
            .map_err(|e| format!("Failed to create query handle: {:?}", e))?;

        query_handle
            .include_children(true)
            .set_return_long_description(true)
            .fetch(move |result| match result {
                Ok(details) => {
                    let Some(collection_info) = details.get(0) else {
                        let _ = tx_inner.send(Err(format!("Collection {} not found", item_id)));
                        return;
                    };

                    if collection_info.file_type != steamworks::FileType::Collection {
                        let _ = tx_inner.send(Err("Item is not a collection".to_string()));
                        return;
                    }

                    let collection_details = (
                        CollectionInfo {
                            id: collection_info.published_file_id.0,
                            title: collection_info.title.clone(),
                            description: collection_info.description.clone(),
                            creator_id: collection_info.owner.raw().to_string(),
                            creator_name: UNKNOWN_NAME.to_string(),
                            preview_url: details.preview_url(0),
                            time_created: collection_info.time_created as u64,
                            time_updated: collection_info.time_updated as u64,
                            num_upvotes: collection_info.num_upvotes,
                            num_downvotes: collection_info.num_downvotes,
                        },
                        details
                            .iter()
                            .enumerate()
                            .flat_map(|(index, item_opt)| match item_opt {
                                Some(_item) => details
                                    .get_children(index as u32)
                                    .unwrap_or_default()
                                    .into_iter()
                                    .map(|file_id| file_id.0)
                                    .collect::<Vec<u64>>(),
                                None => Vec::new(),
                            })
                            .collect::<Vec<u64>>(),
                    );
                    let _ = tx_inner.send(Ok(collection_details));
                }
                Err(e) => {
                    let _ = tx_inner.send(Err(format!("Steam API error: {}", e)));
                }
            });

        let start_time = std::time::Instant::now();
        let timeout_duration = std::time::Duration::from_secs(30);

        loop {
            let _ = tx.blocking_send(());
            if let Ok(result) = rx_inner.try_recv() {
                return result;
            }

            if start_time.elapsed() > timeout_duration {
                return Err("Operation timed out waiting for Steam response".to_string());
            }

            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    });

    let mut collection_result = None;
    let mut fused_task = collection_task.fuse();

    while collection_result.is_none() {
        tokio::select! {
            Some(_) = rx.recv() => {
                steam_manager::run_callbacks(steam_game_id)?;
            }
            task_result = &mut fused_task => {
                collection_result = Some(task_result.map_err(|e| format!("Task error: {:?}", e))??);
            }
        }
    }

    let (mut collection_info, item_ids) = collection_result.unwrap();
    let owner = SteamId::from_raw(collection_info.creator_id.parse().unwrap_or_default());
    if let Some(name) = fetch_creator_names(vec![owner], steam_game_id)
        .await?
        .remove(&owner.raw())
    {
        collection_info.creator_name = name;
    }
    let items = workshop_items(steam_game_id, item_ids).await?;

    Ok(CollectionDetails {
        details: collection_info,
        items,
    })
}
