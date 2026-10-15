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
