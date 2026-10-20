//! Commands that never touch the Steam API (CLI parsing, cache, local paths).

mod common;

use common::*;
use std::path::Path;

#[test]
fn help_lists_every_command() {
    let stdout = s7forge(&["--help"]).stdout();
    for command in [
        "search-workshop",
        "discover-tags",
        "workshop-items",
        "collection-items",
        "subscribed-items",
        "workshop-state",
        "subscription-times",
        "check-item-download",
        "subscribe",
        "unsubscribe",
        "download-workshop-item",
        "clear-cache",
        "workshop-path",
        "steam-library-paths",
        "app-installation-path",
        "--help",
        "--version",
    ] {
        assert!(stdout.contains(command), "help is missing '{command}'");
    }
}

#[test]
fn version_prints_package_version() {
    let run = s7forge(&["--version"]);
    assert_eq!(run.code(), Some(0));
    assert!(run.stdout().contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn missing_command_fails() {
    assert!(s7forge(&[]).stderr().contains("Missing command"));
    assert_eq!(s7forge(&[]).code(), Some(1));
}

#[test]
fn unknown_command_fails() {
    let run = s7forge(&["no-such-command"]);
    assert_eq!(run.code(), Some(1));
    assert!(run.stderr().contains("Unknown command"));
}

#[test]
fn clear_cache_reports_cleared_files() {
    let value = s7forge(&["clear-cache"]).json();
    assert_eq!(value["success"], true);
    assert!(!value["message"].as_str().unwrap().is_empty());
    assert_eq!(
        value["files_cleared"].as_u64().unwrap() as usize,
        as_array(&value["files"]).len()
    );
}

#[test]
fn steam_library_paths_are_existing_directories() {
    let value = s7forge(&["steam-library-paths"]).json();
    let paths = as_array(&value);
    assert!(!paths.is_empty());
    for path in paths {
        let path = path.as_str().unwrap();
        assert!(
            Path::new(path).is_dir(),
            "library path does not exist: {path}"
        );
    }
}

#[test]
fn workshop_path_points_to_app_content_folder() {
    let value = s7forge(&["workshop-path", "--app-id", APP_ID]).json();
    let path = value.as_str().unwrap();
    assert!(
        Path::new(path).is_dir(),
        "workshop path does not exist: {path}"
    );
    assert!(path.ends_with(APP_ID));
}

#[test]
fn workshop_state_reads_the_manifest_without_steam() {
    let value = s7forge(&["workshop-state", "--app-id", APP_ID]).json();
    assert_eq!(value["route"], "files");
    let items = as_array(&value["items"]);
    assert!(!items.is_empty(), "no subscriptions in the manifest");
    let item = items
        .iter()
        .find(|item| item["item_id"].to_string() == ITEM_ID)
        .expect("the fixture item is subscribed");
    assert!(item["installed"].is_boolean());
    assert!(item["folder"].is_string() || item["folder"].is_null());
    assert!(item["needs_update"].is_boolean());
    assert!(item["download_pending"].is_boolean());
}

#[test]
fn app_installation_path_points_to_installed_game() {
    let value = s7forge(&["app-installation-path", "--app-id", APP_ID]).json();
    let path = value.as_str().unwrap();
    assert!(
        Path::new(path).is_dir(),
        "installation path does not exist: {path}"
    );
}

#[test]
fn app_installation_path_for_unknown_app_fails() {
    let stderr = s7forge(&["app-installation-path", "--app-id", "1"]).error();
    assert!(
        stderr.contains("not installed"),
        "unexpected error: {stderr}"
    );
}

#[test]
fn search_workshop_page_zero_fails() {
    let stderr = s7forge(&["search-workshop", "--app-id", APP_ID, "--page", "0"]).error();
    assert!(stderr.contains("Page number must be at least 1"));
}

#[test]
fn search_workshop_rejects_unknown_sort() {
    let stderr = s7forge(&["search-workshop", "--app-id", APP_ID, "--sort-by", "bogus"]).error();
    assert!(
        stderr.contains("Invalid --sort-by"),
        "unexpected error: {stderr}"
    );
}

#[test]
fn search_workshop_rejects_unknown_period() {
    let stderr = s7forge(&[
        "search-workshop",
        "--app-id",
        APP_ID,
        "--sort-by",
        "popular",
        "--period",
        "bogus",
    ])
    .error();
    assert!(
        stderr.contains("Invalid --period"),
        "unexpected error: {stderr}"
    );
}

#[test]
fn search_workshop_period_requires_popular_sort() {
    let stderr = s7forge(&[
        "search-workshop",
        "--app-id",
        APP_ID,
        "--sort-by",
        "recent",
        "--period",
        "one-week",
    ])
    .error();
    assert!(
        stderr.contains("only applicable"),
        "unexpected error: {stderr}"
    );
}

#[test]
fn combined_search_workshop_rejects_unknown_sort() {
    let stderr = s7forge(&[
        "--app-id",
        APP_ID,
        "combined",
        "--search-workshop",
        "--sort-by",
        "bogus",
    ])
    .error();
    assert!(
        stderr.contains("Invalid --sort-by"),
        "unexpected error: {stderr}"
    );
}

/// Removes a directory this test created inside a Steam library.
struct CreatedDir(std::path::PathBuf);

impl Drop for CreatedDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir(&self.0);
    }
}

