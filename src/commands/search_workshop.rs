use bincode::{Decode, Encode};
use futures_util::FutureExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use steamworks::{AppIDs, AppId, UGCQueryType, UGCType};
use tokio::sync::mpsc;

use crate::commands::workshop_items::EnhancedWorkshopItem;
use crate::core::steam_manager;
use crate::core::workshop_item::workshop::{WorkshopItem, WorkshopItemsResult};
use crate::utils::fetch_creator_names::fetch_creator_names;
use crate::utils::get_cache_dir::get_cache_dir;

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
struct SearchCacheKey {
    steam_game_id: u32,
    search_text: String,
    sort_by: String,
    period: Option<String>,
    page: u32,
    tags: Option<String>,
}

#[derive(Debug, Clone, Serialize, Encode, Decode)]
struct CachedSearchResult {
    items: Vec<EnhancedWorkshopItem>,
    timestamp: u64,
}

#[derive(Debug, Default, Serialize, Encode, Decode)]
struct SearchCache {
    entries: HashMap<SearchCacheKey, CachedSearchResult>,
}

impl SearchCache {
    const CACHE_DURATION_MINUTES: u64 = 10;

    fn load_from_disk() -> Self {
        match Self::get_cache_file_path() {
            Ok(cache_path) => {
                if cache_path.exists() {
                    match fs::read(&cache_path) {
                        Ok(data) => {
                            let config = bincode::config::standard();
                            match bincode::decode_from_slice(&data, config) {
                                Ok((cache, _)) => {
                                    let mut cleaned_cache: SearchCache = cache;
                                    cleaned_cache.clean_expired_entries();
                                    return cleaned_cache;
                                }
                                Err(e) => {
                                    eprintln!("Failed to decode search cache: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to read search cache file: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to get cache file path: {}", e);
            }
        }
        Self::default()
    }

    fn save_to_disk(&self) -> Result<(), String> {
        let cache_path = Self::get_cache_file_path()?;
        let config = bincode::config::standard();
        let encoded = bincode::encode_to_vec(self, config)
            .map_err(|e| format!("Failed to encode search cache: {}", e))?;

        fs::write(&cache_path, encoded)
            .map_err(|e| format!("Failed to write search cache to disk: {}", e))?;

        Ok(())
    }

    fn get_cache_file_path() -> Result<PathBuf, String> {
        let cache_dir = get_cache_dir()?;
        Ok(cache_dir.join("search_workshop_cache.bin"))
    }

    fn clean_expired_entries(&mut self) {
        let now = Self::current_timestamp();
        let expiry_duration_secs = Self::CACHE_DURATION_MINUTES * 60;

        self.entries.retain(|_, cached_result| {
            now.saturating_sub(cached_result.timestamp) < expiry_duration_secs
        });
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs()
    }

    fn get(&mut self, key: &SearchCacheKey) -> Option<Vec<EnhancedWorkshopItem>> {
        self.clean_expired_entries();

        if let Some(cached_result) = self.entries.get(key) {
            let now = Self::current_timestamp();
            let expiry_duration_secs = Self::CACHE_DURATION_MINUTES * 60;

            if now.saturating_sub(cached_result.timestamp) < expiry_duration_secs {
                return Some(cached_result.items.clone());
            } else {
                self.entries.remove(key);
            }
        }
        None
    }

    fn insert(&mut self, key: SearchCacheKey, items: Vec<EnhancedWorkshopItem>) {
        let cached_result = CachedSearchResult {
            items,
            timestamp: Self::current_timestamp(),
        };
        self.entries.insert(key, cached_result);

        self.clean_expired_entries();

        if let Err(e) = self.save_to_disk() {
            eprintln!("Warning: Failed to save search cache to disk: {}", e);
        }
    }
}

pub async fn search_workshop(
    steam_game_id: u32,
    search_text: String,
    sort_by: String,
    period: Option<String>,
    page: u32,
    tags: Option<String>,
) -> Result<Vec<EnhancedWorkshopItem>, String> {
    if page == 0 {
        return Err("Page number must be at least 1".to_string());
    }
    let cache_key = SearchCacheKey {
        steam_game_id,
        search_text: search_text.clone(),
        sort_by: sort_by.clone(),
        period: period.clone(),
        page,
        tags: tags.clone(),
    };

    let mut cache = SearchCache::load_from_disk();
    if let Some(cached_result) = cache.get(&cache_key) {
        return Ok(cached_result);
    }

    let steam_client = steam_manager::initialize_client(steam_game_id).await?;

    let (tx, mut rx) = mpsc::channel(32);

    let search_task = tokio::task::spawn_blocking(move || {
        let ugc = steam_client.ugc();
        let (tx_inner, rx_inner) = std::sync::mpsc::channel();
        let app_ids = AppIDs::Both {
            creator: AppId(steam_game_id),
            consumer: AppId(steam_game_id),
        };
        let query_type = match sort_by.as_str() {
            "relevance" => UGCQueryType::RankedByTextSearch,
            "recent" => UGCQueryType::RankedByPublicationDate,
            "popular" => UGCQueryType::RankedByTrend,
            "most-subscribed" => UGCQueryType::RankedByTotalUniqueSubscriptions,
            "recently-updated" => UGCQueryType::RankedByLastUpdatedDate,
            _ => UGCQueryType::RankedByTextSearch,
        };

        let query_handle = ugc
            .query_all(query_type, UGCType::Items, app_ids, page)
            .map_err(|e| format!("Failed to create search query: {:?}", e))?;

        let mut configured_query = query_handle
            .set_return_metadata(true)
            .set_return_children(true)
            .set_return_additional_previews(true)
            .set_return_key_value_tags(true);

        if !search_text.trim().is_empty() {
            configured_query = configured_query.set_search_text(&search_text);
        }
        if query_type == UGCQueryType::RankedByTrend {
            let period_str = period.as_deref().unwrap_or("one-week");
            let trend_days = match period_str {
                "today" => 1,
                "one-week" => 7,
                "three-months" => 90,
                "six-months" => 180,
                "one-year" => 365,
                _ => 7,
            };
            configured_query = configured_query.set_ranked_by_trend_days(trend_days);
        } else if period.is_some() {
            return Err("Period filter is only applicable for popular sort type".to_string());
        }

        if let Some(ref tag_filter) = tags {
            let tag_list: Vec<&str> = tag_filter.split(',').map(|s| s.trim()).collect();
            for tag in tag_list {
                if !tag.is_empty() {
                    configured_query = configured_query.add_required_tag(tag);
                }
            }
        }

        configured_query.fetch(move |fetch_result| {
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
                return Err("Search operation timed out waiting for Steam response".to_string());
            }

            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    });

    let mut search_result = None;
    let mut fused_task = search_task.fuse();

    while search_result.is_none() {
        tokio::select! {
            Some(_) = rx.recv() => {
                steam_manager::run_callbacks(steam_game_id)?;
            }
            task_result = &mut fused_task => {
                search_result = Some(task_result.map_err(|e| format!("Task error: {:?}", e))??);
                break;
            }
        }
    }

    let items_result = search_result.unwrap();
    let workshop_items = items_result
        .items
        .into_iter()
        .filter_map(|item| match item {
            Some(it) if it.file_type == "Community" => Some(it),
            _ => None,
        })
        .collect::<Vec<WorkshopItem>>();

    if workshop_items.is_empty() {
        return Ok(Vec::new());
    }

    let creator_ids: Vec<steamworks::SteamId> = workshop_items
        .iter()
        .map(|item| steamworks::SteamId::from_raw(item.owner.steam_id64))
        .collect();

    let creator_names = fetch_creator_names(creator_ids, steam_game_id).await?;

    let result: Vec<EnhancedWorkshopItem> = workshop_items
        .into_iter()
        .map(|item| {
            let owner = item.owner.clone();
            let creator_name = creator_names
                .get(&item.owner.steam_id64)
                .cloned()
                .unwrap_or_else(|| "[unknown]".to_string());
            EnhancedWorkshopItem::new(item, owner.steam_id64.to_string(), creator_name)
        })
        .collect();

    cache.insert(cache_key, result.clone());

    Ok(result)
}
