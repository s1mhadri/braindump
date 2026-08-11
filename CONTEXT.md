# Brain Dump

A single-purpose CLI for capturing thoughts straight to a local Markdown braindump file. One braindump file per user; the CLI is a capture pipe, not a task manager or journal.

## Language

**Braindump file**:
The single Markdown file where all dumps accumulate. Configured once at setup.
_Avoid_: journal, target file, notes file

**Entry**:
One timestamped block in the braindump file, consisting of a time header (`## HH:MM:SS`) and free-form note text. Every successful invocation appends exactly one entry, under the day's day header.
_Avoid_: thought, todo, reminder, note-type

**Day header**:
The `# YYYY-MM-DD` line, in local time, that opens each day's section of the braindump file.
_Avoid_: date heading, section header

**Time header**:
The `## HH:MM:SS` line, in local time, that opens each entry.
_Avoid_: timestamp, entry heading

**Note**:
The free-form markdown text inside an entry. Never parsed or categorised by the tool.
_Avoid_: content, body

**Dump**:
The action of capturing a thought and appending it to the braindump file as a new entry.
_Avoid_: save, log, record

**Braindump file path**:
The location on disk of the braindump file. Stored in config.
_Avoid_: target_file, journal path, storage path

**Config**:
The persisted settings file holding the braindump file path.
_Avoid_: settings, options, preferences

**Setup**:
The interactive flow, triggered on first run, via `bd --setup`, or whenever the config is defective, that (re)configures the braindump file path. Requires an interactive terminal; without one the tool fails loudly and writes nothing.
_Avoid_: onboarding, wizard, configuration

**Migration**:
Copying an existing braindump file's entries verbatim into a new braindump file when the user changes the braindump file path. Never automatic; always an explicit choice during setup, with migration as the default.
_Avoid_: transfer, import, move