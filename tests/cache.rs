//! Cache location and cache failure handling.

mod common;

use common::*;
use std::fs;

/// Installed apps live in read-only folders such as Program Files, so the cache
/// must not live next to the executable, and a broken cache must never fail a command.
#[test]
fn unwritable_executable_directory_does_not_break_commands() {
    require_steam();
    let dir = TempDir::new("exe-dir");
    let exe = dir.path().join("s7forge.exe");
    fs::copy(env!("CARGO_BIN_EXE_s7forge"), &exe).unwrap();
    // A file where the cache directory would be created makes that directory impossible.
    fs::write(dir.path().join("cache"), b"").unwrap();

    let mut cmd = std::process::Command::new(&exe);
    cmd.args(["workshop-items", "--app-id", APP_ID, "--item-ids", ITEM_ID]);
    cmd.env_remove(CACHE_DIR_ENV);
    let value = run(cmd).json();
    assert_eq!(as_array(&value).len(), 1);
}

#[test]
fn unusable_cache_dir_does_not_break_commands() {
    require_steam();
    let dir = TempDir::new("blocked-cache");
    let blocker = dir.path().join("file");
    fs::write(&blocker, b"").unwrap();

    let mut cmd = command();
    cmd.args(["workshop-items", "--app-id", APP_ID, "--item-ids", ITEM_ID]);
    cmd.env(CACHE_DIR_ENV, blocker.join("cache"));
    let value = run(cmd).json();
    assert_eq!(as_array(&value).len(), 1);
}

#[test]
fn cache_dir_env_var_is_used() {
    require_steam();
    let dir = TempDir::new("cache-env");
    let cache = dir.path().join("nested").join("cache");

    let mut cmd = command();
    cmd.args(["workshop-items", "--app-id", APP_ID, "--item-ids", ITEM_ID]);
    cmd.env(CACHE_DIR_ENV, &cache);
    run(cmd).json();

    assert!(
        fs::read_dir(&cache).map(|d| d.count() > 0).unwrap_or(false),
        "expected cache files in {}",
        cache.display()
    );

    let mut cmd = command();
    cmd.arg("clear-cache").env(CACHE_DIR_ENV, &cache);
    let value = run(cmd).json();
    assert!(value["files_cleared"].as_u64().unwrap() > 0);
    assert_eq!(fs::read_dir(&cache).unwrap().count(), 0);
}

#[test]
fn cached_workshop_items_match_fresh_ones() {
    require_steam();
    let cache = TempDir::new("reuse");
    let ids = format!("{ITEM_ID},{LONG_DESCRIPTION_ITEM_ID},1");
    let args = [
        "workshop-items",
        "--app-id",
        APP_ID,
        "--item-ids",
        ids.as_str(),
    ];

    let fresh = s7forge_with_cache(cache.path(), &args).json();
    let cached = s7forge_with_cache(cache.path(), &args).json();
    assert_eq!(
        as_array(&fresh).len(),
        2,
        "id 1 does not exist and must be skipped"
    );
    assert_eq!(fresh, cached);
}
