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

## Installation

```bash
# From source (requires Rust)
cargo install --path .

# Or build manually
cargo build --release
cp target/release/rafctl /usr/local/bin/
```

## Commands

```bash
# Profile management
rafctl profile add <name> --tool <claude|codex>
rafctl profile list
rafctl profile remove <name>
rafctl profile show <name>

# Authentication
rafctl auth login <profile>
rafctl auth status <profile>
rafctl auth logout <profile>

# Execution
rafctl run <profile>           # Run tool with profile
rafctl run                     # Run with last used profile
rafctl shell <profile>         # Open shell with profile env

# Status
rafctl status                  # All profiles
rafctl status <profile>        # Specific profile
```

## Configuration

All data stored in `~/.rafctl/`:

```
~/.rafctl/
├── config.yaml           # Global settings
├── profiles/
│   ├── work/
│   │   ├── meta.yaml     # Profile metadata
│   │   └── claude/       # Claude config for this profile
│   └── personal/
│       ├── meta.yaml
│       └── claude/
└── cache/
    └── quotas.json       # Cached quota data
```

## Roadmap

- [x] Profile management
- [x] Authentication flow
- [x] Isolated execution
- [ ] Quota monitoring
- [ ] TUI dashboard
- [ ] Shell completions
- [ ] Desktop app (Tauri)

## Development

```bash
# Build
cargo build

# Test
cargo test

# Lint
cargo clippy

# Format
cargo fmt
```

See [AGENTS.md](./AGENTS.md) for detailed development guidelines.

## License

MIT
