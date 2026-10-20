use std::sync::mpsc;
use std::time::{Duration, Instant};

use rustc_hash::FxHashMap;
use serde::Serialize;
use steamworks::{DownloadItemResult, ItemState, PublishedFileId, SteamError};

use crate::core::steam_manager;

const SETTLE_WINDOW: Duration = Duration::from_millis(1500);
const POLL_INTERVAL: Duration = Duration::from_millis(50);

#[derive(Debug, Serialize)]
pub struct DownloadRequest {
    pub item_id: u64,
    pub outcome: Outcome,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Pending,
    Completed,
    Refused,
    Failed,
}

pub async fn download_items(
    steam_game_id: u32,
    item_ids: Vec<u64>,
) -> Result<Vec<DownloadRequest>, String> {
    let steam_client = steam_manager::initialize_client(steam_game_id).await?;
    let ugc = steam_client.ugc();

    let (tx, rx) = mpsc::channel();
    let _callback = steam_client.register_callback(move |result: DownloadItemResult| {
        let _ = tx.send((result.published_file_id.0, result.error));
    });

    let mut requests = Vec::with_capacity(item_ids.len());
    for item_id in &item_ids {
        let item = PublishedFileId(*item_id);
        let outcome = if !ugc.item_state(item).contains(ItemState::SUBSCRIBED) {
            Outcome::Refused
        } else if ugc.download_item(item, false) {
            Outcome::Pending
        } else {
            Outcome::Refused
        };
        requests.push(DownloadRequest {
            item_id: *item_id,
            outcome,
            error: (outcome == Outcome::Refused)
                .then(|| "Steam refused to download the item".to_string()),
        });
    }

    let mut results: FxHashMap<u64, Option<SteamError>> = FxHashMap::default();
    let pending = requests
        .iter()
        .filter(|request| request.outcome == Outcome::Pending)
        .count();
    let start = Instant::now();
    while results.len() < pending && start.elapsed() < SETTLE_WINDOW {
        steam_manager::run_callbacks(steam_game_id)?;
        while let Ok((item_id, error)) = rx.try_recv() {
            results.insert(item_id, error);
        }
        std::thread::sleep(POLL_INTERVAL);
    }

    for request in &mut requests {
        if let Some(error) = results.remove(&request.item_id) {
            request.outcome = match &error {
                None => Outcome::Completed,
                Some(_) => Outcome::Failed,
            };
            request.error = error.map(|error| format!("Download failed: {}", error));
        }
    }
    Ok(requests)
}
