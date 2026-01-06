# Project Context — rafctl

**Last Updated:** 2026-01-06

---

## Project Summary

**rafctl** (Coding Agent Profile Controller) is a CLI utility for managing multiple AI coding assistant profiles with full environment isolation.

| Attribute | Value |
|-----------|-------|
| Language | Rust 2021 Edition |
| Type | CLI utility (single binary) |
| Target Users | Developers using Claude Code / Codex CLI |
| Core Problem | Cannot run multiple accounts simultaneously |
| Solution | ENV-based config directory isolation |

---

## Technology Stack (EXACT VERSIONS)

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
clap_complete = "4"
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
thiserror = "1"
anyhow = "1"
colored = "2"
comfy-table = "7"
dirs = "5"
chrono = { version = "0.4", features = ["serde"] }

[dev-dependencies]
tempfile = "3"
assert_cmd = "2"
predicates = "3"
```

---

## Critical Implementation Rules

### Environment Isolation (CORE MECHANISM)

```bash
# Claude Code
CLAUDE_CONFIG_DIR=~/.rafctl/profiles/<name>/claude claude

# Codex CLI  
CODEX_HOME=~/.rafctl/profiles/<name>/codex codex
```

**NEVER** modify global config files. **ALWAYS** use isolated directories.

### Directory Structure

```
~/.rafctl/                           # 700 permissions (Unix)
├── config.yaml                      # Global settings
└── profiles/
    └── <profile-name>/
        ├── meta.yaml                # Profile metadata (600 permissions)
        └── <tool>/                  # Tool config dir (managed by tool)
```

### Atomic File Writes (MANDATORY)

```rust
// ALWAYS use temp file + rename pattern
let tmp_path = path.with_extension("yaml.tmp");
std::fs::write(&tmp_path, content)?;
std::fs::rename(&tmp_path, &path)?;
```

### File Permissions (Unix only)

```rust
#[cfg(unix)]
{
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))?;
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600))?;
}
// Windows: no-op (permissions not applicable)
```

---

## Code Patterns (MUST FOLLOW)

### Error Handling

```rust
// Library code (core/, tools/): thiserror
#[derive(Debug, thiserror::Error)]
pub enum RafctlError {
    #[error("Profile '{0}' not found")]
    ProfileNotFound(String),
    
    #[error("Failed to read config '{path}'")]
    ConfigRead { 
        path: PathBuf,
        #[source] source: std::io::Error,
    },
}

// Application code (main.rs, cli/): anyhow
pub fn run() -> anyhow::Result<()> { }
```

**RULE:** Include path/context in ALL IO errors.

### Path Handling

```rust
// ALWAYS use PathBuf, NEVER String for paths
use std::path::PathBuf;
use dirs::home_dir;

let base = home_dir().ok_or(RafctlError::NoHomeDir)?;
let config = base.join(".rafctl").join("config.yaml");
```

### CLI Output

```rust
use colored::Colorize;

// Success: ✓ green
println!("{} Profile '{}' created", "✓".green(), name);

// Warning: ⚠ yellow  
println!("{} Auth may need refresh", "⚠".yellow());

// Error: ✗ red
eprintln!("{} Profile not found", "✗".red());

// Tables: use comfy-table
use comfy_table::Table;
```

**Respects NO_COLOR env var automatically.**

### Naming Conventions

| Element | Convention | Example |
|---------|------------|---------|
| Modules/files | snake_case | `profile_manager.rs` |
| Types/structs | PascalCase | `ProfileConfig` |
| Functions | snake_case | `create_profile()` |
| Constants | SCREAMING_SNAKE_CASE | `MAX_PROFILE_NAME_LENGTH` |
| Error variants | Noun-first | `ProfileNotFound` |
| Bool functions | `is_*` prefix | `is_authenticated()` |
| YAML keys | snake_case | `default_profile` |

### Import Order

```rust
// 1. std library
use std::path::PathBuf;

// 2. External crates
use clap::Parser;
use serde::{Deserialize, Serialize};

