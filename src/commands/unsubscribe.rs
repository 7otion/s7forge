use futures_util::FutureExt;
use serde::Serialize;
use steamworks::PublishedFileId;
use tokio::sync::mpsc;

use crate::core::steam_manager;

#[derive(Debug, Serialize)]
pub struct UnsubscribeResult {
    pub item_id: u64,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub async fn unsubscribe(
    steam_game_id: u32,
    item_ids: Vec<u64>,
) -> Result<Vec<UnsubscribeResult>, String> {
    let steam_client = steam_manager::initialize_client(steam_game_id).await?;
    let mut results = Vec::new();

    // Every item gets a result; one failure must not hide what happened to the others.
    for item_id in item_ids {
        let error = unsubscribe_single_item(&steam_client, steam_game_id, item_id)
            .await
            .err();
        results.push(UnsubscribeResult {
            item_id,
            success: error.is_none(),
            error,
        });
    }

    Ok(results)
}

async fn unsubscribe_single_item(
    steam_client: &steamworks::Client,
    steam_game_id: u32,
    item_id: u64,
) -> Result<(), String> {
    let (tx, mut rx) = mpsc::channel(32);

    let steam_client_clone = steam_client.clone();
    let unsub_task = tokio::task::spawn_blocking(move || {
        let ugc = steam_client_clone.ugc();
        let (tx_inner, rx_inner) = std::sync::mpsc::channel();

        ugc.unsubscribe_item(PublishedFileId(item_id), move |result| {
            let _ = tx_inner.send(result);
        });

        let start_time = std::time::Instant::now();
        let timeout_duration = std::time::Duration::from_secs(30);

        loop {
            let _ = tx.blocking_send(());
            if let Ok(result) = rx_inner.try_recv() {
                return result.map_err(|e| format!("Steam API error: {:?}", e));
            }

            if start_time.elapsed() > timeout_duration {
                return Err("Operation timed out waiting for Steam response".to_string());
            }

            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    });

    let mut result = None;
    let mut unsub_task = unsub_task.fuse();

    while result.is_none() {
        tokio::select! {
            Some(_) = rx.recv() => {
                steam_manager::run_callbacks(steam_game_id)?;
            }
            task_result = &mut unsub_task => {
                result = Some(task_result.map_err(|e| format!("Task join error: {:?}", e))?);
                break;
            }
        }
    }

    result.unwrap()
}
