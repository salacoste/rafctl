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
# Profile management
rafctl profile add <name> --tool <claude|codex>
rafctl profile add <name> --tool claude --auth-mode api-key
rafctl profile list
rafctl profile remove <name>
rafctl profile show <name>

# Authentication
rafctl auth login <profile>
rafctl auth status <profile>
rafctl auth logout <profile>
rafctl auth set-key <profile>   # For API key mode

# Execution
rafctl run <profile>            # Run tool with profile
rafctl run                      # Run with default/last used profile

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

Use `--json` or `--plain` flags for machine-readable output:

```bash
# Get authenticated profiles as JSON
rafctl status --json | jq '.profiles[] | select(.authenticated == true) | .name'

# Plain output (no colors, tab-separated)
rafctl status --plain

# Respects NO_COLOR environment variable
NO_COLOR=1 rafctl status
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
- [ ] Desktop app (Tauri)

## Development

```bash
# Build
cargo build

# Test (55 tests: 21 unit + 34 integration)
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
