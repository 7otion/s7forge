//! Commands that talk to the Steam API. Steam must be running and own `APP_ID`.

mod common;

use common::*;
use std::sync::Mutex;

/// Held by tests that start or inspect a download of `ITEM_ID`, which run in
/// parallel otherwise: a re-download in one makes the other see "downloading".
static ITEM_DOWNLOAD: Mutex<()> = Mutex::new(());

fn lock_item_download() -> std::sync::MutexGuard<'static, ()> {
    ITEM_DOWNLOAD
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[test]
fn subscription_times_cover_the_fixture_item() {
    require_steam();
    let value = s7forge(&["subscription-times", "--app-id", APP_ID]).json();
    let times = as_array(&value);
    let item = times
        .iter()
        .find(|time| time["item_id"].to_string() == ITEM_ID)
        .expect("the fixture item is subscribed");
    assert!(item["time_subscribed"].as_u64().unwrap() > 0);
}

#[test]
fn workshop_items_returns_requested_item() {
    require_steam();
    let value = s7forge(&["workshop-items", "--app-id", APP_ID, "--item-ids", ITEM_ID]).json();
    let items = as_array(&value);
    assert_eq!(items.len(), 1);
    assert_workshop_item(&items[0]);
    assert_eq!(items[0]["published_file_id"].to_string(), ITEM_ID);
}

#[test]
fn workshop_items_keeps_input_order() {
    require_steam();
    let first = s7forge(&["workshop-items", "--app-id", APP_ID, "--item-ids", ITEM_ID]).json();
    let collection = s7forge(&[
        "collection-items",
        "--app-id",
        APP_ID,
        "--item-id",
        COLLECTION_ID,
    ])
    .json();
    let other = collection["items"][0]["published_file_id"].to_string();

    let ids = format!("{other},{ITEM_ID}");
    let value = s7forge(&["workshop-items", "--app-id", APP_ID, "--item-ids", &ids]).json();
    let items = as_array(&value);
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["published_file_id"].to_string(), other);
    assert_eq!(items[1]["published_file_id"], first[0]["published_file_id"]);
}

#[test]
fn subscribed_items_report_local_install_state() {
    require_steam();
    let _lock = lock_item_download();
    let value = s7forge(&["subscribed-items", "--app-id", APP_ID]).json();
    let item = as_array(&value)
        .iter()
        .find(|item| item["published_file_id"].as_u64() == ITEM_ID.parse().ok())
        .expect("fixture item must be subscribed");

    assert_eq!(item["state"]["subscribed"], true);
    assert_eq!(item["state"]["installed"], true);
    assert_eq!(item["state"]["downloading"], false);

    let folder = item["install"]["folder"].as_str().unwrap();
    assert!(folder.ends_with(ITEM_ID), "unexpected folder: {folder}");
    assert!(std::path::Path::new(folder).is_dir());
    assert!(item["install"]["size_on_disk"].as_u64().unwrap() > 0);
    // The installed copy is never newer than the published one.
    assert!(item["install"]["time_updated"].as_u64() <= item["time_updated"].as_u64());
}

#[test]
fn search_results_include_local_state() {
    // assert_workshop_item checks `state` and `install` on every result.
    search(&["--sort-by", "recent"]);
}

#[test]
fn subscribed_items_report_subscription_time() {
    require_steam();
    let value = s7forge(&["subscribed-items", "--app-id", APP_ID]).json();
    let item = as_array(&value)
        .iter()
        .find(|item| item["published_file_id"].as_u64() == ITEM_ID.parse().ok())
        .expect("fixture item must be subscribed");

    let subscribed = item["time_subscribed"]
        .as_u64()
        .expect("time_subscribed must be a number");
    // Nobody can subscribe to an item before it exists.
    assert!(subscribed >= item["time_created"].as_u64().unwrap());
}

#[test]
fn subscribed_items_contains_fixture_item() {
    require_steam();
    let value = s7forge(&["subscribed-items", "--app-id", APP_ID]).json();
    let items = as_array(&value);
    for item in items {
        assert_workshop_item(item);
    }
    assert!(
        items
            .iter()
            .any(|item| item["published_file_id"].as_u64() == ITEM_ID.parse().ok()),
        "test account must be subscribed to {ITEM_ID}"
    );
}

#[test]
fn collection_items_returns_details_and_children() {
    require_steam();
    let value = s7forge(&[
        "collection-items",
        "--app-id",
        APP_ID,
        "--item-id",
        COLLECTION_ID,
    ])
    .json();

    let details = &value["details"];
    assert_eq!(details["id"].to_string(), COLLECTION_ID);
    assert!(!details["title"].as_str().unwrap().is_empty());
    assert_unix_seconds(&details["time_created"]);
    assert_unix_seconds(&details["time_updated"]);

    let items = as_array(&value["items"]);
    assert!(!items.is_empty());
    for item in items {
        assert_workshop_item(item);
    }
}

