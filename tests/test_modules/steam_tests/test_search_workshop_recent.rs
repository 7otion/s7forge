use crate::test_modules::utils::{
    TestConfig, assert_valid_json, run_search_workshop_command, steam_test_or_skip,
};

#[test]
fn test_search_workshop_recent() {
    steam_test_or_skip(|| {
        let config = TestConfig::load();
        let output = run_search_workshop_command(&[
            "search-workshop",
            "--app-id",
            &config.app_id.to_string(),
            "--sort-by",
            "recent",
            "--page",
            "1",
        ]);

        assert!(
            output.status.success(),
            "Recent search should succeed but failed with: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        let value = assert_valid_json(&stdout);
        assert!(value.is_array(), "Expected JSON array, got: {}", stdout);

        let items_array = value.as_array().unwrap();
        println!("✓ Found {} recent items", items_array.len());

        if !items_array.is_empty() {
            let first_item = &items_array[0];
            assert!(
                first_item.is_object(),
                "Expected recent item to be an object"
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

            let time_created = first_item
                .get("time_created")
                .expect("Missing 'time_created' field");
            assert!(
                time_created.is_number(),
                "Expected 'time_created' to be a number"
            );
            assert!(
                time_created.as_u64().unwrap() > 0,
                "Expected valid timestamp"
            );

            let time_updated = first_item
                .get("time_updated")
                .expect("Missing 'time_updated' field");
            assert!(
                time_updated.is_number(),
                "Expected 'time_updated' to be a number"
            );

            let creator_name = first_item
                .get("creator_name")
                .expect("Missing 'creator_name' field");
            assert!(
                creator_name.is_string(),
                "Expected 'creator_name' to be a string"
            );

            println!(
                "✓ Recent search validation passed: '{}' by {}",
                title.as_str().unwrap(),
                creator_name.as_str().unwrap()
            );
        }
    });
}
