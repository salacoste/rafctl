# Changelog

All notable changes to rafctl will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0] - 2026-01-08

### Security

- **API keys now stored in secure platform keyring** instead of plaintext YAML
  - macOS: Keychain
  - Linux: secret-service (libsecret/GNOME Keyring)
  - Windows: Windows Credential Manager
  - Backwards compatible: existing plaintext keys still work during migration
- **API key input now hidden** — uses `rpassword` crate to mask input during `auth set-key`

### Added

- New `rafctl switch <profile>` command — set profile as default and show its status in one command
- New `core::credentials` module for cross-platform credential management
- New `core::constants` module for centralized configuration constants
- New `--yes` / `-y` flag for `profile remove` to skip confirmation prompt
- New `ProcessSpawn` and `NoDefaultProfile` error variants for clearer error messages
- Confirmation prompt before removing profiles (safety feature)

### Changed

- Replaced custom macOS Keychain code with cross-platform `keyring` crate
- OAuth tokens and API keys now use unified credential storage
- `Profile.api_key` field deprecated (stored in keyring instead)
- Dashboard `r`/`Enter` and `l` keys now execute actions after exiting TUI (run profile, login)
- Refactored `handle_run` — extracted `spawn_tool()`, `build_rafctl_env()`, `execute_command()`
- API requests now use 30-second timeout (prevents hanging on network issues)
- Global config now uses atomic writes (prevents corruption on interrupted writes)
- User-Agent header now uses dynamic version from Cargo.toml

## [0.4.0] - 2026-01-07

### Added

- **Session Monitoring** (Epic 14)
  - `rafctl sessions` command to list recent sessions with duration, messages, tools, errors
  - `rafctl sessions <id>` for detailed session breakdown with tool usage stats
  - `rafctl sessions --today` to filter today's sessions only
  - `rafctl watch` for live real-time session monitoring with file tailing
  - New `core::transcript` module for parsing Claude Code JSONL transcripts
  - Agent/Task call tracking with subagent types and durations

- **Native Rust HUD Plugin** (Epic 15)
  - `rafctl-hud` standalone binary implementing Claude Code's statusLine protocol
  - Profile-aware display showing active rafctl profile in statusline
  - Context window percentage with color-coded progress bar (green/yellow/red)
  - Git branch, session duration, tool/agent counts display
  - `rafctl hud install [profile]` to configure HUD for a profile
  - `rafctl hud uninstall` to remove HUD configuration
  - `rafctl hud status` to check HUD installation status
  - Zero external dependencies (no Node.js required)

### Technical

- Added `notify` crate for filesystem watching (live session monitor)
- Separate binary target for `rafctl-hud` (src/bin/rafctl-hud.rs)
- New `hud` module with stdin protocol parser and renderer

## [0.3.0] - 2026-01-07

### Added

- **Usage Analytics** (Epic 12)
  - `rafctl analytics` command to display historical usage statistics
  - Parses Claude Code's local `stats-cache.json` (no API calls required)
  - Shows daily activity: messages, sessions, tool calls, tokens
  - Model usage breakdown with visual progress bars
  - `rafctl analytics --all` for cross-profile comparison
  - `rafctl analytics --cost` for estimated API costs
  - `rafctl analytics --days N` to customize time window
  - Supports `--json` and `--plain` output formats

- **Profile-Aware HUD** (Epic 13)
  - Profile environment injection: `RAFCTL_PROFILE`, `RAFCTL_PROFILE_TOOL`, `RAFCTL_VERSION`
  - Terminal title shows active profile: `[rafctl:work] claude`
  - Third-party HUD plugins can read profile from environment

### Technical

- New `core::stats` module for stats-cache.json parsing
- Cost estimation with configurable pricing per model

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

[Unreleased]: https://github.com/salacoste/rafctl/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/salacoste/rafctl/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/salacoste/rafctl/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/salacoste/rafctl/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/salacoste/rafctl/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/salacoste/rafctl/releases/tag/v0.1.0
