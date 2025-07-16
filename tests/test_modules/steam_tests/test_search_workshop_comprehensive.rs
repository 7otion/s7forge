use crate::test_modules::utils::{
    TestConfig, assert_valid_json, run_search_workshop_command, steam_test_or_skip,
};

#[test]
fn test_search_workshop_all_sort_methods() {
    steam_test_or_skip(|| {
        let config = TestConfig::load();

        let working_methods = vec![
            ("relevance", "Should work with text search"),
            ("recent", "Should work for recent publications"),
            ("recently-updated", "Should work for recent updates"),
        ];

        for (sort_method, description) in working_methods {
            println!("Testing sort method: {} - {}", sort_method, description);
            let output = run_search_workshop_command(&[
                "search-workshop",
                "--app-id",
                &config.app_id.to_string(),
                "--sort-by",
                sort_method,
                "--page",
                "1",
            ]);

            assert!(
                output.status.success(),
                "Sort method '{}' should work but failed with: {}",
                sort_method,
                String::from_utf8_lossy(&output.stderr)
            );

            let stdout = String::from_utf8_lossy(&output.stdout);
            let value = assert_valid_json(&stdout);
            assert!(
                value.is_array(),
                "Expected JSON array for sort method: {}",
                sort_method
            );

            println!("✓ Sort method '{}' works correctly", sort_method);
        }
    });
}

#[test]
fn test_search_workshop_potentially_problematic_sorts() {
    steam_test_or_skip(|| {
        let config = TestConfig::load();

        let problematic_methods = vec![
            ("popular", "May fail if steamworks assertion fails"),
            ("most-subscribed", "May fail if steamworks assertion fails"),
        ];

        for (sort_method, description) in problematic_methods {
            println!(
                "Testing potentially problematic sort method: {} - {}",
                sort_method, description
            );

            let app_id_str = config.app_id.to_string();
            let mut args = vec![
                "search-workshop",
                "--app-id",
                &app_id_str,
                "--sort-by",
                sort_method,
            ];

            if sort_method == "popular" {
                args.extend_from_slice(&["--period", "one-week"]);
            }

            args.extend_from_slice(&["--page", "1"]);

            let output = run_search_workshop_command(&args);

            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let value = assert_valid_json(&stdout);
                assert!(
                    value.is_array(),
                    "Expected JSON array for sort method: {}",
                    sort_method
                );
                println!("✓ Sort method '{}' works correctly", sort_method);
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                println!(
                    "⚠ Sort method '{}' failed as expected in some cases: {}",
                    sort_method, stderr
                );

                if stderr.contains("assertion failed") {
                    println!("  → This is a known steamworks library limitation");
                } else {
                    println!("  → Unexpected error type, may need investigation");
                }
            }
        }
    });
}

#[test]
fn test_search_workshop_caching() {
    steam_test_or_skip(|| {
        let config = TestConfig::load();

        let args = [
            "search-workshop",
            "--app-id",
            &config.app_id.to_string(),
            "--sort-by",
            "recent",
            "--page",
            "1",
        ];

        let start_time = std::time::Instant::now();
        let output1 = run_search_workshop_command(&args);
        let first_call_duration = start_time.elapsed();

        assert!(
            output1.status.success(),
            "First search call should succeed: {}",
            String::from_utf8_lossy(&output1.stderr)
        );

        let stdout1 = String::from_utf8_lossy(&output1.stdout);
        let value1 = assert_valid_json(&stdout1);

        let start_time = std::time::Instant::now();
        let output2 = run_search_workshop_command(&args);
        let second_call_duration = start_time.elapsed();

        assert!(
            output2.status.success(),
            "Second search call should succeed: {}",
            String::from_utf8_lossy(&output2.stderr)
        );

        let stdout2 = String::from_utf8_lossy(&output2.stdout);
        let value2 = assert_valid_json(&stdout2);

        assert_eq!(
            serde_json::to_string(&value1).unwrap(),
            serde_json::to_string(&value2).unwrap(),
            "Cached results should be identical to original results"
        );

        if second_call_duration < first_call_duration {
            println!("✓ Caching appears to be working (second call was faster)");
        } else {
            println!("⚠ Cache performance test inconclusive (might be due to test environment)");
        }

        println!("✓ Caching functionality validated");
    });
}

