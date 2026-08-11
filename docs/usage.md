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

## Behavior notes

- Arguments are joined with a single space; interior spacing is preserved (`bd foo "bar  baz"` appends `foo bar  baz`).
- Line endings in the note are normalized to LF, and the note keeps a single trailing LF.
- Repeated dumps append in order; the file is untouched except for the append.
- On success `bd` prints nothing. On failure it prints `bd: <error>` to stderr and exits with status 1.
- Notes starting with `-` are literal text, never parsed as flags.
- An invocation that produces only blank text (no arguments, an empty string, or whitespace only) is a silent no-op: nothing is written.
