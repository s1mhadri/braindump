# Braindump

A cross-platform CLI for instant terminal braindumping. `bd` appends timestamped thoughts to a local Markdown file with zero friction, so you can capture ideas without ever leaving the terminal.

It's a capture pipe, not a task manager or journal.

## Why

If you're comfortable in a terminal, opening a notes app or a browser to write something down breaks your flow. Either the thought or your focus doesn't survive the trip. Braindump is one command that appends to a plain file and gets you back to what you were doing.

It's deliberately not a full app or a TUI. There's no file to name and no section to pick. You write the thought down and you're done. Sorting it out is a problem for later, not now.

That's also why entries are timestamped: a plain chronological record is easy for you to skim later, and it's what lets an LLM reading the file understand what came before what, rather than working through a pile of loose, unordered notes.

## Features

- Single command appends a timestamped entry to your braindump file
- Run `bd` with no arguments for multi-line capture, ended with `Ctrl+D`
- One Markdown file per user, append-only: entries are never edited, deleted, or reordered
- Day-grouped format (`# YYYY-MM-DD` per day, `## HH:MM:SS` per entry), readable in any Markdown renderer
- Notes starting with `-` are written as-is, never misread as flags

## Install

**macOS & Linux**
```sh
curl -fsSL https://raw.githubusercontent.com/s1mhadri/braindump/main/install.sh | sh
bd --version
```

**From source** (requires Rust + Cargo)
```sh
cargo install --path .
```

## Quick start

```sh
bd remember to buy milk
```

The first run asks where dumps should live (default: `~/braindump.md`). Change it anytime with `bd --setup`.

## Usage

| Command | Description |
|---|---|
| `bd <text>` | Append a one-line entry |
| `bd` | Multi-line capture, ends with `Ctrl+D` |
| `bd -- <text>` | Force literal text mode |
| `bd --setup` | Configure or reconfigure where dumps are stored |
| `bd --uninstall` | Remove the `bd` binary and config (your notes are untouched) |
| `bd -v`, `--version` | Print version |
| `bd -h`, `--help` | Print help |

Full file format and behavior details: [`docs/usage.md`](docs/usage.md)

## Development

```sh
cargo build
cargo test
cargo fmt --check
cargo clippy
```

## License

MIT