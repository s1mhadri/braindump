## Agent skills

### Issue tracker

Issues and specs live in the GitHub issue tracker (s1mhadri/braindump). See `docs/agents/issue-tracker.md`.

### Triage labels

Five canonical roles, each label equal to its name: needs-triage, needs-info, ready-for-agent, ready-for-human, wontfix. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: one `CONTEXT.md` and `docs/adr/` at the repo root. See `docs/agents/domain.md`.

### Docs maintenance

- Keep docs in sync with the code. For every change, check each doc for statements it invalidates and update them alongside the code. A diff where the code and its docs disagree means the work is not done.
- **README.md** — minimal user-facing essentials: Update when the feature set, install steps, the invocation, or the output format change; keep the deep behavior in `docs/usage.md`.
