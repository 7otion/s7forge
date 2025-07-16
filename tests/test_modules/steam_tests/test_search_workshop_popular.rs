use crate::test_modules::utils::{
    TestConfig, assert_valid_json, run_search_workshop_command, steam_test_or_skip,
};

#[test]
fn test_search_workshop_popular() {
    steam_test_or_skip(|| {
        let config = TestConfig::load();
        let output = run_search_workshop_command(&[
            "search-workshop",
            "--app-id",
            &config.app_id.to_string(),
            "--sort-by",
            "popular",
            "--period",
            "one-week",
            "--page",
            "1",
        ]);
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let value = assert_valid_json(&stdout);
            assert!(value.is_array(), "Expected JSON array, got: {}", stdout);

            let items_array = value.as_array().unwrap();
            println!("✓ Found {} popular items (one-week)", items_array.len());

            if !items_array.is_empty() {
                let first_item = &items_array[0];
                assert!(
                    first_item.is_object(),
                    "Expected popular item to be an object"
                );

                let published_file_id = first_item
                    .get("published_file_id")
                    .expect("Missing 'published_file_id' field");
                assert!(
                    published_file_id.is_number(),
                    "Expected 'published_file_id' to be a number"
                );

                let title = first_item.get("title").expect("Missing 'title' field");
                assert!(title.is_string(), "Expected 'title' to be a string");

                let num_upvotes = first_item
                    .get("num_upvotes")
                    .expect("Missing 'num_upvotes' field");
                assert!(
                    num_upvotes.is_number(),
                    "Expected 'num_upvotes' to be a number"
                );

                let creator_name = first_item
                    .get("creator_name")
                    .expect("Missing 'creator_name' field");
                assert!(
                    creator_name.is_string(),
                    "Expected 'creator_name' to be a string"
                );

                println!(
                    "✓ Popular search validation passed: '{}' ({} upvotes) by {}",
                    title.as_str().unwrap(),
                    num_upvotes.as_u64().unwrap(),
                    creator_name.as_str().unwrap()
                );
            }
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("Search workshop popular test failed: {}", stderr);
        }
    });
}
