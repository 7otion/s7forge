use std::sync::mpsc;
use std::time::{Duration, Instant};
use steamworks::{DownloadItemResult, ItemState, PublishedFileId, SteamError};

use crate::core::steam_manager;

const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(10 * 60);
const POLL_INTERVAL: Duration = Duration::from_millis(100);

/// Asks Steam to (re)download a subscribed item and waits for Steam's
/// `DownloadItemResult`, which reports success or the exact failure.
pub async fn download_workshop_item(steam_game_id: u32, item_id: u64) -> Result<(), String> {
    let steam_client = steam_manager::initialize_client(steam_game_id).await?;
    let ugc = steam_client.ugc();
    let item = PublishedFileId(item_id);

    if !ugc.item_state(item).contains(ItemState::SUBSCRIBED) {
        return Err("Workshop item is not subscribed".to_string());
    }

    // Registered before DownloadItem so a fast result cannot be missed.
    let (tx, rx) = mpsc::channel();
    let _callback = steam_client.register_callback(move |result: DownloadItemResult| {
        if result.published_file_id == item {
            let _ = tx.send(result.error);
        }
    });

    if !ugc.download_item(item, true) {
        return Err(format!(
            "Steam refused to download workshop item {}",
            item_id
        ));
    }

    let start = Instant::now();
    loop {
        steam_manager::run_callbacks(steam_game_id)?;
        if let Ok(error) = rx.try_recv() {
            return download_outcome(error);
        }
        if start.elapsed() > DOWNLOAD_TIMEOUT {
            return Err(format!(
                "Download timeout after {} minutes",
                DOWNLOAD_TIMEOUT.as_secs() / 60
            ));
        }
        std::thread::sleep(POLL_INTERVAL);
    }
}

fn download_outcome(error: Option<SteamError>) -> Result<(), String> {
    match error {
        None => Ok(()),
        Some(error) => Err(format!("Download failed: {}", error)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steam_error_in_result_is_reported() {
        let outcome = download_outcome(Some(SteamError::Timeout));
        assert!(outcome.unwrap_err().starts_with("Download failed: "));
    }

    #[test]
    fn result_without_error_is_success() {
        assert!(download_outcome(None).is_ok());
    }
}
