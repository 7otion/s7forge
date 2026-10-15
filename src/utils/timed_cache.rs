use bincode::{Decode, Encode};
use rustc_hash::FxHashMap;
use std::fs;
use std::hash::Hash;
use std::path::Path;

use crate::utils::get_cache_dir::write_cache_file;

pub fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// A key-value cache where every entry remembers when it was stored, so each
/// entry expires on its own schedule.
#[derive(Debug, Encode, Decode)]
pub struct TimedCache<K: Eq + Hash, V> {
    entries: FxHashMap<K, Entry<V>>,
}

#[derive(Debug, Encode, Decode)]
struct Entry<V> {
    value: V,
    stored_at: u64,
}

impl<K: Eq + Hash, V> Default for TimedCache<K, V> {
    fn default() -> Self {
        Self {
            entries: FxHashMap::default(),
        }
    }
}

impl<K, V> TimedCache<K, V>
where
    K: Eq + Hash + Encode + Decode<()>,
    V: Encode + Decode<()>,
{
    pub fn load(path: &Path) -> Self {
        fs::read(path)
            .ok()
            .and_then(|bytes| {
                bincode::decode_from_slice(&bytes, bincode::config::standard())
                    .ok()
                    .map(|(cache, _)| cache)
            })
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) {
        if let Ok(bytes) = bincode::encode_to_vec(self, bincode::config::standard()) {
            write_cache_file(path, &bytes);
        }
    }

    /// Returns the value if it was stored less than `ttl_secs` ago.
    pub fn get(&self, key: &K, now: u64, ttl_secs: u64) -> Option<&V> {
        self.entries
            .get(key)
            .filter(|entry| now.saturating_sub(entry.stored_at) < ttl_secs)
            .map(|entry| &entry.value)
    }

    pub fn insert(&mut self, key: K, value: V, now: u64) {
        self.entries.insert(
            key,
            Entry {
                value,
                stored_at: now,
            },
        );
    }

    /// Drops entries older than `ttl_secs` so the file does not grow forever.
    pub fn prune(&mut self, now: u64, ttl_secs: u64) {
        self.entries
            .retain(|_, entry| now.saturating_sub(entry.stored_at) < ttl_secs);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOUR: u64 = 60 * 60;

    #[test]
    fn fresh_entry_is_returned() {
        let mut cache = TimedCache::<u64, String>::default();
        cache.insert(1, "a".into(), 1000);
        assert_eq!(
            cache.get(&1, 1000 + HOUR, 24 * HOUR).map(String::as_str),
            Some("a")
        );
    }

    #[test]
    fn entry_expires_even_when_other_entries_are_written_later() {
        let mut cache = TimedCache::<u64, String>::default();
        cache.insert(1, "old".into(), 0);
        cache.insert(2, "new".into(), 23 * HOUR);
        assert_eq!(
            cache.get(&1, 25 * HOUR, 24 * HOUR),
            None,
            "entry 1 is 25 h old"
        );
        assert!(cache.get(&2, 25 * HOUR, 24 * HOUR).is_some());
    }

    #[test]
    fn each_kind_of_entry_can_use_its_own_ttl() {
        let mut cache = TimedCache::<u64, Option<String>>::default();
        cache.insert(1, None, 0);
        // A "not found" answer is trusted for 1 h, a real item for 24 h.
        assert!(cache.get(&1, 2 * HOUR, HOUR).is_none());
    }

    #[test]
    fn prune_removes_only_expired_entries() {
        let mut cache = TimedCache::<u64, String>::default();
        cache.insert(1, "old".into(), 0);
        cache.insert(2, "new".into(), 23 * HOUR);
        cache.prune(25 * HOUR, 24 * HOUR);
        assert_eq!(cache.entries.len(), 1);
        assert!(cache.entries.contains_key(&2));
    }
}
