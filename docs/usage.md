# Braindump usage

The detailed usage, format, and behavior documentation for Braindump. See `README.md` for the minimal quick start.

## Installation and distribution

Braindump distributes pre-built native binaries for macOS and Linux on versioned GitHub release tags (`v0.1.0` style).

Supported release targets:
- macOS Intel (`x86_64-apple-darwin`)
- macOS Apple Silicon (`aarch64-apple-darwin`)
- Linux x86_64 (`x86_64-unknown-linux-gnu`)
- Linux ARM64 (`aarch64-unknown-linux-gnu`)

### One-line installer

The one-line installer script automatically detects operating system and architecture, downloads the target release archive, verifies its SHA-256 checksum against the published checksum, and installs the binary into `/usr/local/bin` (or `$HOME/.local/bin` if `/usr/local/bin` is not writable):

```sh
curl -fsSL https://raw.githubusercontent.com/s1mhadri/braindump/main/install.sh | sh
```

A release archive whose checksum does not match is rejected before anything is installed. If no checksum file is available for a release, or the system has neither `sha256sum` nor `shasum`, the installer warns and proceeds.

Options for `install.sh`:
- `INSTALL_DIR`: Set custom installation destination directory (e.g. `INSTALL_DIR=$HOME/bin sh install.sh`).
- `VERSION`: Specify exact version to install (e.g. `VERSION=v0.1.0 sh install.sh`).

The release pipeline injects the release tag into the binary at build time, so `bd --version` reports exactly the released version (`v0.1.0` builds report `bd 0.1.0`).

## Braindump file

Every dump lands in the braindump file path configured at setup, defaulting to `~/braindump.md`. One file per user, append-only: `bd` never edits, deletes, or reorders existing entries, and it never parses note text.

Each successful dump appends exactly one entry under the current day's header:

```
# 2026-08-11

## 14:37:01
remember to buy milk
```

Format rules:

- A day header `# YYYY-MM-DD` (local time) opens each day's section.
- An entry header `## HH:MM:SS` (local time, second precision) opens each entry.
- A fresh file starts directly at the day header; exactly one blank line separates entries and day sections.
- The file ends with a single trailing LF; UTF-8 with LF line endings, no BOM.

Everything after an entry header is free-form note text, always written verbatim.

## Commands

Only an exact match at the first position against the slot-0 allow-list is treated as a command; everything else, including dash-prefixed text, is note content joined with a single space.

- `bd -h` / `bd --help` prints usage to stdout and exits.
- `bd -v` / `bd --version` prints the version and exits.
- `bd --setup` runs the interactive setup flow, which (re)configures the braindump file path.
- `bd --uninstall` removes the bd binary and the config directory (see [Uninstall](#uninstall)).
- `bd -- <text...>` forces literal text mode: the `--` is dropped and everything after it is appended verbatim, even if it looks like a command (`bd -- --search foo` appends `--search foo`).
- Command-like tokens in any position after the first are always note text (`bd call -h support` appends `call -h support`).

## Uninstall

`bd --uninstall` removes bd from the machine with a single confirmed command. It is a slot-0 command, so it works without trusting a remote uninstaller script and needs no network access.

Before deleting anything it prints a summary of what will be removed (the binary path and the config directory) and requires an explicit `y`/`Y`. Enter, `n`, Ctrl+C, or EOF aborts, deletes nothing, and exits non-zero.

On confirmation it removes the config directory at the platform-standard location (`$XDG_CONFIG_HOME/braindump`, `~/.config/braindump`, or `%APPDATA%\braindump`) and unlinks the running binary itself. When the binary's directory is not writable (typically a `/usr/local/bin` install), the uninstall escalates with `sudo` just like the installer does. Re-running when the binary or config is already gone succeeds and reports what was and was not removed rather than erroring.

The braindump file is never deleted: your notes are your data, and after uninstalling you decide what to do with them. If the configured braindump file lives inside the config directory, the directory itself is kept (only the config file is removed) so the notes survive.

## Setup

Setup runs interactively the first time `bd` is invoked with no config file, on demand via `bd --setup`, and whenever the config does not yield a usable braindump file path: malformed TOML, a missing or empty `braindump_file_path` key, or a configured path that is now a directory, unwritable, or has a parent that cannot be created. The rule is a single sentence: "config didn't get me to a usable path => setup." A pending dump is not lost: the note is appended only after setup completes successfully.

Setup requires an interactive terminal on stdin. When it is triggered with piped stdin (a script, cron, or any non-interactive context) it fails loudly instead: `bd: no terminal available for setup; run `bd --setup` from a terminal` on stderr, a non-zero exit, and nothing written.

It prompts for the braindump file path (`Braindump file path [default: ~/braindump.md]: `), defaulting to `~/braindump.md` (press Enter to accept). A custom path is taken as typed: a leading `~` is expanded to the home directory and relative paths are resolved against the current directory. A path that points at an existing directory is rejected and the prompt repeats.

When setup is run with an existing usable braindump file path and a different target path is selected, setup asks whether to migrate existing entries (`Migrate existing braindump file? [Y/n]: `). Migration is the default (press Enter or `y`). Choosing migrate reads the source entries verbatim as raw bytes and merges them into the selected target file, preserving entry order and appending into an existing target file if present. If both files are non-empty, exactly one LF byte is inserted at the boundary unless the target already ends in LF or the source begins with LF. Choosing new (`n`) starts using the new target path without copying entries or deleting or truncating either file. First-run setup, defective config without a usable prior path, or re-selecting the same path skip the migration prompt.

Failure and cancellation guarantees: Ctrl+C, EOF, or cancellation at either prompt exits immediately with a non-zero exit code, leaving the config, old braindump file, target file, and any pending dump unchanged. Migration writes complete staged work via a temporary sibling file in the target directory before renaming into place. If migration fails or config persistence fails after migration, the old configured path and old source file remain intact, a recoverable error is reported, and any pending dump remains unappended.

The config lives at `$XDG_CONFIG_HOME/braindump/config.toml` (or `~/.config/braindump/config.toml`) on Linux/macOS and `%APPDATA%\braindump\config.toml` on Windows. It holds the braindump file path under the `braindump_file_path` key.

## Behavior notes

- Arguments are joined with a single space; interior spacing is preserved (`bd foo "bar  baz"` appends `foo bar  baz`).
- With no arguments, `bd` reads multi-line input from stdin until end-of-file: in a terminal it prints `bd: dumping, Ctrl+D to save` and reads until `Ctrl+D`; from a pipe it reads until the pipe closes with no hint.
- Interactive input trims leading and trailing blank lines; interior content (blank lines, indentation, tabs, spacing) is preserved exactly.
- Inline mode normalizes line endings to LF; interactive input preserves interior bytes exactly. Every entry ends with a single trailing LF.
- Repeated dumps append in order; the file is untouched except for the append.
- On success `bd` prints nothing. On failure it prints `bd: <error>` to stderr and exits with status 1.
- Notes starting with `-` are literal text, never parsed as flags, except for an exact slot-0 match against `-h`, `--help`, `-v`, `--version`, `--setup`, or `--uninstall`; `bd -- <text>` escapes even those.
- An invocation that produces only blank text (no arguments, an empty string, or whitespace only) is a silent no-op: nothing is written.
