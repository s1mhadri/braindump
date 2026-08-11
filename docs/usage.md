# Braindump usage

The detailed usage, format, and behavior documentation for Braindump. See `README.md` for the minimal quick start.

## Braindump file

Every dump lands in `~/braindump/braindump.md`, created on first use. One file per user, append-only: `bd` never edits, deletes, or reorders existing entries, and it never parses note text.

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
- `bd --setup` routes to the setup flow, which reconfigures the braindump file path. (Not yet implemented; reports a clear error.)
- `bd -- <text...>` forces literal text mode: the `--` is dropped and everything after it is appended verbatim, even if it looks like a command (`bd -- --search foo` appends `--search foo`).
- Command-like tokens in any position after the first are always note text (`bd call -h support` appends `call -h support`).

## Behavior notes

- Arguments are joined with a single space; interior spacing is preserved (`bd foo "bar  baz"` appends `foo bar  baz`).
- With no arguments, `bd` reads multi-line input from stdin until end-of-file: in a terminal it prints `bd: dumping, Ctrl+D to save` and reads until `Ctrl+D`; from a pipe it reads until the pipe closes with no hint.
- Interactive input trims leading and trailing blank lines; interior content (blank lines, indentation, tabs, spacing) is preserved exactly.
- Inline mode normalizes line endings to LF; interactive input preserves interior bytes exactly. Every entry ends with a single trailing LF.
- Repeated dumps append in order; the file is untouched except for the append.
- On success `bd` prints nothing. On failure it prints `bd: <error>` to stderr and exits with status 1.
- Notes starting with `-` are literal text, never parsed as flags, except for an exact slot-0 match against `-h`, `--help`, `-v`, `--version`, or `--setup`; `bd -- <text>` escapes even those.
- An invocation that produces only blank text (no arguments, an empty string, or whitespace only) is a silent no-op: nothing is written.
