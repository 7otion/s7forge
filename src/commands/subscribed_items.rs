use futures_util::FutureExt;
use rustc_hash::FxHashMap;
use serde::Serialize;
use steamworks::{AppIDs, AppId, PublishedFileId, UGCType, UserList, UserListOrder};
use tokio::task;

use crate::commands::workshop_items::{EnhancedWorkshopItem, workshop_items};
use crate::core::steam_manager;
use crate::utils::get_cache_dir::cache_file;
use crate::utils::timed_cache::{TimedCache, now_secs};

const SUBSCRIPTION_TTL_SECS: u64 = 365 * 24 * 60 * 60;
const PAGE_SIZE: usize = 50;

type SubscriptionTimeCache = TimedCache<u64, u64>;

#[derive(Debug, Clone, Serialize)]
pub struct SubscriptionTime {
    pub item_id: u64,
    pub time_subscribed: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SubscribedItem {
    #[serde(flatten)]
    pub item: EnhancedWorkshopItem,
    pub time_subscribed: Option<u64>,
}

pub async fn subscribed_items(steam_game_id: u32) -> Result<Vec<SubscribedItem>, String> {
    let steam_client = steam_manager::initialize_client(steam_game_id).await?;

    let subscribed_items: Vec<PublishedFileId> = task::spawn_blocking({
        let steam_client = steam_client.clone();
        move || steam_client.ugc().subscribed_items()
    })
    .await
    .map_err(|e| format!("Failed to fetch subscribed items: {:?}", e))?;

    let item_ids: Vec<u64> = subscribed_items.iter().map(|id| id.0).collect();
    if item_ids.is_empty() {
        return Ok(Vec::new());
    }

    let items = workshop_items(steam_game_id, item_ids.clone()).await?;
    let times = subscription_times(steam_game_id, &item_ids).await;
    Ok(items
        .into_iter()
        .map(|item| {
            let time_subscribed = times.get(&item.workshop_item.published_file_id).copied();
            SubscribedItem {
                item,
                time_subscribed,
            }
        })
        .collect())
}

pub async fn subscription_times_of_account(
    steam_game_id: u32,
) -> Result<Vec<SubscriptionTime>, String> {
    let steam_client = steam_manager::initialize_client(steam_game_id).await?;
    let item_ids: Vec<u64> = task::spawn_blocking(move || steam_client.ugc().subscribed_items())
        .await
        .map_err(|e| format!("Failed to fetch subscribed items: {:?}", e))?
        .into_iter()
        .map(|id| id.0)
        .collect();
    let times = subscription_times(steam_game_id, &item_ids).await;
    Ok(item_ids
        .into_iter()
        .filter_map(|item_id| {
            times
                .get(&item_id)
                .map(|&time_subscribed| SubscriptionTime {
                    item_id,
                    time_subscribed,
                })
        })
        .collect())
}

async fn subscription_times(steam_game_id: u32, item_ids: &[u64]) -> FxHashMap<u64, u64> {
    let cache_path = cache_file("subscription_times_cache.bin");
    let mut cache = SubscriptionTimeCache::load(&cache_path);
    let now = now_secs();

    let mut page = 1;
    loop {
        let Ok(entries) = subscription_page(steam_game_id, page).await else {
            break;
        };
        let mut reached_known = false;
        for &(item_id, time) in &entries {
            if cache.get(&item_id, now, SUBSCRIPTION_TTL_SECS) == Some(&time) {
                reached_known = true;
            }
            cache.insert(item_id, time, now);
        }
        let has_every_time = item_ids
            .iter()
            .all(|item_id| cache.get(item_id, now, SUBSCRIPTION_TTL_SECS).is_some());
        if entries.len() < PAGE_SIZE || (reached_known && has_every_time) {
            break;
        }
        page += 1;
    }

    cache.prune(now, SUBSCRIPTION_TTL_SECS);
    cache.save(&cache_path);
    item_ids
        .iter()
        .filter_map(|item_id| {
            cache
                .get(item_id, now, SUBSCRIPTION_TTL_SECS)
                .map(|time| (*item_id, *time))
        })
        .collect()
}

async fn subscription_page(steam_game_id: u32, page: u32) -> Result<Vec<(u64, u64)>, String> {
    let steam_client = steam_manager::initialize_client(steam_game_id).await?;
    let account = steam_client.user().steam_id().account_id();

    let (tx, mut rx) = tokio::sync::mpsc::channel(32);
    let page_task = task::spawn_blocking(move || {
        let (tx_inner, rx_inner) = std::sync::mpsc::channel();
        steam_client
            .ugc()
            .query_user(
                account,
                UserList::Subscribed,
                UGCType::Items,
                UserListOrder::SubscriptionDateDesc,
                AppIDs::ConsumerAppId(AppId(steam_game_id)),
                page,
            )
            .map_err(|e| format!("Failed to create query handle: {:?}", e))?
            .fetch(move |fetch_result| {
                let _ = tx_inner.send(
                    fetch_result
                        .map(|results| {
                            results
                                .iter()
                                .flatten()
                                .map(|item| {
                                    (
                                        item.published_file_id.0,
                                        u64::from(item.time_added_to_user_list),
                                    )
                                })
                                .collect::<Vec<_>>()
                        })
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

    let mut fused_task = page_task.fuse();
    loop {
        tokio::select! {
            Some(_) = rx.recv() => {
                steam_manager::run_callbacks(steam_game_id)?;
            }
            task_result = &mut fused_task => {
                return task_result.map_err(|e| format!("Task error: {:?}", e))?;
            }
        }
    }
}
