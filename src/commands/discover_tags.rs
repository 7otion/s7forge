use futures_util::FutureExt;
use std::collections::HashSet;
use steamworks::{AppIDs, AppId, UGCQueryType, UGCType};
use tokio::sync::mpsc;

use crate::core::steam_manager;
use crate::core::workshop_item::workshop::WorkshopItemsResult;

pub async fn discover_tags(steam_game_id: u32) -> Result<Vec<String>, String> {
    let steam_client = steam_manager::initialize_client(steam_game_id).await?;
    let mut all_tags = HashSet::new();

    let sampling_tasks = vec![
        ("Popular (All Time)", UGCQueryType::RankedByVote, None),
        ("Popular (This Week)", UGCQueryType::RankedByTrend, Some(7)),
        (
            "Popular (This Month)",
            UGCQueryType::RankedByTrend,
            Some(30),
        ),
        ("Recent Items", UGCQueryType::RankedByPublicationDate, None),
        (
            "Most Subscribed",
            UGCQueryType::RankedByTotalUniqueSubscriptions,
            None,
        ),
    ];

    for (_source_name, query_type, trend_days) in sampling_tasks {
        match sample_tags_from_source(&steam_client, steam_game_id, query_type, trend_days).await {
            Ok(tags) => {
                all_tags.extend(tags);
            }
            Err(_e) => {}
        }
    }

    let mut tag_list: Vec<String> = all_tags.into_iter().collect();
    tag_list.sort();

    Ok(tag_list)
}

async fn sample_tags_from_source(
    steam_client: &steamworks::Client,
    steam_game_id: u32,
    query_type: UGCQueryType,
    trend_days: Option<u32>,
) -> Result<HashSet<String>, String> {
    let (tx, mut rx) = mpsc::channel(32);

    let client_clone = steam_client.clone();
    let search_task = tokio::task::spawn_blocking(move || {
        let ugc = client_clone.ugc();
        let (tx_inner, rx_inner) = std::sync::mpsc::channel();
        let app_ids = AppIDs::Both {
            creator: AppId(steam_game_id),
            consumer: AppId(steam_game_id),
        };

        let query_handle = ugc
            .query_all(query_type, UGCType::Items, app_ids, 1)
            .map_err(|e| format!("Failed to create query: {:?}", e))?;

        let configured_query = if let Some(days) = trend_days {
            query_handle.set_ranked_by_trend_days(days)
        } else {
            query_handle
        };

        configured_query
            .set_return_metadata(true)
            .set_return_key_value_tags(true)
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
                return Err("Sampling operation timed out waiting for Steam response".to_string());
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
    let mut tags = HashSet::new();

    for item in items_result.items.into_iter().flatten() {
        if item.file_type == "Community" {
            tags.extend(item.tags.into_iter().filter(|tag| !tag.trim().is_empty()));
        }
    }

    Ok(tags)
}
