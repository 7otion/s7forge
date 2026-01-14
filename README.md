A CLI utility for Steam Workshop operations via Steam SDK, designed as a sidecar binary for delegating Steam integration tasks.

## Installation

### Prerequisites

- Windows OS
- Rust 1.70+
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
- `popular` - Most voted items (supports time periods)
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
    "consumer_steam_game_id": "number | null",
    "title": "string",
    "description": "string",
    "owner": {
      "steam_id64": "number",
      "steam_id32": "string",
      "account_id": "number"
    },
    "time_created": "number",
    "time_updated": "number",
    "time_added_to_user_list": "number",
    "visibility": "string",
    "banned": "boolean",
    "accepted_for_use": "boolean",
    "tags": "string",
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

### Caching

- Creator names and workshop items cached in `{executable_directory}/cache/`
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
