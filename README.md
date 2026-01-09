# rafctl

**AI Coding Agent Profile Manager** — manage multiple accounts for Claude Code and Codex CLI with full isolation.

## Problem

You have multiple AI coding assistant accounts (personal, work, client projects). Current tools store credentials globally, making it impossible to:
- Run multiple instances with different accounts
- Switch accounts without re-authentication
- Track quota usage across accounts

## Solution

rafctl provides isolated profiles with environment-based separation:

```bash
# Create profiles
rafctl profile add work --tool claude
rafctl profile add personal --tool claude

# Authenticate each
rafctl auth login work
rafctl auth login personal

# Run simultaneously in different terminals
rafctl run work        # Terminal 1
rafctl run personal    # Terminal 2

# Check status
rafctl status
```

## How It Works

rafctl uses environment variables to redirect config directories:

| Tool | ENV Variable | rafctl Override |
|------|--------------|-----------------|
| Claude Code | `CLAUDE_CONFIG_DIR` | `~/.rafctl/profiles/<name>/claude` |
| Codex CLI | `CODEX_HOME` | `~/.rafctl/profiles/<name>/codex` |

Zero overhead — no containers, no virtualization, just environment isolation.

### Authentication Modes

| Mode | Parallel Instances | Use Case |
|------|-------------------|----------|
| **OAuth** (default) | ❌ Single instance | Subscription features, no API key needed |
| **API Key** | ✅ Unlimited | Full isolation, requires API key |

```bash
# Create OAuth profile (default)
rafctl profile add work --tool claude

# Create API Key profile for parallel execution
rafctl profile add parallel --tool claude --auth-mode api-key
rafctl auth set-key parallel
```

## Installation

### From Source

```bash
# Requires Rust 1.70+
cargo install --path .

# Or build manually
cargo build --release
cp target/release/rafctl /usr/local/bin/
```

### From Releases

