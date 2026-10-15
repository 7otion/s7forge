#![allow(dead_code)]

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU32, Ordering};

pub const CACHE_DIR_ENV: &str = "S7FORGE_CACHE_DIR";

/// Total War: WARHAMMER III. Every Steam fixture below belongs to this app.
pub const APP_ID: &str = "1142710";
/// Small (~13 KB) Community item that the test account is subscribed to.
pub const ITEM_ID: &str = "2858004741";
/// "Script debug activator": its description is 471 characters long (checked on the Steam web API).
pub const LONG_DESCRIPTION_ITEM_ID: &str = "2789857593";
/// Collection with ~10 children.
pub const COLLECTION_ID: &str = "3816852595";

pub struct Run {
    pub output: Output,
}

impl Run {
    pub fn code(&self) -> Option<i32> {
        self.output.status.code()
    }

    pub fn stdout(&self) -> String {
        String::from_utf8_lossy(&self.output.stdout).into_owned()
    }

    pub fn stderr(&self) -> String {
        String::from_utf8_lossy(&self.output.stderr).into_owned()
    }

    /// Asserts a successful run whose entire stdout is a single JSON document.
    pub fn json(&self) -> Value {
        assert_eq!(
            self.code(),
            Some(0),
            "expected exit code 0\nstderr:\n{}\nstdout:\n{}",
            self.stderr(),
            self.stdout()
        );
        let stdout = self.stdout();
        serde_json::from_str(&stdout)
            .unwrap_or_else(|e| panic!("stdout is not a single JSON document ({e}):\n{stdout}"))
    }

    /// Asserts a failed run (exit code 1, nothing on stdout) and returns stderr.
    pub fn error(&self) -> String {
        assert_eq!(
            self.code(),
            Some(1),
            "expected exit code 1\nstderr:\n{}\nstdout:\n{}",
            self.stderr(),
            self.stdout()
        );
        assert!(
            self.stdout().trim().is_empty(),
            "expected empty stdout on failure, got:\n{}",
            self.stdout()
        );
        self.stderr()
    }
}

/// A uniquely named directory under the system temp folder, removed on drop.
pub struct TempDir(PathBuf);

impl TempDir {
    pub fn new(label: &str) -> Self {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let unique = format!(
            "{label}-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        );
        let path = std::env::temp_dir().join("s7forge-tests").join(unique);
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("failed to create temp dir");
        Self(path)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

pub fn command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_s7forge"))
}

pub fn run(mut command: Command) -> Run {
    let output = command.output().expect("failed to spawn s7forge");
    Run { output }
}

/// Runs s7forge with a fresh, empty cache so no test sees another test's cached data.
pub fn s7forge(args: &[&str]) -> Run {
    let cache = TempDir::new("cache");
    s7forge_with_cache(cache.path(), args)
}

pub fn s7forge_with_cache(cache_dir: &Path, args: &[&str]) -> Run {
    let mut cmd = command();
    cmd.args(args).env(CACHE_DIR_ENV, cache_dir);
    run(cmd)
}

pub fn require_steam() {
    let output = Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq steam.exe", "/NH"])
        .output()
        .expect("failed to run tasklist");
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .to_ascii_lowercase()
            .contains("steam.exe"),
        "Steam must be running (and own app {APP_ID}) for this test"
    );
}

pub fn as_array(value: &Value) -> &Vec<Value> {
    value
        .as_array()
        .unwrap_or_else(|| panic!("expected a JSON array, got: {value}"))
}

pub fn assert_workshop_item(item: &Value) {
    let field = |name: &str| {
        item.get(name)
            .unwrap_or_else(|| panic!("workshop item is missing '{name}': {item}"))
    };

    assert!(field("published_file_id").as_u64().unwrap() > 0);
    assert!(!field("title").as_str().unwrap().is_empty());
    assert!(field("description").is_string());
    assert!(field("owner").is_object());
    assert_unix_seconds(field("time_created"));
    assert_unix_seconds(field("time_updated"));
    assert!(
        item.get("time_added_to_user_list").is_none(),
        "time_added_to_user_list is always 0 for these queries and must not be returned"
    );
    assert!(
        field("tags")
            .as_array()
            .is_some_and(|tags| tags.iter().all(|tag| tag.is_string())),
        "tags must be an array of strings: {item}"
    );
    assert!(field("url").is_string());
    assert!(field("num_upvotes").is_u64());
    assert!(field("num_downvotes").is_u64());
    assert!(field("file_size").is_u64());
    assert!(field("required_items").is_array());
    assert_eq!(field("file_type").as_str().unwrap(), "Community");
    assert!(!field("creator_id").as_str().unwrap().is_empty());
    assert!(!field("creator_name").as_str().unwrap().is_empty());
}

/// Timestamps are Unix seconds, like SQLite's `unixepoch()`.
/// Anything above 10^11 would be milliseconds (year 5138 in seconds).
pub fn assert_unix_seconds(value: &Value) {
    let secs = value
        .as_u64()
        .unwrap_or_else(|| panic!("expected a timestamp, got {value}"));
    assert!(
        secs > 1_000_000_000 && secs < 100_000_000_000,
        "expected Unix seconds, got {secs}"
    );
}
