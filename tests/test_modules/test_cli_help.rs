use crate::test_modules::utils::run_command;

#[test]
fn test_cli_help() {
    let output = run_command(&["--help"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("Steam utility"),
        "Help should mention 'Steam utility'"
    );

    assert!(
        stdout.contains("search-workshop"),
        "Help should list 'search-workshop' command"
    );
    assert!(
        stdout.contains("discover-tags"),
        "Help should list 'discover-tags' command"
    );
    assert!(
        stdout.contains("workshop-items"),
        "Help should list 'workshop-items' command"
    );
    assert!(
        stdout.contains("collection-items"),
        "Help should list 'collection-items' command"
    );
    assert!(
        stdout.contains("subscribed-items"),
        "Help should list 'subscribed-items' command"
    );
    assert!(
        stdout.contains("check-item-download"),
        "Help should list 'check-item-download' command"
    );
    assert!(
        stdout.contains("subscribe"),
        "Help should list 'subscribe' command"
    );
    assert!(
        stdout.contains("unsubscribe"),
        "Help should list 'unsubscribe' command"
    );
    assert!(
        stdout.contains("clear-cache"),
        "Help should list 'clear-cache' command"
    );
    assert!(
        stdout.contains("workshop-path"),
        "Help should list 'workshop-path' command"
    );
    assert!(
        stdout.contains("steam-library-paths"),
        "Help should list 'steam-library-paths' command"
    );
    assert!(
        stdout.contains("app-installation-path"),
        "Help should list 'app-installation-path' command"
    );

    assert!(
        stdout.contains("--help"),
        "Help should mention '--help' option"
    );
    assert!(
        stdout.contains("--version"),
        "Help should mention '--version' option"
    );
}