Download pre-built binaries from [GitHub Releases](https://github.com/salacoste/rafctl/releases).

## Commands

```bash
# Profile management (supports aliases: w -> work, p -> personal, etc.)
rafctl profile add <name> --tool <claude|codex>
rafctl profile add <name> --tool claude --auth-mode api-key
rafctl profile list
rafctl profile remove <name>           # Asks for confirmation
rafctl profile remove <name> --yes     # Skip confirmation
rafctl profile remove <name> --dry-run # See what would be removed
rafctl profile show <name>

# Authentication
rafctl auth login <profile>
rafctl auth status <profile>
rafctl auth logout <profile>
rafctl auth logout <profile> --dry-run  # See what would be cleared
rafctl auth set-key <profile>   # For API key mode

# Execution
rafctl run <profile>            # Run tool with profile (or alias: rafctl run w)
rafctl run                      # Run with default/last used profile
rafctl switch <profile>         # Set as default and show status
rafctl env <profile>            # Export environment variables for manual use

# Configuration
rafctl config show              # Show current config
rafctl config set-default <p>   # Set default profile
rafctl config clear-default     # Clear default profile
rafctl config path              # Show config directory

# Status
rafctl status                   # All profiles (table view)
rafctl status <profile>         # Specific profile details
rafctl status --json            # JSON output for scripting

# Quota Monitoring
rafctl quota                    # Show quota for all profiles
rafctl quota <profile>          # Show quota for specific profile

# Usage Analytics
rafctl analytics                # Show usage stats for default profile
rafctl analytics --all          # Show all profiles
rafctl analytics --cost         # Show estimated costs
rafctl analytics --days 30      # Custom time range

# Session Monitoring
rafctl sessions                 # List recent sessions
rafctl sessions --today         # Today's sessions only
rafctl sessions <session-id>    # Session details
rafctl watch                    # Watch live session in real-time

# TUI Dashboard
rafctl dashboard                # Interactive profile management
```

## Shell Completions

Generate shell completions for your shell:

```bash
# Bash
rafctl completion bash > ~/.local/share/bash-completion/completions/rafctl

# Zsh (add to ~/.zshrc or create completion file)
rafctl completion zsh > ~/.zfunc/_rafctl
# Then add to .zshrc: fpath=(~/.zfunc $fpath) && autoload -Uz compinit && compinit

# Fish
rafctl completion fish > ~/.config/fish/completions/rafctl.fish
```

## Configuration

All data stored in `~/.rafctl/`:

```
~/.rafctl/
├── config.yaml           # Global settings (default profile, etc.)
├── oauth.lock            # Lock file for OAuth mode
├── profiles/
│   ├── work/
│   │   ├── meta.yaml     # Profile metadata
│   │   └── claude/       # Claude config for this profile
│   └── personal/
│       ├── meta.yaml
│       └── claude/
```

## Scripting

Use `--json`, `--plain`, or `--verbose` flags for different output modes:

```bash
# Get authenticated profiles as JSON
rafctl status --json | jq '.profiles[] | select(.authenticated == true) | .name'

# Plain output (no colors, tab-separated)
rafctl status --plain

# Verbose debug output
rafctl run work --verbose  # Shows env vars, config paths, auth mode

# Respects NO_COLOR environment variable
NO_COLOR=1 rafctl status
```

## Profile Aliases

All commands support profile name aliases for faster typing:

```bash
# Create profiles
rafctl profile add work --tool claude
rafctl profile add personal --tool claude

# Use aliases (prefix match)
rafctl run w              # Runs 'work'
rafctl auth login p       # Logs into 'personal'
rafctl profile show w     # Shows 'work' details

# Exact matches take precedence
rafctl profile add w --tool claude  # Creates profile literally named 'w'
rafctl run w              # Now runs 'w', not 'work'
```

## Shell Environment Export

Export rafctl environment variables for manual tool invocation:

```bash
# Export environment for a profile
eval $(rafctl env work)

# Now you can run the tool directly
claude  # Will use work profile's config

# Check what would be exported
rafctl env work
# export CLAUDE_CONFIG_DIR="/Users/you/.rafctl/profiles/work/claude"
# export RAFCTL_PROFILE="work"
# export RAFCTL_PROFILE_TOOL="claude"
# export RAFCTL_VERSION="0.6.0"
```

## Troubleshooting

### "OAuth mode conflict: another OAuth instance is already running"

OAuth profiles swap tokens in macOS Keychain, so only one can run at a time.

**Solutions:**
1. Close the other OAuth instance first
2. Use API Key mode for parallel execution:
   ```bash
   rafctl profile add parallel --tool claude --auth-mode api-key
   rafctl auth set-key parallel
   ```

### "Profile not found"

Profile names are case-insensitive. rafctl will suggest similar names if found.

```bash
rafctl profile list  # See all available profiles
```

### "Tool not found"

The underlying tool (claude, codex) isn't installed.

```bash
# Install Claude Code
npm install -g @anthropic-ai/claude-code

# Install Codex CLI
pip install openai-codex
```

### "Profile is not authenticated"

Run the auth flow for the profile:

```bash
rafctl auth login <profile>
```

### Reserved profile names

These names are reserved and cannot be used: `default`, `config`, `cache`, `profiles`, `oauth`

## Roadmap

- [x] Profile management
- [x] Authentication flow (OAuth + API Key)
- [x] Isolated execution
- [x] Shell completions
- [x] JSON/Plain output formats
- [x] CI/CD pipeline
- [x] Quota monitoring
- [x] TUI dashboard
- [x] Usage analytics (v0.3.0)
- [x] Session monitoring (v0.4.0)
- [x] Native HUD plugin (v0.4.0)
- [x] Secure keyring storage (v0.5.0)
- [x] Developer experience improvements (v0.6.0)
- [ ] Desktop app (Tauri)

## Documentation

See the [docs/](./docs/) folder for detailed guides:

- [Quota Monitoring](./docs/quota-monitoring.md) - Track Claude API usage limits
- [TUI Dashboard](./docs/dashboard.md) - Interactive terminal interface
- [Sessions & Analytics](./docs/sessions.md) - Usage analytics and session monitoring
- [HUD Statusline](./docs/hud.md) - Native statusline plugin for Claude Code

## Development

```bash
# Build
cargo build

# Test (95 tests: 61 unit + 34 integration)
cargo test

# Lint
cargo clippy -- -D warnings

# Format
cargo fmt

# Full check (CI equivalent)
cargo fmt --check && cargo clippy -- -D warnings && cargo test
```

See [AGENTS.md](./AGENTS.md) for detailed development guidelines.

## License

MIT
