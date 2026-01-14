use crate::test_modules::utils::{TestConfig, assert_valid_json, run_command};

#[test]
fn test_app_installation_path() {
    let config = TestConfig::load();
    let output = run_command(&[
        "app-installation-path",
        "--app-id",
        &config.app_id.to_string(),
    ]);

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_valid_json(&stdout);
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Error:"));
    }
}