// 3. Crate modules
use crate::config::Config;
use crate::profile::Profile;
```

---

## Architectural Boundaries

### Layer Dependencies (UNIDIRECTIONAL)

```
cli/ ──▶ core/ ──▶ tools/ ──▶ error.rs
```

| Layer | Can Import From |
|-------|-----------------|
| `cli/` | `core/`, `tools/`, `error` |
| `core/` | `tools/`, `error` |
| `tools/` | `error` only |
| `error` | nothing (leaf) |

### Module Responsibilities

| Module | Responsibility |
|--------|----------------|
| `cli/` | Parse args, format output, orchestrate |
| `core/` | Business logic, file I/O, validation |
| `tools/` | Tool-specific config (ENV vars, paths) |
| `error` | Error types, messages, suggestions |

---

## Forbidden Patterns (NEVER DO)

```rust
// ❌ unwrap() in production code
let config = fs::read_to_string(path).unwrap();

// ❌ expect() without good reason
let home = home_dir().expect("no home");

// ❌ String for paths
fn load_config(path: String) -> Result<Config> { }

// ❌ IO errors without context
Err(e) => Err(e.into())

// ❌ Debug prints left in code
dbg!(value);
println!("DEBUG: {}", x);

// ❌ as any / type suppression (if using TypeScript anywhere)
// ❌ Empty catch blocks
```

### Instead Use

```rust
// ✅ ? operator with context
let config = fs::read_to_string(&path)
    .map_err(|e| RafctlError::ConfigRead { path: path.clone(), source: e })?;

// ✅ PathBuf for all paths
fn load_config(path: &Path) -> Result<Config> { }

// ✅ ok_or for Option -> Result
let home = home_dir().ok_or(RafctlError::NoHomeDir)?;
```

---

## Profile Name Validation

```rust
// Pattern: alphanumeric, underscore, hyphen only
const PROFILE_NAME_PATTERN: &str = r"^[a-zA-Z0-9_-]+$";
const MAX_PROFILE_NAME_LENGTH: usize = 64;

// Comparison: case-insensitive
// "Work" == "work"
```

---

## Testing Requirements

### Unit Tests
- In same file under `#[cfg(test)]`
- Use `Arrange / Act / Assert` pattern

### Integration Tests
- Use `tempfile::TempDir` for filesystem isolation
- CRITICAL: Isolation test must pass (two profiles don't share config)

### Test Commands
```bash
cargo test                     # All tests
cargo test <test_name>         # Single test
cargo test -- --nocapture      # With stdout
```

### Before Every Commit
```bash
cargo fmt --check && cargo clippy -- -D warnings && cargo test
```

---

## CI/CD Pipeline

```yaml
# On push/PR
- cargo fmt --check
- cargo clippy -- -D warnings
- cargo test

# Matrix: ubuntu-latest, macos-latest

# On tag (vX.Y.Z)
- Build 4 targets:
  - x86_64-unknown-linux-gnu
  - x86_64-apple-darwin
  - aarch64-apple-darwin
  - x86_64-pc-windows-msvc
- Create GitHub Release with binaries
```

---

## Key Data Structures

```rust
pub struct Profile {
    pub name: String,
    pub tool: ToolType,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
}

pub enum ToolType {
    Claude,  // CLAUDE_CONFIG_DIR
    Codex,   // CODEX_HOME
}

pub enum StatusIndicator {
    Active,   // ✓ green
    Warning,  // ⚠ yellow (last_used > 7 days)
    Error,    // ✗ red
    Unknown,  // ? gray
}
```

---

## Auth Approach

- **Spawn tool with ENV isolation, wait for exit**
- **Check credential file existence** after tool exits
- **NEVER read credential contents into memory**
- **No pre-check before `run`** (let tool handle expired tokens)
- **Timeout:** 10 minutes default, configurable

---

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid arguments / usage error |
| 3 | Profile not found |
| 4 | Authentication error |
| 5 | Tool execution error |

---

## Quick Reference

| Action | Command |
|--------|---------|
| Build | `cargo build` |
| Build release | `cargo build --release` |
| Test | `cargo test` |
| Single test | `cargo test test_name` |
| Lint | `cargo clippy -- -D warnings` |
| Format | `cargo fmt` |
| Full check | `cargo fmt --check && cargo clippy -- -D warnings && cargo test` |
| Docs | `cargo doc --open` |

---

## Implementation Sequence

1. Project scaffolding (Cargo.toml, directory structure)
2. CLI skeleton with clap (all subcommands stub)
3. Core data structures (Profile, Config)
4. File I/O with atomic writes
5. Tool abstraction (Claude, Codex)
6. ENV isolation mechanism
7. **Isolation tests (CRITICAL)** — validates core mechanism
8. Command implementations
9. Integration tests
10. CI/CD setup
11. Release workflow
