# Braindump

A cross-platform CLI for instant terminal braindumping. It appends timestamped thoughts directly to a local Markdown file with zero friction, so you can capture ideas without leaving the terminal. The CLI is a capture pipe, not a task manager or journal.

## Features

- Append a timestamped entry to your braindump file with a single command
- Run `bd` with no arguments to capture multi-line input until `Ctrl+D`
- One Markdown file per user, append-only: existing entries are never edited, deleted, or reordered
- Day-grouped format (`# YYYY-MM-DD` header per day, `## HH:MM:SS` header per entry) for browsing in any Markdown renderer
- Notes starting with `-` are written as-is, never misread as flags
- `bd -h`/`--help` and `bd -v`/`--version` for usage and version; `bd -- <text>` forces literal text mode
- `bd --setup` (or the first run) interactively configures where dumps are stored, with `~/braindump/braindump.md` as the default
- A broken config (malformed, missing path, or a path that no longer accepts a dump) re-runs setup; setup requires a terminal and fails loudly without one
- Config persisted at the platform-standard location
- Silent on success and on blank invocations

## Getting started

### Quick install (macOS & Linux)

```sh
curl -fsSL https://raw.githubusercontent.com/s1mhadri/braindump/main/install.sh | sh
```

### From source

Requires Rust with Cargo.

```sh
cargo install --path .
```

### Quick start

```sh
bd remember to buy milk
```

The first run asks where dumps should live, defaulting to `~/braindump/braindump.md`. Run `bd --setup` at any time to change it. Run `bd` with no arguments to type directly in the terminal, ending with `Ctrl+D`. See `docs/usage.md` for the full file format and behavior details.

## Development

```
cargo build
cargo test
```
