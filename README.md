# s7forge

A small command-line tool for the Steam Workshop. It runs one command, prints JSON, and
exits: look items up, subscribe, ask Steam to download, read what the local Steam client
knows about your subscriptions, find where a game is installed.

It exists so that another program can delegate its Steam work to a process that lives for a moment, instead of linking the Steam SDK itself.

## Getting it

Windows only. Each [release](https://github.com/7otion/s7forge/releases) ships two files
that belong together:

- `s7forge.exe`
- `steam_api64.dll`, the Steam SDK library; keep it next to the executable.

Commands that talk to Steam need the Steam client running and logged in.

### Building from source

Rust 1.85 or newer (edition 2024). The Steam SDK is pulled in by the `steamworks` crate, so nothing has to be downloaded by hand:

```bash
cargo build --release
```

`steam_api64.dll` ends up in the build output (`target/release/build/steamworks-sys-*/out/`);
copy it next to `target/release/s7forge.exe`. Tests find it on their own, see
[TESTING.md](TESTING.md).

## How it answers

- One JSON document on stdout, exit code 0.
- On failure: a message on stderr, exit code 1, nothing on stdout.
- Most commands take `--app-id <APP_ID>`, the Steam app id of the game (Warhammer III is
  `1142710`). It can go before or after the command name.
- `s7forge --help`, `s7forge <command> --help`, `s7forge --version` (prints `s7forge 0.3.0`).

## Commands

### Items

```bash
# Details of one or more items
s7forge workshop-items --app-id 1142710 --item-ids 123,456,789

# A collection: its own details, then its items
s7forge collection-items --app-id 1142710 --item-id 987654321
```

### Subscriptions

```bash
# Everything the logged-in account is subscribed to, with full details
s7forge subscribed-items --app-id 1142710

# The same list as local state only, no titles: fast, usually no Steam API
s7forge workshop-state --app-id 1142710

# When this account subscribed to each item
s7forge subscription-times --app-id 1142710

# Subscribe or unsubscribe, several at once
s7forge subscribe --app-id 1142710 --item-ids 123,456,789
s7forge unsubscribe --app-id 1142710 --item-ids 123,456,789
```

`subscribe` and `unsubscribe` return one result per item, in the order given. A failed item
doesn't stop the rest:

```json
[
  { "item_id": 1, "success": false, "error": "Steam API error: FileNotFound" },
  { "item_id": 123, "success": true }
]
```

### Downloads

```bash
# Ask Steam to download or update several subscribed items; returns at once
s7forge download-items --app-id 1142710 --item-ids 123,456,789

# Download state and progress of several items
s7forge download-progress --app-id 1142710 --item-ids 123,456,789

# Progress of one item
s7forge check-item-download --app-id 1142710 --item-id 123456789

# Force a (re)download of one subscribed item and wait for Steam's answer
s7forge download-workshop-item --app-id 1142710 --item-id 123456789
```

### Discovery

```bash
# Text search
s7forge search-workshop --app-id 1142710 --query "tank" --sort-by relevance --page 1

# Trending this week; --period only applies to popular
s7forge search-workshop --app-id 1142710 --sort-by popular --period one-week --page 1

# Newest, most subscribed, recently updated
s7forge search-workshop --app-id 1142710 --sort-by recent
s7forge search-workshop --app-id 1142710 --sort-by most-subscribed
s7forge search-workshop --app-id 1142710 --sort-by recently-updated

# Narrow by tags
s7forge search-workshop --app-id 1142710 --query "weapon" --tags "mod,multiplayer"

# Which tags a game's Workshop uses
s7forge discover-tags --app-id 1142710
```

`--sort-by`: `relevance` (needs `--query`), `popular` (the only one that takes `--period`:
`today`, `one-week`, `three-months`, `six-months`, `one-year`; default `one-week`), `recent`,
`most-subscribed`, `recently-updated`. `discover-tags` returns an array of tag names.

### Paths

```bash
# Where a game is installed
s7forge app-installation-path --app-id 1142710

# Its Workshop content folder
s7forge workshop-path --app-id 1142710

# Every Steam library folder on this PC
s7forge steam-library-paths
```

`app-installation-path` and `steam-library-paths` read Steam's own files first and only ask
the running client when those can't be read, so they also work with Steam closed.

### Several commands in one run

```bash
s7forge --app-id 1142710 combined --subscribed-items --workshop-path
s7forge --app-id 1142710 combined --workshop-items --item-ids 123,456 --discover-tags
```

`--app-id` comes before `combined` and applies to every subcommand; each subcommand's own
options follow its flag. Subcommands: `subscribed-items`, `workshop-state`,
`subscription-times`, `workshop-path`, `search-workshop`, `workshop-items`,
`check-item-download`, `collection-items`, `discover-tags`. The output is one object keyed
by subcommand name (`search-workshop-1`, `workshop-items-0`, … for the ones that can repeat);
a failed subcommand gets `{ "error": "…" }` under its key and the others still answer.

### Cache

```bash
s7forge clear-cache
```

## What the JSON looks like

### An item

Every item from `workshop-items`, `subscribed-items`, `collection-items` and
`search-workshop`:

```json
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
  "owner": { "steam_id64": "number", "steam_id32": "string", "account_id": "number" },
  "time_created": "number (Unix seconds)",
  "time_updated": "number (Unix seconds)",
  "visibility": "string",
  "banned": "boolean",
  "accepted_for_use": "boolean",
  "tags": "array of strings, exactly as stored on Steam",
  "tags_truncated": "boolean",
  "url": "string",
  "num_upvotes": "number",
  "num_downvotes": "number",
  "num_children": "number",
  "preview_url": "string | null",
  "statistics": { "num_subscriptions": "number | null", "num_favorites": "number | null", "...": "..." },
  "required_items": "array of numbers",
  "file_type": "string",
  "file_size": "number"
}
```

Items from `subscribed-items` also carry `time_subscribed` (Unix seconds, or `null` when
Steam didn't say).

### Local install state

`install` and `state` are what the Steam client on this PC knows about the item. They are
read fresh on every run and never cached:

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

`install.time_updated` is the `time_updated` of the version installed here. When it is lower
than the item's own `time_updated`, a newer version is published but not installed yet.

### A collection

`collection-items` returns the collection's own details and its items:

```json
{
  "details": {
    "id": 987654321,
    "title": "string",
    "description": "string",
    "creator_id": "string",
    "creator_name": "string",
    "preview_url": "string | null",
    "time_created": 1742939179,
    "time_updated": 1742939179,
    "num_upvotes": 10,
    "num_downvotes": 0
  },
  "items": ["array of items, as above"]
}
```

### Downloads

`download-items` asks the Steam client to download or update each subscribed item and
returns at once. `outcome` is `pending` (queued; follow it with `download-progress`),
`completed` (Steam answered immediately, so the item was already up to date), `refused`
(not subscribed, or rejected by Steam) or `failed` (Steam reported an error straight away,
given in `error`):

```json
[
  { "item_id": 2858004741, "outcome": "completed" },
  { "item_id": 1, "outcome": "refused", "error": "Steam refused to download the item" }
]
```

`download-progress` returns the local state of several items plus the bytes of a download in
progress (`null` otherwise):

```json
[
  {
    "item_id": 2858004741,
    "state": { "subscribed": true, "installed": false, "needs_update": false, "downloading": true, "download_pending": false },
    "downloaded_bytes": 10485760,
    "total_bytes": 52428800,
    "installed_time_updated": 1742939179
  }
]
```

`check-item-download` returns one item's progress:

```json
{
  "is_downloading": true,
  "downloaded_bytes": 10485760,
  "total_bytes": 52428800,
  "progress_percentage": 20.0,
  "download_complete": false
}
```

### Subscription state

`workshop-state` lists what the local Steam client knows about every subscription of the
logged-in account, without titles or other details, so a caller can tell what changed since
it last looked:

```json
{
  "route": "files",
  "account_id": 288138212,
  "items": [
    {
      "item_id": 2858004741,
      "installed": true,
      "folder": "F:\\SteamLibrary\\steamapps\\workshop\\content\\1142710\\2858004741",
      "size_on_disk": 13386,
      "installed_time_updated": 1742939179,
      "latest_time_updated": 1742939179,
      "needs_update": false,
      "download_pending": false
    }
  ]
}
```

It reads Steam's own `appworkshop_<app_id>.acf` and `loginusers.vdf` (`route: "files"`, a
few milliseconds, no Steam API). If those files can't be read it asks the running Steam
client instead (`route: "steam"`; `latest_time_updated` is then `null`) and remembers the
failure for a day, so later calls skip the files until then. If the Steam client can't be
reached either, it lists the item folders under `<library>/steamapps/workshop/content/<app_id>`
(`route: "folder"`: every item counts as installed, sizes and times are `null`).

### Subscription times

`subscription-times` returns `[{ "item_id": 2858004741, "time_subscribed": 1747551218 }]`.
Steam only gives this time when listing the account's subscriptions page by page (50 per
page), not when looking items up by id, so s7forge walks the list newest first and caches the
times; later runs usually need one page.

