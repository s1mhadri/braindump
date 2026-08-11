# Braindump

A cross-platform CLI for instant terminal braindumping. It appends timestamped thoughts directly to a local Markdown file with zero friction, so you can capture ideas without leaving the terminal. The CLI is a capture pipe, not a task manager or journal.

## Features

- Append a timestamped entry to your braindump file with a single command
- One Markdown file per user, append-only: existing entries are never edited, deleted, or reordered
- Day-grouped format (`# YYYY-MM-DD` header per day, `## HH:MM:SS` header per entry) for browsing in any Markdown renderer
- Notes starting with `-` are written as-is, never misread as flags
- Silent on success and on blank invocations

## Getting started

Requires Rust with Cargo.

```
cargo install --path .
bd remember to buy milk
```

This appends the note to `~/braindump/braindump.md`, created on first use. See `docs/usage.md` for the full file format and behavior details.

## Development

```
cargo build
cargo test
```
