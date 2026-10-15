# Testing Guide

## Requirements

- Steam must be running and logged in to an account that owns
  **Total War: WARHAMMER III** (app id `1142710`) and has it installed.
- The account must be subscribed to workshop item `2858004741` (a ~13 KB mod).

The fixtures live in `tests/common/mod.rs`.

## Running Tests

```bash
cargo test
```

This runs:

- Unit tests inside `src/` (`#[cfg(test)]` modules): pure logic, such as how Steam item states map to output. They don't need Steam running.
- `tests/cache.rs`: cache location and cache failure handling.
- `tests/offline.rs`: commands that never touch the Steam API (CLI parsing, cache, local paths).
- `tests/steam.rs`: commands that talk to the Steam API. Each test fails immediately if `steam.exe` is not running.

### Subscription tests

```bash
cargo test -- --ignored
```

`subscribe_and_unsubscribe_round_trip` really changes the account's subscriptions. It then
restores the original state, so it is ignored by default.

## How tests run the binary

Tests run the binary that cargo built (`CARGO_BIN_EXE_s7forge`), not `cargo run`.
Cargo puts the steamworks-sys output folder on `PATH` for test processes, so
`steam_api64.dll` is found without copying it next to the executable.

A test passes only when the command exits with code 0 **and** its entire stdout
parses as a single JSON document. Failure tests require exit code 1 and an empty stdout.