## Caching

- Cache files live in `cache\` next to `s7forge.exe`. Set `S7FORGE_CACHE_DIR` to use another
  directory, e.g. when the executable sits in a read-only folder such as Program Files.
- Caching is best-effort: if the cache directory can't be read or written, commands still
  succeed without it.
- What is cached, and for how long:
  - item details: 30 minutes, so `time_updated` shows new versions soon
  - creator names: 7 days (an unknown name can take up to 2 seconds to look up)
  - search results: 10 minutes
  - subscription times: 1 year (each run still checks the newest subscriptions, so a
    re-subscription shows its new time)
  - the "Steam's files couldn't be read" note of `workshop-state`: 1 day
- Local install state (`install`, `state`, `download-progress`) is never cached.
- `s7forge clear-cache` removes everything above.

## Testing

See [TESTING.md](TESTING.md). The Steam tests need a logged-in client that owns Warhammer
III and is subscribed to one small item.

## Licence

MIT, see [LICENSE](LICENSE).

## Credits

- [steamworks-rs](https://github.com/Noxime/steamworks-rs), the Rust bindings to the Steam API.
- [steamworks.js](https://github.com/ceifa/steamworks.js), the reference implementation this started from (MIT).

## Contributing

Contributions are welcome. Before opening a pull request: `cargo fmt`, `cargo clippy`,
`cargo test` (see TESTING.md for what the Steam tests need), and keep the JSON output
backward compatible where you can; other programs parse it.