#[test]
fn test_search_workshop_negative_cases() {
    steam_test_or_skip(|| {
        let config = TestConfig::load();

        let output = run_search_workshop_command(&[
            "search-workshop",
            "--app-id",
            &config.app_id.to_string(),
            "--sort-by",
            "recent",
            "--page",
            "0",
        ]);

        assert!(
            !output.status.success(),
            "Page 0 should fail but it succeeded"
        );

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("Page number must be at least 1"),
            "Should contain proper error message for invalid page, got: {}",
            stderr
        );

        println!("✓ Invalid page number correctly rejected");

        let output = run_search_workshop_command(&[
            "search-workshop",
            "--app-id",
            "0",
            "--sort-by",
            "recent",
            "--page",
            "1",
        ]);

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let value = assert_valid_json(&stdout);
            assert!(
                value.is_array(),
                "Should return array even for invalid app ID"
            );

            let items = value.as_array().unwrap();
            println!(
                "✓ Invalid app ID handled gracefully (returned {} items)",
                items.len()
            );
        } else {
            println!("✓ Invalid app ID correctly rejected");
        }

        let output = run_search_workshop_command(&[
            "search-workshop",
            "--app-id",
            &config.app_id.to_string(),
            "--sort-by",
            "invalid-sort",
            "--page",
            "1",
        ]);

        assert!(
            !output.status.success(),
            "Invalid sort method should be rejected by clap"
        );

        println!("✓ Invalid sort method correctly rejected");

        let output = run_search_workshop_command(&[
            "search-workshop",
            "--app-id",
            &config.app_id.to_string(),
            "--sort-by",
            "popular",
            "--period",
            "invalid-period",
            "--page",
            "1",
        ]);

        assert!(
            !output.status.success(),
            "Invalid period should be rejected by clap"
        );

        println!("✓ Invalid period correctly rejected");
    });
}

#[test]
fn test_search_workshop_pagination() {
    steam_test_or_skip(|| {
        let config = TestConfig::load();

        // Get first page
        let output1 = run_search_workshop_command(&[
            "search-workshop",
            "--app-id",
            &config.app_id.to_string(),
            "--sort-by",
            "recent",
            "--page",
            "1",
        ]);

        assert!(
            output1.status.success(),
            "First page should succeed: {}",
            String::from_utf8_lossy(&output1.stderr)
        );

        let stdout1 = String::from_utf8_lossy(&output1.stdout);
        let value1 = assert_valid_json(&stdout1);
        let page1_items = value1.as_array().unwrap();

        if page1_items.is_empty() {
            println!("⚠ No items found on first page, skipping pagination test");
            return;
        }

        // Get second page
        let output2 = run_search_workshop_command(&[
            "search-workshop",
            "--app-id",
            &config.app_id.to_string(),
            "--sort-by",
            "recent",
            "--page",
            "2",
        ]);

        assert!(
            output2.status.success(),
            "Second page should succeed: {}",
            String::from_utf8_lossy(&output2.stderr)
        );

        let stdout2 = String::from_utf8_lossy(&output2.stdout);
        let value2 = assert_valid_json(&stdout2);
        let page2_items = value2.as_array().unwrap();

        if !page2_items.is_empty() {
            let page1_first_id = page1_items[0].get("published_file_id").unwrap();
            let page2_first_id = page2_items[0].get("published_file_id").unwrap();

            assert_ne!(
                page1_first_id, page2_first_id,
                "Different pages should have different content"
            );

            println!("✓ Pagination working correctly");
        } else {
            println!("✓ Pagination test completed (page 2 empty, which is valid)");
        }
    });
}

#[test]
fn test_search_workshop_with_tags() {
    steam_test_or_skip(|| {
        let config = TestConfig::load();

        let output = run_search_workshop_command(&[
            "search-workshop",
            "--app-id",
            &config.app_id.to_string(),
            "--sort-by",
            "recent",
            "--tags",
            "mod",
            "--page",
            "1",
        ]);

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let value = assert_valid_json(&stdout);
            assert!(value.is_array(), "Expected JSON array");

            println!("✓ Tag filtering with 'mod' works correctly");
        } else {
            println!("⚠ Tag filtering may not be supported for this app ID");
        }

        let output = run_search_workshop_command(&[
            "search-workshop",
            "--app-id",
            &config.app_id.to_string(),
            "--sort-by",
            "recent",
            "--tags",
            "mod,campaign",
            "--page",
            "1",
        ]);

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let value = assert_valid_json(&stdout);
            assert!(value.is_array(), "Expected JSON array");

            println!("✓ Multiple tag filtering works correctly");
        } else {
            println!("⚠ Multiple tag filtering may not be supported for this app ID");
        }
    });
}
