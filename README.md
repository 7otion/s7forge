A CLI utility for Steam Workshop operations via Steam SDK, designed as a sidecar binary for delegating Steam integration tasks.

## Installation

### Prerequisites

- Windows OS
- Rust 1.85+ (edition 2024)
- Steam SDK files (steam_api64.dll, steam_api64.lib) in binary directory
- Steam client running

### Build from Source

```bash
git clone <repository-url>
cd s7forge
cargo build --release
```

## Usage

Most commands require `--app-id` parameter. Use `s7forge help` and `s7forge <command> --help` for detailed help.

### Commands

#### Workshop Items

```bash
# Get item details
s7forge workshop-items --app-id 548430 --item-ids 123,456,789

# Check download status
s7forge check-item-download --app-id 548430 --item-id 123456789

# Get collection items
s7forge collection-items --app-id 548430 --item-id 987654321
```

#### Subscriptions

```bash
# Subscribe to items (batch supported)
s7forge subscribe --app-id 548430 --item-ids 123,456,789

# Unsubscribe from items (batch supported)
s7forge unsubscribe --app-id 548430 --item-ids 123,456,789

# List subscribed items
s7forge subscribed-items --app-id 548430
```

`subscribe` and `unsubscribe` return one result per requested item, in order. A failed item
does not stop the rest of the batch:

```json
[
  { "item_id": 1, "success": false, "error": "Steam API error: FileNotFound" },
  { "item_id": 123, "success": true }
]
```

#### Discovery

```bash
# Search workshop with multiple sort options
s7forge search-workshop --app-id 548430 --query "tank" --sort-by relevance --page 1

# Popular items with time periods
s7forge search-workshop --app-id 548430 --sort-by popular --period one-week --page 1

# Recent items
s7forge search-workshop --app-id 548430 --sort-by recent --page 1

# Most subscribed items
s7forge search-workshop --app-id 548430 --sort-by most-subscribed --page 1

# Recently updated items
s7forge search-workshop --app-id 548430 --sort-by recently-updated --page 1

# Search with tag filtering
s7forge search-workshop --app-id 548430 --query "weapon" --tags "mod,multiplayer" --page 1

# Discover available tags for a game
s7forge discover-tags --app-id 548430
```

**Sort Options:**
- `relevance` - Text search relevance (requires query text)
- `popular` - Trending items within `--period` (default `one-week`); the only sort that accepts `--period`
- `recent` - Recently published items
- `most-subscribed` - Most subscribed items
- `recently-updated` - Recently updated items

**Time Periods:** `today`, `one-week`, `three-months`, `six-months`, `one-year`

#### Management

```bash
# Re-Download (Trigger forced update) a workshop item
s7forge download-workshop-item --app-id 548430 --item-id 123456789

# Get workshop directory path
s7forge workshop-path --app-id 548430

# List Steam library paths
s7forge steam-library-paths

# Get installation path of an app
s7forge app-installation-path --app-id 1142710

# Clear cache
s7forge clear-cache
```

### Example Workshop Item Output

```json
[
  {
    "published_file_id": "number",
    "creator_steam_game_id": "number | null",
    "creator_id": "string",
    "creator_name": "string",
    "install": "object | null (null when not downloaded on this PC)",
    "state": "object (local Steam client flags, see below)",
    "consumer_steam_game_id": "number | null",
    "title": "string",
    "description": "string",
    "owner": {
      "steam_id64": "number",
      "steam_id32": "string",
      "account_id": "number"
    },
    "time_created": "number (Unix seconds)",
    "time_updated": "number (Unix seconds)",
    "visibility": "string",
    "banned": "boolean",
    "accepted_for_use": "boolean",
    "tags": "array of strings (exactly as stored on Steam, e.g. [\"mod\", \"battle\"])",
    "tags_truncated": "boolean",
    "url": "string",
    "num_upvotes": "number",
    "num_downvotes": "number",
    "num_children": "number",
    "preview_url": "string | null",
    "statistics": {
      "num_subscriptions": "number | null",
      "num_favorites": "number | null",
      "num_followers": "number | null",
      "num_unique_subscriptions": "number | null",
      "num_unique_favorites": "number | null",
      "num_unique_followers": "number | null",
      "num_unique_website_views": "number | null",
      "report_score": "number | null",
      "num_seconds_played": "number | null",
      "num_playtime_sessions": "number | null",
      "num_comments": "number | null",
      "num_seconds_played_during_time_period": "number | null",
      "num_playtime_sessions_during_time_period": "number | null"
    },
    "required_items": "array of numbers",
    "file_type": "string",
    "file_size": "number"
  }
]
```

### Local install state

Every item from `workshop-items`, `subscribed-items`, `collection-items` and
`search-workshop` carries what the local Steam client knows about it. This is
read fresh on every run and never cached.

```json
"install": {
  "folder": "F:\\SteamLibrary\\steamapps\\workshop\\content\\1142710\\2858004741",
  "size_on_disk": 13386,
  "time_updated": 1742939179
},
"state": {
  "subscribed": true,
  "installed": true,
  "needs_update": false,
  "downloading": false,
  "download_pending": false
}
```

`install.time_updated` is the `time_updated` of the version installed on this
PC. If it is lower than the item's own `time_updated`, a newer version is
published but not installed yet.

### Caching

- Cache files live in `cache\` next to `s7forge.exe`. Set the `S7FORGE_CACHE_DIR` environment variable to use another directory, e.g. when the executable is installed in a read-only folder such as Program Files.
- Caching is best-effort: if the cache directory cannot be read or written, commands still succeed without it.
- What is cached, and for how long:
  - workshop item details: 30 minutes (so `time_updated` shows new mod versions soon)
  - creator names: 7 days (looking up an unknown name can take up to 2 seconds)
  - search results: 10 minutes
- Local install state (`install`, `state`) is never cached.
- Use `s7forge clear-cache` to clear all cached data

## License

This project is licensed under the MIT License.

## Credits

- [steamworks.js](https://github.com/ceifa/steamworks.js) - Reference implementation (MIT License)
- [steamworks-rs](https://github.com/Noxime/steamworks-rs) - Rust Steam API bindings

## Contributing

Contributions are welcome! Please ensure:

1. Code follows Rust conventions and passes `cargo clippy`
2. Code is formatted with `cargo fmt`
3. Code is tested with `cargo test`
4. New features include appropriate error handling
5. Changes maintain backward compatibility where possible
