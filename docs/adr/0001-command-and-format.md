# Command grammar: slot-0 allow-list with a `--` escape hatch

The CLI is a capture sink whose primary input is free-form text, which legitimately begins with `-` (code, shell examples). We decided that only an exact match at the first position against a small allow-list (`-h`, `--help`, `-v`, `--version`, `--setup`, and future dash-prefixed commands like `--search`) is interpreted as a command; everything else, including dash-prefixed text, is note content joined by a single space. In the common POSIX alternative (clap default), any `-` token is a flag and unknown flags error, which would break core use cases like `bd git checkout -f` or `bd call -h support`.

A `--` token at position zero permanently forces text mode for the rest of the invocation, giving users a universal escape hatch regardless of how the command list grows. Adding any new slot-0 command is a breaking decision for notes that begin with that exact string.

# Journal file format: day-grouped entries

Each day of the braindump file opens with a `# YYYY-MM-DD` header; each entry is a `## HH:MM:SS` header (local time, second precision) followed by free-form note text. A fresh file starts directly at the day header, exactly one blank line separates entries (and day headers), and the file ends with a single trailing LF; UTF-8, LF line endings, no BOM on every platform.

The alternative was the spec's flat `# YYYY-MM-DD HH:MM:SS` per entry, which is maximally greppable (`rg '^# '` gives a full chronology) but produces hundreds of level-1 headers, which is semantically wrong for a Markdown chronology and breaks renderer outlines. Day-grouping keeps ISO-style timestamps while giving the document a correct `Day → Entries` hierarchy. The cost is that a single entry line is no longer an absolute standalone timestamp — greps need the day header for context — a fair trade since this is fundamentally a file for humans to browse.