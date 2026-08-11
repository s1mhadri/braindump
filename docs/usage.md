# Braindump usage

The detailed usage, format, and behavior documentation for Braindump. See `README.md` for the minimal quick start.

## Braindump file

Every dump lands in the braindump file path configured at setup, defaulting to `~/braindump/braindump.md`. One file per user, append-only: `bd` never edits, deletes, or reorders existing entries, and it never parses note text.

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
- `bd -- <text...>` forces literal text mode: the `--` is dropped and everything after it is appended verbatim, even if it looks like a command (`bd -- --search foo` appends `--search foo`).
- Command-like tokens in any position after the first are always note text (`bd call -h support` appends `call -h support`).

## Setup

Setup runs interactively the first time `bd` is invoked with no config file, on demand via `bd --setup`, and whenever the config does not yield a usable braindump file path: malformed TOML, a missing or empty `braindump_file_path` key, or a configured path that is now a directory, unwritable, or has a parent that cannot be created. The rule is a single sentence: "config didn't get me to a usable path => setup." A pending dump is not lost: the note is appended only after setup completes successfully.

Setup requires an interactive terminal on stdin. When it is triggered with piped stdin (a script, cron, or any non-interactive context) it fails loudly instead: `bd: no terminal available for setup; run `bd --setup` from a terminal` on stderr, a non-zero exit, and nothing written.

It prompts for the braindump file path with `~/braindump/braindump.md` as the default (press Enter to accept). A custom path is taken as typed: a leading `~` is expanded to the home directory and relative paths are resolved against the current directory. A path that points at an existing directory is rejected and the prompt repeats. Once accepted, parent directories are auto-created and the config is written. Ctrl+C aborts with nothing written and a non-zero exit.

The config lives at `$XDG_CONFIG_HOME/braindump/config.toml` (or `~/.config/braindump/config.toml`) on Linux/macOS and `%APPDATA%\braindump\config.toml` on Windows. It holds the braindump file path under the `braindump_file_path` key.

## Behavior notes

- Arguments are joined with a single space; interior spacing is preserved (`bd foo "bar  baz"` appends `foo bar  baz`).
- With no arguments, `bd` reads multi-line input from stdin until end-of-file: in a terminal it prints `bd: dumping, Ctrl+D to save` and reads until `Ctrl+D`; from a pipe it reads until the pipe closes with no hint.
- Interactive input trims leading and trailing blank lines; interior content (blank lines, indentation, tabs, spacing) is preserved exactly.
- Inline mode normalizes line endings to LF; interactive input preserves interior bytes exactly. Every entry ends with a single trailing LF.
- Repeated dumps append in order; the file is untouched except for the append.
- On success `bd` prints nothing. On failure it prints `bd: <error>` to stderr and exits with status 1.
- Notes starting with `-` are literal text, never parsed as flags, except for an exact slot-0 match against `-h`, `--help`, `-v`, `--version`, or `--setup`; `bd -- <text>` escapes even those.
- An invocation that produces only blank text (no arguments, an empty string, or whitespace only) is a silent no-op: nothing is written.
