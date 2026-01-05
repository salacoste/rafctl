# capctl

**AI Coding Agent Profile Manager** — manage multiple accounts for Claude Code and Codex CLI with full isolation.

## Problem

You have multiple AI coding assistant accounts (personal, work, client projects). Current tools store credentials globally, making it impossible to:
- Run multiple instances with different accounts
- Switch accounts without re-authentication
- Track quota usage across accounts

## Solution

capctl provides isolated profiles with environment-based separation:

```bash
# Create profiles
capctl profile add work --tool claude
capctl profile add personal --tool claude

# Authenticate each
capctl auth login work
capctl auth login personal

# Run simultaneously in different terminals
capctl run work        # Terminal 1
capctl run personal    # Terminal 2

# Check status
capctl status
```

## How It Works

capctl uses environment variables to redirect config directories:

| Tool | ENV Variable | capctl Override |
|------|--------------|-----------------|
| Claude Code | `CLAUDE_CONFIG_DIR` | `~/.capctl/profiles/<name>/claude` |
| Codex CLI | `CODEX_HOME` | `~/.capctl/profiles/<name>/codex` |

Zero overhead — no containers, no virtualization, just environment isolation.

## Installation

```bash
# From source (requires Rust)
cargo install --path .

# Or build manually
cargo build --release
cp target/release/capctl /usr/local/bin/
```

## Commands

```bash
# Profile management
capctl profile add <name> --tool <claude|codex>
capctl profile list
capctl profile remove <name>
capctl profile show <name>

# Authentication
capctl auth login <profile>
capctl auth status <profile>
capctl auth logout <profile>

# Execution
capctl run <profile>           # Run tool with profile
capctl run                     # Run with last used profile
capctl shell <profile>         # Open shell with profile env

# Status
capctl status                  # All profiles
capctl status <profile>        # Specific profile
```

## Configuration

All data stored in `~/.capctl/`:

```
~/.capctl/
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
