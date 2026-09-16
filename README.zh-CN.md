# Claude Vault

[English](README.md) · [Русский](README.ru.md) · **简体中文** · [Deutsch](README.de.md) · [Español](README.es.md) · [Français](README.fr.md)

**切换 Claude 账户，同时保留聊天记录。** Claude Vault 是一款非官方、本地优先的桌面工具，可直接从文件系统查看、备份和恢复 Claude Desktop / Claude Code 会话数据。

Claude Desktop 在切换账户时可能清除本地可见的聊天。Claude Vault 会将会话元数据、完整 JSONL 记录、附件和工作区文件保存到独立仓库，并可在之后安全地补回缺失文件。恢复时绝不会覆盖现有 Claude 文件。

它不使用官方导出、不需要账户，也不会上传数据。支持 Windows、macOS 和 Linux，并可在设置中自定义路径。

开发：运行 `npm install` 和 `npm run tauri dev`。检查：`npm run check` 以及 `cargo test --manifest-path src-tauri/Cargo.toml`。归档格式见 [docs/archive-format.md](docs/archive-format.md)。

本项目与 Anthropic 无关联，采用 MIT 许可证。
