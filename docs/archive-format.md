# Archive format

Claude Vault makes a file-level mirror rather than converting conversations into a proprietary database. This keeps the backup inspectable and preserves data the current UI may not yet understand.

## Source trees

| Data | Default source | Purpose |
| --- | --- | --- |
| Session metadata | Claude app data / `claude-code-sessions` | Titles, identifiers, turn counts, and discovery |
| Transcripts | `%USERPROFILE%\.claude\projects` | JSONL conversation history and project files |
| Workspaces | Claude app data / `scratch-workspaces` | Files created or attached during local work |

Claude app data is auto-detected under the Microsoft Store package cache, `%LOCALAPPDATA%\Claude`, or `%APPDATA%\Claude`. The metadata and transcript roots can be overridden in Settings.

## Vault layout

```text
vault/
├── metadata/      exact mirror of claude-code-sessions
├── transcripts/   exact mirror of .claude/projects
└── workspaces/    exact mirror of scratch-workspaces
```

Relative paths are preserved. Files are compared in streaming 64 KiB blocks before an archived copy is updated, avoiding loading large attachments into memory.

## Restore behavior

Each archive section is restored only to its matching source tree. A target file is copied only when it does not already exist. Existing target files are skipped even when their contents differ, preventing the vault from rolling back newer Claude data.

## Display behavior

The viewer extracts user and assistant text blocks from JSONL records. Tool payloads and internal system-reminder blocks are omitted visually. This is presentation-only filtering: archived files remain unchanged.