#[test]
fn collection_items_rejects_non_collection() {
    require_steam();
    let stderr = s7forge(&["collection-items", "--app-id", APP_ID, "--item-id", ITEM_ID]).error();
    assert!(
        stderr.contains("not a collection"),
        "unexpected error: {stderr}"
    );
}

#[test]
fn check_item_download_reports_installed_item_as_complete() {
    require_steam();
    let _lock = lock_item_download();
    let value = s7forge(&[
        "check-item-download",
        "--app-id",
        APP_ID,
        "--item-id",
        ITEM_ID,
    ])
    .json();
    assert_eq!(value["download_complete"], true);
    assert_eq!(value["is_downloading"], false);
    assert_eq!(value["progress_percentage"].as_f64().unwrap(), 100.0);
}

#[test]
fn download_progress_reports_installed_item() {
    require_steam();
    let _lock = lock_item_download();
    let value = s7forge(&[
        "download-progress",
        "--app-id",
        APP_ID,
        "--item-ids",
        ITEM_ID,
    ])
    .json();
    let items = as_array(&value);
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["item_id"].to_string(), ITEM_ID);
    assert_eq!(items[0]["state"]["installed"], true);
    assert_eq!(items[0]["state"]["downloading"], false);
    assert!(items[0]["installed_time_updated"].is_u64());
}

#[test]
fn download_items_reports_installed_item() {
    require_steam();
    let _lock = lock_item_download();
    let value = s7forge(&["download-items", "--app-id", APP_ID, "--item-ids", ITEM_ID]).json();
    let items = as_array(&value);
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["item_id"].to_string(), ITEM_ID);
    let outcome = items[0]["outcome"].as_str().unwrap();
    assert!(
        outcome == "completed" || outcome == "pending",
        "unexpected outcome {outcome}"
    );
}

#[test]
fn download_items_refuses_unsubscribed_item() {
    require_steam();
    let value = s7forge(&["download-items", "--app-id", APP_ID, "--item-ids", "1"]).json();
    assert_eq!(as_array(&value)[0]["outcome"], "refused");
}

fn search(args: &[&str]) -> Vec<serde_json::Value> {
    require_steam();
    let mut full = vec!["search-workshop", "--app-id", APP_ID];
    full.extend_from_slice(args);
    let value = s7forge(&full).json();
    let items = as_array(&value).clone();
    for item in &items {
        assert_workshop_item(item);
    }
    items
}

#[test]
fn search_workshop_supports_every_sort() {
    for sort in [
        "relevance",
        "recent",
        "recently-updated",
        "most-subscribed",
        "popular",
    ] {
        let items = search(&["--sort-by", sort]);
        assert!(!items.is_empty(), "sort '{sort}' returned no items");
    }
}

#[test]
fn search_workshop_popular_supports_every_period() {
    for period in [
        "today",
        "one-week",
        "three-months",
        "six-months",
        "one-year",
    ] {
        search(&["--sort-by", "popular", "--period", period]);
    }
}

#[test]
fn search_workshop_pages_differ() {
    let page1 = search(&["--sort-by", "recent", "--page", "1"]);
    let page2 = search(&["--sort-by", "recent", "--page", "2"]);
    assert!(!page1.is_empty() && !page2.is_empty());
    assert_ne!(page1[0]["published_file_id"], page2[0]["published_file_id"]);
}

#[test]
fn search_workshop_with_query_matches_titles() {
    let items = search(&["--query", "unit", "--sort-by", "relevance"]);
    assert!(!items.is_empty());
}

#[test]
fn search_workshop_with_tag_only_returns_tagged_items() {
    let items = search(&["--tags", "UI", "--sort-by", "recent"]);
    assert!(!items.is_empty());
    for item in &items {
        let tags = as_array(&item["tags"]);
        assert!(
            tags.iter()
                .any(|t| t.as_str().is_some_and(|t| t.eq_ignore_ascii_case("ui"))),
            "item lacks UI tag: {tags:?}"
        );
    }
}

#[test]
fn discover_tags_returns_sorted_unique_tags() {
    require_steam();
    let value = s7forge(&["discover-tags", "--app-id", APP_ID]).json();
    let tags: Vec<&str> = as_array(&value)
        .iter()
        .map(|t| t.as_str().unwrap())
        .collect();
    assert!(!tags.is_empty());
    let mut sorted = tags.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(tags, sorted);
}