/// A game's workshop folder appears when its first mod is downloaded; a
/// "not found" answer from before that must not be cached.
#[test]
fn workshop_path_notices_a_folder_created_after_a_miss() {
    // An app id no real game uses, so the folder is certainly ours.
    const FAKE_APP_ID: &str = "4000000001";
    let cache = TempDir::new("workshop-miss");

    let libraries = s7forge(&["steam-library-paths"]).json();
    let library = as_array(&libraries)[0].as_str().unwrap().to_string();
    let folder = Path::new(&library)
        .join("steamapps")
        .join("workshop")
        .join("content")
        .join(FAKE_APP_ID);
    assert!(
        !folder.exists(),
        "leftover test folder: {}",
        folder.display()
    );

    let args = ["workshop-path", "--app-id", FAKE_APP_ID];
    s7forge_with_cache(cache.path(), &args).error();

    std::fs::create_dir(&folder).unwrap();
    let _cleanup = CreatedDir(folder.clone());

    let value = s7forge_with_cache(cache.path(), &args).json();
    assert_eq!(Path::new(value.as_str().unwrap()), folder);
}

fn combined(args: &[&str]) -> Run {
    let mut full = vec!["--app-id", APP_ID, "combined"];
    full.extend_from_slice(args);
    s7forge(&full)
}

#[test]
fn combined_rejects_arguments_before_the_first_subcommand() {
    let stderr = combined(&["--query", "x", "--workshop-path"]).error();
    assert!(stderr.contains("--query"), "unexpected error: {stderr}");
}

#[test]
fn combined_rejects_arguments_to_subcommands_without_options() {
    let stderr = combined(&["--workshop-path", "--bogus"]).error();
    assert!(stderr.contains("--bogus"), "unexpected error: {stderr}");
}

#[test]
fn combined_rejects_item_ids_without_value() {
    let stderr = combined(&["--workshop-items", "--item-ids"]).error();
    assert!(stderr.contains("--item-ids"), "unexpected error: {stderr}");
}

#[test]
fn item_ids_are_required() {
    for command in ["workshop-items", "subscribe", "unsubscribe"] {
        let stderr = s7forge(&[command, "--app-id", APP_ID]).error();
        assert!(
            stderr.contains("--item-ids"),
            "{command}: unexpected error: {stderr}"
        );
    }
}

/// Callers show these messages to users, so they must be plain text, not Rust debug strings.
#[test]
fn runtime_errors_are_plain_text() {
    let stderr = s7forge(&["app-installation-path", "--app-id", "1"]).error();
    assert_eq!(
        stderr.trim_end(),
        "Error: App 1 is not installed or manifest file not found"
    );
}
