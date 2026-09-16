# Account switching (experimental, Windows only)

This feature switches the main Claude Desktop data directory. It does not launch isolated windows, sign users in automatically, modify the Claude executable, or read credentials.

## Workflow

1. Fully quit Claude Desktop, including background/tray processes.
2. Open Accounts in Claude Vault, name the current account, and select **Save current**.
3. Add a new account with **Add new**. This creates an empty profile snapshot.
4. Select **Switch**. If Claude is running, Vault asks you to quit it and does not switch.
5. Select **Open Claude**, then sign in normally once for that new account.
6. Before switching back, quit Claude. Vault snapshots the current profile and installs the selected saved profile.

A session may expire or be revoked, requiring another login. The selected account label is user supplied; Vault does not verify account identity or subscription status.

## Safety and recovery

Account snapshots live under the app's local-data directory in `account-profiles`, outside `vault`. They contain sensitive session data and must never be exported, uploaded, or shared. They are intended only for the same Windows user on the same computer. Existing Claude encryption is preserved; Vault does not claim to encrypt every profile file independently.

Vault checks for `Claude.exe` processes before a snapshot and immediately before replacing the live directory. It uses strict copies, rejects symbolic links, and aborts on file-access errors. Backup/restore operations and account switching are serialized.

The new profile is prepared in a sibling staging directory first. The live directory is renamed to a `Claude-vault-recovery-*` backup before replacement. A recovery journal is recorded before this operation. If an operation fails or the app stops during replacement, switching and opening Claude are blocked until **Recover previous profile** is used. Recovery preserves displaced data rather than deleting it.

Previous snapshots and recovery folders are retained intentionally. They can consume significant disk space and also contain sensitive session data. Automated cleanup is not provided in this experimental release.

Do not manually start Claude during switching. The process check cannot prevent an unrelated launcher from starting Claude between checks.

## Limitations

- Currently enabled only on Windows. macOS Keychain and Linux credential-store behavior require separate implementation and verification.
- Claude Code CLI credentials and other operating-system credential stores are not switched.
- MSIX launches use package identity; classic installations use the detected executable. A manual executable path can be supplied if discovery fails.
- Claude's undocumented profile format may change. Build/tests do not establish that a saved login works in every Claude release.
- Switching real accounts must be verified manually in Claude; automated tests never touch real user profiles.

The original separate-profile launcher approach was researched in [Claude-Code-Desktop-Switcher](https://github.com/PriyanshuGeTRekT/Claude-Code-Desktop-Switcher). This implementation uses a different, main-profile snapshot workflow.