#[test]
fn combined_returns_one_entry_per_subcommand() {
    require_steam();
    let value = s7forge(&[
        "--app-id",
        APP_ID,
        "combined",
        "--subscribed-items",
        "--workshop-path",
        "--workshop-items",
        "--item-ids",
        ITEM_ID,
    ])
    .json();
    assert!(value["subscribed-items"].is_array());
    assert!(value["workshop-path"].is_string());
    assert_eq!(
        value["workshop-items-2"][0]["published_file_id"].to_string(),
        ITEM_ID
    );
}

/// Modifies the account's subscriptions; restores the original state.
#[test]
#[ignore]
fn subscribe_and_unsubscribe_round_trip() {
    require_steam();
    let subscribed = || {
        as_array(&s7forge(&["subscribed-items", "--app-id", APP_ID]).json())
            .iter()
            .any(|item| item["published_file_id"].as_u64() == ITEM_ID.parse().ok())
    };
    let initially = subscribed();
    let (first, second) = if initially {
        ("unsubscribe", "subscribe")
    } else {
        ("subscribe", "unsubscribe")
    };

    let value = s7forge(&[first, "--app-id", APP_ID, "--item-ids", ITEM_ID]).json();
    assert_eq!(value[0]["success"], true);
    assert_ne!(subscribed(), initially);

    let value = s7forge(&[second, "--app-id", APP_ID, "--item-ids", ITEM_ID]).json();
    assert_eq!(value[0]["success"], true);
    assert_eq!(subscribed(), initially);
}

#[test]
fn collection_items_for_missing_item_fails_cleanly() {
    require_steam();
    for missing in ["1", "9999999999"] {
        let stderr =
            s7forge(&["collection-items", "--app-id", APP_ID, "--item-id", missing]).error();
        assert!(
            stderr.contains("not found"),
            "unexpected error for {missing}: {stderr}"
        );
    }
}

#[test]
fn workshop_items_returns_full_description() {
    require_steam();
    let value = s7forge(&[
        "workshop-items",
        "--app-id",
        APP_ID,
        "--item-ids",
        LONG_DESCRIPTION_ITEM_ID,
    ])
    .json();
    let description = value[0]["description"].as_str().unwrap();
    assert!(
        description.chars().count() > 400,
        "description was truncated to {} chars",
        description.chars().count()
    );
}

/// Subscribing to an item the account is already subscribed to is a no-op,
/// so this test does not change the account.
#[test]
fn subscribe_reports_each_item_even_when_one_fails() {
    require_steam();
    let ids = format!("1,{ITEM_ID}");
    let value = s7forge(&["subscribe", "--app-id", APP_ID, "--item-ids", &ids]).json();
    let results = as_array(&value);
    assert_eq!(
        results.len(),
        2,
        "every requested item needs a result: {value}"
    );

    assert_eq!(results[0]["item_id"], 1);
    assert_eq!(results[0]["success"], false);
    assert!(
        results[0]["error"]
            .as_str()
            .unwrap()
            .contains("FileNotFound")
    );

    assert_eq!(results[1]["item_id"].to_string(), ITEM_ID);
    assert_eq!(results[1]["success"], true);
    assert!(results[1].get("error").is_none());
}

/// Re-downloads the ~13 KB fixture item; Steam finishes in about a second.
#[test]
fn download_workshop_item_completes_for_subscribed_item() {
    require_steam();
    let _lock = lock_item_download();
    let value = s7forge(&[
        "download-workshop-item",
        "--app-id",
        APP_ID,
        "--item-id",
        ITEM_ID,
    ])
    .json();
    assert!(value.is_string());

    let state = s7forge(&[
        "check-item-download",
        "--app-id",
        APP_ID,
        "--item-id",
        ITEM_ID,
    ])
    .json();
    assert_eq!(state["download_complete"], true);
}

#[test]
fn download_workshop_item_rejects_unsubscribed_item() {
    require_steam();
    let stderr = s7forge(&[
        "download-workshop-item",
        "--app-id",
        APP_ID,
        "--item-id",
        "1",
    ])
    .error();
    assert!(
        stderr.contains("not subscribed"),
        "unexpected error: {stderr}"
    );
}

/// Tags are passed through exactly as Steam stores them (WH3 uses lowercase
/// values plus a "mod" type tag); consumers decide what to show.
#[test]
fn workshop_items_returns_raw_steam_tags() {
    require_steam();
    let value = s7forge(&["workshop-items", "--app-id", APP_ID, "--item-ids", ITEM_ID]).json();
    assert_eq!(value[0]["tags"], serde_json::json!(["mod", "battle"]));
}
