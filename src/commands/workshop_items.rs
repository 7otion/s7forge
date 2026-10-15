use bincode::{Decode, Encode};
use futures_util::FutureExt;
use rustc_hash::FxHashMap;
use serde::Serialize;
use steamworks::{PublishedFileId, SteamId};

use crate::core::steam_manager;
use crate::core::workshop_item::workshop::{WorkshopItem, WorkshopItemsResult};
use crate::utils::fetch_creator_names::{UNKNOWN_NAME, fetch_creator_names};
use crate::utils::get_cache_dir::cache_file;
use crate::utils::timed_cache::{TimedCache, now_secs};

/// How long fetched item details (including "Steam returned nothing for this
/// id") are reused. Short, so that `time_updated` reflects new mod versions soon.
const ITEM_TTL_SECS: u64 = 30 * 60;

/// `None` records an id that Steam returned no Community item for.
type WorkshopItemCache = TimedCache<u64, Option<WorkshopItem>>;

#[derive(Debug, Clone, Serialize, Encode, Decode)]
pub struct EnhancedWorkshopItem {
    #[serde(flatten)]
    pub workshop_item: WorkshopItem,
    pub creator_id: String,
    pub creator_name: String,
}

impl EnhancedWorkshopItem {
    pub fn new(workshop_item: WorkshopItem, creator_id: String, creator_name: String) -> Self {
        Self {
            workshop_item,
            creator_id,
            creator_name,
        }
    }
}

pub async fn workshop_items(
    steam_game_id: u32,
    item_ids: Vec<u64>,
) -> Result<Vec<EnhancedWorkshopItem>, String> {
    if item_ids.is_empty() {
        return Ok(Vec::new());
    }

    let cache_path = cache_file("workshop_items_cache.bin");
    let mut cache = WorkshopItemCache::load(&cache_path);
    let now = now_secs();

    let mut ids_to_fetch: Vec<u64> = item_ids
        .iter()
        .filter(|id| cache.get(id, now, ITEM_TTL_SECS).is_none())
        .copied()
        .collect();
    ids_to_fetch.sort_unstable();
    ids_to_fetch.dedup();

    if !ids_to_fetch.is_empty() {
        let mut fetched: FxHashMap<u64, WorkshopItem> = query_items(steam_game_id, &ids_to_fetch)
            .await?
            .into_iter()
            .map(|item| (item.published_file_id, item))
            .collect();
        for id in ids_to_fetch {
            cache.insert(id, fetched.remove(&id), now);
        }
        cache.prune(now, ITEM_TTL_SECS);
        cache.save(&cache_path);
    }

    let items: Vec<WorkshopItem> = item_ids
        .iter()
        .filter_map(|id| cache.get(id, now, ITEM_TTL_SECS).cloned().flatten())
        .collect();

    with_creator_names(steam_game_id, items).await
}

/// Fetches item details from Steam, keeping only Community items.
async fn query_items(steam_game_id: u32, ids: &[u64]) -> Result<Vec<WorkshopItem>, String> {
    let ids_to_fetch = ids.to_vec();
    let steam_client = steam_manager::initialize_client(steam_game_id).await?;

    let (tx, mut rx) = tokio::sync::mpsc::channel(32);
    let items_task = tokio::task::spawn_blocking(move || {
        let ugc = steam_client.ugc();
        let (tx_inner, rx_inner) = std::sync::mpsc::channel();
        let query_handle = ugc
            .query_items(ids_to_fetch.iter().map(|id| PublishedFileId(*id)).collect())
            .map_err(|e| format!("Failed to create query handle: {:?}", e))?;

        query_handle
            .include_children(true)
            .set_return_long_description(true)
            .fetch(move |fetch_result| {
                let _ = tx_inner.send(
                    fetch_result
                        .map(|query_results| WorkshopItemsResult::from_query_results(query_results))
                        .map_err(|e| format!("Steam API error: {:?}", e)),
                );
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

    let mut items_result = None;
    let mut fused_task = items_task.fuse();

    while items_result.is_none() {
        tokio::select! {
            Some(_) = rx.recv() => {
                steam_manager::run_callbacks(steam_game_id)?;
            }
            task_result = &mut fused_task => {
                items_result = Some(
                    task_result.map_err(|e| format!("Task error: {:?}", e))?
                );
                break;
            }
        }
    }

    let items_result = items_result.unwrap()?;

    Ok(items_result
        .items
        .into_iter()
        .filter_map(|item| match item {
            Some(it) if it.file_type == "Community" => Some(it),
            _ => None,
        })
        .collect())
}

pub async fn with_creator_names(
    steam_game_id: u32,
    items: Vec<WorkshopItem>,
) -> Result<Vec<EnhancedWorkshopItem>, String> {
    let creator_ids: Vec<SteamId> = items
        .iter()
        .map(|item| SteamId::from_raw(item.owner.steam_id64))
        .collect();

    let creator_names = fetch_creator_names(creator_ids, steam_game_id).await?;

    Ok(items
        .into_iter()
        .map(|item| {
            let owner = item.owner.clone();
            let creator_name = creator_names
                .get(&item.owner.steam_id64)
                .cloned()
                .unwrap_or_else(|| UNKNOWN_NAME.to_string());
            EnhancedWorkshopItem::new(item, owner.steam_id64.to_string(), creator_name)
        })
        .collect())
}
