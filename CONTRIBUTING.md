# Contributing

Thank you for improving Claude Vault.

1. Open an issue for substantial behavioral or archive-format changes.
2. Keep backup and restore operations local and conservative.
3. Never overwrite live Claude data during restore.
4. Run `npm run format:check`, `npm run check`, and `cargo test --manifest-path src-tauri/Cargo.toml` before submitting a pull request.
5. Explain assumptions about undocumented Claude formats in the pull request.

Never include personal conversations, real session identifiers, user paths, or archived Claude data in tests, screenshots, issues, or commits.
