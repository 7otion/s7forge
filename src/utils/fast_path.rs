//! A command with two routes: a fast one that reads Steam's files on disk,
//! and a slow one through the running Steam client. When the fast route
//! fails, the failure is remembered for a day so every call until then goes
//! straight to the slow route instead of failing the same way again.

use std::future::Future;

use serde::Serialize;

use crate::utils::get_cache_dir::cache_file;
use crate::utils::timed_cache::{TimedCache, now_secs};

const FAILURE_TTL_SECS: u64 = 24 * 60 * 60;

/// Route name -> why the fast route failed last time.
type FailureCache = TimedCache<String, String>;

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Route {
    Files,
    Steam,
}

pub enum Fast<T> {
    Answer(Result<T, String>),
    Failed(String),
}

pub async fn with_fallback<T, FastFn, SlowFn, SlowFut>(
    name: &str,
    fast: FastFn,
    slow: SlowFn,
) -> Result<(Route, T), String>
where
    FastFn: FnOnce() -> Fast<T>,
    SlowFn: FnOnce() -> SlowFut,
    SlowFut: Future<Output = Result<T, String>>,
{
    let cache_path = cache_file("fast_path_failures_cache.bin");
    let mut failures = FailureCache::load(&cache_path);
    let now = now_secs();
    let name = name.to_owned();

    if failures.get(&name, now, FAILURE_TTL_SECS).is_none() {
        match fast() {
            Fast::Answer(answer) => return answer.map(|value| (Route::Files, value)),
            Fast::Failed(reason) => {
                failures.insert(name, reason, now);
                failures.prune(now, FAILURE_TTL_SECS);
                failures.save(&cache_path);
            }
        }
    }
    slow().await.map(|value| (Route::Steam, value))
}
