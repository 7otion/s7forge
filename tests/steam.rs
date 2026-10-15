//! Commands that talk to the Steam API. Steam must be running and own `APP_ID`.

mod common;

use common::*;

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
            .any(|item| item["published_file_id"].to_string() == ITEM_ID),
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
    assert!(details["time_created"].as_u64().unwrap() > 0);

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
        let tags = item["tags"].as_str().unwrap();
        assert!(
            tags.split(", ").any(|t| t == "UI"),
            "item lacks UI tag: {tags}"
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
            .any(|item| item["published_file_id"].to_string() == ITEM_ID)
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
