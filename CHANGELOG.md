# Changelog

All notable changes to rafctl will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Security

- **API keys now stored in secure platform keyring** instead of plaintext YAML
  - macOS: Keychain
  - Linux: secret-service (libsecret/GNOME Keyring)
  - Windows: Windows Credential Manager
  - Backwards compatible: existing plaintext keys still work during migration

### Changed

- Replaced custom macOS Keychain code with cross-platform `keyring` crate
- OAuth tokens and API keys now use unified credential storage
- `Profile.api_key` field deprecated (stored in keyring instead)

### Added

- New `core::credentials` module for cross-platform credential management

## [0.2.0] - 2026-01-06

### Added

- **Quota Monitoring** (Epic 10)
  - `rafctl quota` command to check Claude API usage limits
  - Shows 5-hour and 7-day utilization windows
  - Visual progress bars with color coding (green/yellow/red)
  - Reset time display for each window
  - Supports `--json` and `--plain` output formats
  - Uses Anthropic OAuth API (`/api/oauth/usage`)

- **TUI Dashboard** (Epic 11)
  - `rafctl dashboard` command for interactive profile management
  - Table view with all profiles, auth status, and last used time
  - Keyboard navigation: `j/k` or arrows, `Enter/r` to run, `l` to login, `q` to quit
  - Visual selection with highlight indicator
  - Built with ratatui + crossterm

### Dependencies

- Added `ureq` for HTTP requests (quota API)
- Added `ratatui` and `crossterm` for TUI

## [0.1.0] - 2026-01-06

### Added

- **Profile Management**
  - Create, list, show, and remove profiles
  - Case-insensitive profile names with validation
  - "Did you mean?" suggestions for typos

- **Tool Support**
  - Claude Code with `CLAUDE_CONFIG_DIR` isolation
  - Codex CLI with `CODEX_HOME` isolation

- **Authentication**
  - OAuth mode: Token swapping via macOS Keychain (single instance)
  - API Key mode: `ANTHROPIC_API_KEY` env var (parallel instances)
  - `auth login`, `auth logout`, `auth status`, `auth set-key` commands
  - OAuth lockfile to prevent race conditions

- **Execution**
  - `rafctl run <profile>` to launch tools with isolated config
  - `rafctl run` uses default/last-used profile
  - Automatic `last_used` tracking

- **Configuration**
  - Global config at `~/.rafctl/config.yaml`
  - `config set-default`, `config clear-default`, `config path`, `config show`
  - Priority chain: CLI > ENV > config > last_used

- **Output Formats**
  - `--json` flag for machine-readable output
  - `--plain` flag for scripting (no colors/emoji)
  - `NO_COLOR` environment variable support

- **Shell Completions**
  - `rafctl completion bash/zsh/fish`

- **Status Dashboard**
  - `rafctl status` shows all profiles in a table
  - Authentication status, last used time, default marker

### Technical

- 22 unit tests, 34 integration tests
- CI/CD with GitHub Actions (check, fmt, clippy, test)
- Cross-platform builds (Linux x86_64, macOS x86_64/aarch64)
- Release workflow with automatic artifact creation

[Unreleased]: https://github.com/salacoste/rafctl/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/salacoste/rafctl/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/salacoste/rafctl/releases/tag/v0.1.0
