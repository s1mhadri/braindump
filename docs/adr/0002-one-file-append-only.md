# One braindump file per user, append-only, migration is explicit

Each user has exactly one braindump file whose path is fixed once at setup. The tool is strictly append-only: it never edits, deletes, searches, or reorders existing entries, and it never parses note content.

The decision is anchored in the product's identity: it is a capture pipe, not a task manager or journal. Users jot quickly and sort it out later elsewhere. Structured note types, per-topic files, and any file mutation were deliberately rejected as out of scope for v0.1. When the path is changed at setup, existing entries are never auto-migrated; the user explicitly chooses migrate-or-new (migrate being the default), and migrated entries are merged into the target file preserving order.