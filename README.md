# Never lose your Claude chats when switching accounts

[English](README.md) · [Русский](README.ru.md) · [简体中文](README.zh-CN.md) · [Deutsch](README.de.md) · [Español](README.es.md) · [Français](README.fr.md)

![Tauri 2](https://img.shields.io/badge/Tauri-2-24C8DB?logo=tauri&logoColor=white) ![Rust](https://img.shields.io/badge/Rust-backend-000000?logo=rust) ![Local only](https://img.shields.io/badge/privacy-local--only-22c55e) ![MIT](https://img.shields.io/badge/license-MIT-blue)

**Claude Vault** keeps an independent, local copy of your Claude Desktop and Claude Code conversations so an account switch cannot make valuable work disappear.

The project is built with Tauri 2, Rust, React, TypeScript, Tailwind CSS, and genuine shadcn-style Radix UI components. It does not use Anthropic's official export flow and does not upload conversation data anywhere.

> This is an independent community project. It is not affiliated with, endorsed by, or supported by Anthropic.

## Download

Choose the installer for your operating system and processor. All files come directly from the latest [GitHub Release](https://github.com/Krimchanin/claude-vault/releases/latest).

| Platform | Recommended download | Alternative |
| --- | --- | --- |
| Windows x64 | [Setup `.exe`](https://github.com/Krimchanin/claude-vault/releases/latest/download/Claude.Vault_0.2.0_x64-setup.exe) | [`.msi`](https://github.com/Krimchanin/claude-vault/releases/latest/download/Claude.Vault_0.2.0_x64_en-US.msi) |
| Windows x86 / 32-bit | [Setup `.exe`](https://github.com/Krimchanin/claude-vault/releases/latest/download/Claude.Vault_0.2.0_x86-setup.exe) | [`.msi`](https://github.com/Krimchanin/claude-vault/releases/latest/download/Claude.Vault_0.2.0_x86_en-US.msi) |
| Windows ARM64 | [Setup `.exe`](https://github.com/Krimchanin/claude-vault/releases/latest/download/Claude.Vault_0.2.0_arm64-setup.exe) | [`.msi`](https://github.com/Krimchanin/claude-vault/releases/latest/download/Claude.Vault_0.2.0_arm64_en-US.msi) |
| macOS Apple Silicon | [`.dmg`](https://github.com/Krimchanin/claude-vault/releases/latest/download/Claude.Vault_0.2.0_aarch64.dmg) | [`.app.tar.gz`](https://github.com/Krimchanin/claude-vault/releases/latest/download/Claude.Vault_0.2.0_aarch64.app.tar.gz) |
| macOS Intel | [`.dmg`](https://github.com/Krimchanin/claude-vault/releases/latest/download/Claude.Vault_0.2.0_x64.dmg) | [`.app.tar.gz`](https://github.com/Krimchanin/claude-vault/releases/latest/download/Claude.Vault_0.2.0_x64.app.tar.gz) |
| Linux x64 | [`.AppImage`](https://github.com/Krimchanin/claude-vault/releases/latest/download/Claude.Vault_0.2.0_amd64.AppImage) | [`.deb`](https://github.com/Krimchanin/claude-vault/releases/latest/download/Claude.Vault_0.2.0_amd64.deb) · [`.rpm`](https://github.com/Krimchanin/claude-vault/releases/latest/download/Claude.Vault-0.2.0-1.x86_64.rpm) |
| Linux ARM64 | [`.AppImage`](https://github.com/Krimchanin/claude-vault/releases/latest/download/Claude.Vault_0.2.0_aarch64.AppImage) | [`.deb`](https://github.com/Krimchanin/claude-vault/releases/latest/download/Claude.Vault_0.2.0_arm64.deb) · [`.rpm`](https://github.com/Krimchanin/claude-vault/releases/latest/download/Claude.Vault-0.2.0-1.aarch64.rpm) |

The current builds are unsigned, so Windows SmartScreen or macOS Gatekeeper may display a warning. The complete source and reproducible release workflow are public in this repository.

## How it works

```text
Claude's local files
        ↓
Back up to Claude Vault
        ↓
Switch Claude accounts safely
        ↓
Restore only files that are missing
```

Claude Vault mirrors Claude's original files instead of converting conversations into a proprietary database. When restoring, it merges missing data back into the matching Claude folders and leaves every existing file untouched.

> **Private by design:** no cloud, no accounts, no analytics, no telemetry, and no network synchronization. Your raw conversations stay on your computer. Restore never overwrites an existing Claude file.

## What it does

- Detects Claude Desktop data in both classic and Microsoft Store installation locations on Windows.
- Lists locally available sessions with titles, dates, sizes, and turn counts.
- Opens the complete user/assistant transcript for sessions that have matching JSONL history.
- Creates an independent archive inside Claude Vault's own application-data directory.
- Updates archived files whose contents changed without duplicating unchanged files.
- Restores only missing files; existing Claude files are never overwritten.
- Preserves raw metadata, transcripts, attachments, and scratch-workspace files byte-for-byte.
- Supports custom paths and Russian, English, Simplified Chinese, German, Spanish, and French.

## Safety semantics

- **Backup** adds new files and updates archived copies whose contents changed.
- **Restore** only adds missing files. It never replaces an existing Claude file.
- Symbolic links are ignored while scanning and copying.
- `<system-reminder>` blocks are hidden only in the viewer. Raw JSONL is never modified.

See [docs/archive-format.md](docs/archive-format.md) for exact paths and archive layout.

## Development

Requirements: Windows 10/11, Node.js 20+, Rust stable with MSVC, WebView2, and the Tauri 2 Windows prerequisites.

```powershell
npm install
npm run tauri dev
```

Quality checks:

```powershell
npm run format:check
npm run check
cargo test --manifest-path src-tauri/Cargo.toml
```

Build Windows installers with `npm run tauri build`. Tauri writes them under `src-tauri/target/release/bundle/`.

## Privacy and limitations

- All conversation operations are local; there are no analytics, accounts, or network synchronization.
- Claude's internal formats are undocumented and may change in future releases.
- Restored sessions may require restarting Claude Desktop before they appear.
- Without a JSONL transcript, metadata can still be preserved and listed, but the full conversation cannot be displayed.
- Windows is the currently verified platform. macOS and Linux discovery is implemented but still needs broader real-device testing.

## Motivation

Claude Desktop can remove locally visible chats when the user switches accounts. Other coding clients preserve their local history across account changes, so this behavior is surprising and can make valuable work appear lost. Claude Vault gives that history an independent home and lets the user merge it back later without overwriting newer files.

## Contributing, security, and license

Read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request. Report sensitive issues according to [SECURITY.md](SECURITY.md). Licensed under the [MIT License](LICENSE).
