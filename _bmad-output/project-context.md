# Project Context — capctl

**Last Updated:** 2025-01-05

---

## Project Summary

**capctl** (Coding Agent Profile Controller) is a CLI utility for managing multiple AI coding assistant profiles with full environment isolation.

| Attribute | Value |
|-----------|-------|
| Language | Rust |
| Type | CLI utility |
| Target Users | Developers using Claude Code / Codex CLI |
| Core Problem | Cannot run multiple accounts simultaneously |
| Solution | ENV-based config directory isolation |

---

## Key Technical Decisions

### Environment Isolation (CRITICAL)

The entire architecture relies on environment variable overrides:

```bash
# Claude Code
CLAUDE_CONFIG_DIR=~/.capctl/profiles/<name>/claude claude

# Codex CLI  
CODEX_HOME=~/.capctl/profiles/<name>/codex codex
```

**Never** modify global config files. **Always** use isolated directories.

### Directory Structure

```
~/.capctl/
├── config.yaml                 # Global config (default_profile, settings)
├── profiles/
│   └── <profile-name>/
│       ├── meta.yaml           # Profile metadata
│       └── claude/ or codex/   # Tool-specific config
└── cache/
    └── quotas.json             # Cached quota data
```

### Default Profile Behavior

- Track `last_used` timestamp in global config
- `capctl run` without args uses last used profile
- Auto-generated names: `profile-YYYYMMDD-HHMMSS`

---

## Code Patterns (MUST FOLLOW)

### Error Handling
```rust
// Library code: thiserror
#[derive(Debug, thiserror::Error)]
pub enum CapctlError {
    #[error("Profile '{0}' not found")]
    ProfileNotFound(String),
}

// Application code: anyhow
fn main() -> anyhow::Result<()> { }
```

### Path Handling
```rust
// ALWAYS use PathBuf, NEVER String for paths
use std::path::PathBuf;
use dirs::home_dir;

let base = home_dir().ok_or(CapctlError::NoHomeDir)?;
let config = base.join(".capctl").join("config.yaml");
```

### CLI Output
```rust
use colored::Colorize;

// Success
println!("{} Profile '{}' created", "✓".green(), name);

// Error
eprintln!("{} {}", "Error:".red().bold(), msg);

// Tables: use comfy-table
```

---

## Forbidden Patterns

| Pattern | Why | Instead |
|---------|-----|---------|
| `unwrap()` | Panics in production | Use `?` or `ok_or()` |
| `expect()` without context | Unclear errors | Add descriptive message |
| Hardcoded paths | Breaks cross-platform | Use `dirs` + `PathBuf` |
| `println!` for errors | Goes to stdout | Use `eprintln!` |
| String paths | Not cross-platform | Use `PathBuf` |

---

## Testing Requirements

1. **Unit tests** in same file (`#[cfg(test)]`)
2. **Integration tests** use `tempfile::TempDir` for isolation
3. **All tests must pass** before any PR
4. Run full suite: `cargo test`
5. Run single test: `cargo test <test_name>`

---

## CLI Command Structure

```
capctl
├── profile
│   ├── add <name> --tool <claude|codex>
│   ├── list
│   ├── remove <name>
│   └── show <name>
├── auth
│   ├── login <profile>
│   ├── logout <profile>
│   └── status [profile]
├── run [profile] [-- args...]
├── shell <profile>
└── status [profile]
```

---

## Dependencies

```toml
# Core
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"

# Error handling
thiserror = "1"
anyhow = "1"

# CLI output
colored = "2"
comfy-table = "7"

# System
dirs = "5"
chrono = { version = "0.4", features = ["serde"] }
```

---

## Quick Reference

| Action | Command |
|--------|---------|
| Build | `cargo build` |
| Test | `cargo test` |
| Single test | `cargo test test_name` |
| Lint | `cargo clippy -- -D warnings` |
| Format | `cargo fmt` |
| Full check | `cargo fmt --check && cargo clippy -- -D warnings && cargo test` |

---

## Open Decisions (TBD)

1. Quota monitoring implementation (parse /status vs API)
2. Future tool support (Cursor, Aider, Continue)
3. Team/enterprise features scope
