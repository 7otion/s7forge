use futures_util::FutureExt;
use rustc_hash::{FxHashMap, FxHashSet};
use steamworks::SteamId;

use crate::core::steam_manager;
use crate::utils::get_cache_dir::cache_file;
use crate::utils::timed_cache::{TimedCache, now_secs};

/// The name Steam reports until it has the persona data for a user.
pub const UNKNOWN_NAME: &str = "[unknown]";
/// Names are re-fetched after this long so renamed creators eventually update.
const CREATOR_NAME_TTL_SECS: u64 = 7 * 24 * 60 * 60;

type CreatorNameCache = TimedCache<u64, String>;

pub async fn fetch_creator_names(
    creator_ids: Vec<SteamId>,
    steam_game_id: u32,
) -> Result<FxHashMap<u64, String>, String> {
    let cache_path = cache_file("creator_names_cache.bin");
    let mut cache = CreatorNameCache::load(&cache_path);
    let now = now_secs();

    let mut names = FxHashMap::default();
    let mut ids_to_fetch = FxHashSet::default();
    for id in creator_ids {
        match cache.get(&id.raw(), now, CREATOR_NAME_TTL_SECS) {
            Some(name) => {
                names.insert(id.raw(), name.clone());
            }
            None => {
                ids_to_fetch.insert(id);
            }
        }
    }

    if ids_to_fetch.is_empty() {
        return Ok(names);
    }

    let steam_client = steam_manager::initialize_client(steam_game_id).await?;
    let (creator_tx, mut creator_rx) = tokio::sync::mpsc::channel(32);

    let creator_task = tokio::task::spawn_blocking(move || {
        let friends = steam_client.friends();
        let mut unknown_creators = FxHashSet::default();

        for &creator_id in &ids_to_fetch {
            if friends.get_friend(creator_id).name() == UNKNOWN_NAME {
                unknown_creators.insert(creator_id);
                let _ = friends.request_user_information(creator_id, true);
            }
        }

        if !unknown_creators.is_empty() {
            let start_time = std::time::Instant::now();
            let timeout = std::time::Duration::from_secs(2);

            while !unknown_creators.is_empty() && start_time.elapsed() < timeout {
                let _ = creator_tx.blocking_send(());
                std::thread::sleep(std::time::Duration::from_millis(50));
                unknown_creators.retain(|&id| friends.get_friend(id).name() == UNKNOWN_NAME);
            }
        }

        ids_to_fetch
            .into_iter()
            .map(|id| (id.raw(), friends.get_friend(id).name()))
            .collect::<FxHashMap<u64, String>>()
    });

    let mut creator_result = None;
    let mut fused_creator_task = creator_task.fuse();
    while creator_result.is_none() {
        tokio::select! {
            Some(_) = creator_rx.recv() => {
                steam_manager::run_callbacks(steam_game_id)?;
            }
            task_result = &mut fused_creator_task => {
                creator_result = Some(
                    task_result.map_err(|e| format!("Creator task error: {:?}", e))?
                );
                break;
            }
        }
    }

    let fetched_names = creator_result.unwrap();
    store_resolved_names(&mut cache, &fetched_names, now);
    cache.prune(now, CREATOR_NAME_TTL_SECS);
    cache.save(&cache_path);

    names.extend(fetched_names);
    Ok(names)
}

/// Caches only names Steam actually resolved; an unresolved creator is asked about again next run.
fn store_resolved_names(cache: &mut CreatorNameCache, names: &FxHashMap<u64, String>, now: u64) {
    for (&id, name) in names {
        if name != UNKNOWN_NAME {
            cache.insert(id, name.clone(), now);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unresolved_names_are_not_cached() {
        let mut cache = CreatorNameCache::default();
        let mut names = FxHashMap::default();
        names.insert(1, UNKNOWN_NAME.to_string());
        names.insert(2, "Creator".to_string());

        store_resolved_names(&mut cache, &names, 0);

        assert_eq!(cache.get(&1, 0, CREATOR_NAME_TTL_SECS), None);
        assert_eq!(
            cache.get(&2, 0, CREATOR_NAME_TTL_SECS).map(String::as_str),
            Some("Creator")
        );
    }

    #[test]
    fn cached_names_expire() {
        let mut cache = CreatorNameCache::default();
        let mut names = FxHashMap::default();
        names.insert(2, "Creator".to_string());
        store_resolved_names(&mut cache, &names, 0);
        assert_eq!(
            cache.get(&2, CREATOR_NAME_TTL_SECS, CREATOR_NAME_TTL_SECS),
            None
        );
    }
}
