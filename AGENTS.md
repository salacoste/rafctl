# AGENTS.md — rafctl

Guidelines for AI coding agents working on this repository.

---

## Project Overview

**rafctl** is a CLI profile manager for AI coding agents (Claude Code, OpenAI Codex CLI).

| Aspect | Details |
|--------|---------|
| Language | Rust |
| Type | CLI utility |
| Target | Developers using multiple AI coding assistant accounts |
| Key Mechanism | ENV-based isolation (`CLAUDE_CONFIG_DIR`, `CODEX_HOME`) |

---

## Build & Test Commands

```bash
# Build
cargo build                    # Debug build
cargo build --release          # Release build

# Run
cargo run -- <args>            # Run with arguments
cargo run -- profile list      # Example: list profiles

# Test
cargo test                     # Run all tests
cargo test <test_name>         # Run single test
cargo test -- --nocapture      # Run tests with stdout

# Lint & Format
cargo fmt                      # Format code
cargo fmt -- --check           # Check formatting (CI)
cargo clippy                   # Lint
cargo clippy -- -D warnings    # Lint, fail on warnings (CI)

# Documentation
cargo doc --open               # Generate and open docs
```

---

## Code Style Guidelines

### Rust Conventions

**Formatting:**
- Use `rustfmt` defaults (run `cargo fmt` before commits)
- Max line length: 100 characters
- Use 4-space indentation

**Naming:**
```rust
// Modules and files: snake_case
mod profile_manager;

// Types, traits, enums: PascalCase
struct ProfileConfig { }
enum ToolType { Claude, Codex }

// Functions, methods, variables: snake_case
fn create_profile(name: &str) -> Result<Profile> { }

// Constants: SCREAMING_SNAKE_CASE
const DEFAULT_CONFIG_DIR: &str = ".rafctl";

// Lifetimes: short lowercase ('a, 'b)
fn parse<'a>(input: &'a str) -> &'a str { }
```

**Imports:**
```rust
// Order: std → external crates → crate modules
use std::path::PathBuf;

use clap::Parser;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::profile::Profile;
```

### Error Handling

```rust
// Use thiserror for custom errors
#[derive(Debug, thiserror::Error)]
pub enum RafctlError {
    #[error("Profile '{0}' not found")]
    ProfileNotFound(String),
    
    #[error("Failed to read config: {0}")]
    ConfigError(#[from] std::io::Error),
}

// Use anyhow::Result in main/commands for convenience
pub fn run() -> anyhow::Result<()> { }

// NEVER use unwrap() in production code
// NEVER use expect() without a good reason
// Use ? operator for error propagation
```

### CLI Structure (clap)

```rust
/// rafctl - AI Coding Agent Profile Manager
#[derive(Parser)]
#[command(name = "rafctl", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage profiles
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },
    /// Authentication commands
    Auth { /* ... */ },
    /// Run a tool with a profile
    Run { profile: String },
    /// Show status of all profiles
    Status { profile: Option<String> },
}
```

---

## Project Structure

```
rafctl/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry point, CLI parsing
│   ├── lib.rs               # Library exports
│   ├── cli/                 # CLI command handlers
│   │   ├── mod.rs
│   │   ├── profile.rs       # profile add/list/remove
│   │   ├── auth.rs          # auth login/logout/status
│   │   ├── run.rs           # run command
│   │   └── status.rs        # status command
│   ├── core/                # Core business logic
│   │   ├── mod.rs
│   │   ├── profile.rs       # Profile struct and operations
│   │   ├── config.rs        # Global config management
│   │   └── isolation.rs     # ENV isolation logic
│   ├── tools/               # Tool-specific implementations
│   │   ├── mod.rs
│   │   ├── claude.rs        # Claude Code specifics
│   │   └── codex.rs         # Codex CLI specifics
│   └── error.rs             # Custom error types
├── tests/                   # Integration tests
│   ├── profile_tests.rs
│   └── integration.rs
└── _bmad-output/            # BMAD artifacts (gitignored)
```

---

## Key Abstractions

### Profile
```rust
pub struct Profile {
    pub name: String,
    pub tool: ToolType,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
}
```

### ToolType
```rust
pub enum ToolType {
    Claude,  // Uses CLAUDE_CONFIG_DIR
    Codex,   // Uses CODEX_HOME
}
```

### Environment Isolation
```rust
// Core mechanism - set env vars before spawning tool
fn get_env_overrides(profile: &Profile) -> HashMap<String, String> {
    let config_dir = get_profile_config_dir(profile);
    match profile.tool {
        ToolType::Claude => [("CLAUDE_CONFIG_DIR", config_dir)],
        ToolType::Codex => [("CODEX_HOME", config_dir)],
    }
}
```

---

## Important Patterns

### 1. Path Handling
```rust
// Always use dirs crate for home directory
use dirs::home_dir;

// Always use PathBuf for paths, not String
let config_path: PathBuf = home_dir()
    .expect("Home directory not found")
    .join(".rafctl")
    .join("config.yaml");
```

### 2. Config Files
- Global config: `~/.rafctl/config.yaml`
- Profile metadata: `~/.rafctl/profiles/<name>/meta.yaml`
- Use serde with YAML for all config files

### 3. Output Formatting
```rust
// Use colored crate for terminal colors
use colored::Colorize;

println!("{} Profile '{}' created", "✓".green(), name);
println!("{} Profile not found", "✗".red());

// Use comfy-table for tabular output
use comfy_table::Table;
```

---

## Testing Guidelines

```rust
// Unit tests in same file
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_profile_creation() {
        // Arrange
        let name = "test-profile";
        
        // Act
        let profile = Profile::new(name, ToolType::Claude);
        
        // Assert
        assert_eq!(profile.name, name);
    }
}

// Integration tests use temp directories
use tempfile::TempDir;

#[test]
fn test_profile_persistence() {
    let temp = TempDir::new().unwrap();
    // Test with temp.path() as config root
}
```

---

## CRITICAL Rules

1. **Never hardcode paths** — Use `dirs::home_dir()` and `PathBuf::join()`
2. **Never use `unwrap()` in production** — Use `?` operator or proper error handling
3. **Never suppress errors silently** — Log or propagate all errors
4. **Always run `cargo fmt` and `cargo clippy`** before commits
5. **Never commit with failing tests**
6. **Cross-platform paths** — Use `std::path::MAIN_SEPARATOR` or `PathBuf`

---

## Dependencies (Cargo.toml)

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
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

## Git Workflow

- Main branch: `main`
- Feature branches: `feat/<feature-name>`
- Bugfix branches: `fix/<bug-name>`
- Commit messages: Conventional Commits format
  - `feat: add profile list command`
  - `fix: handle missing config directory`
  - `docs: update AGENTS.md`

---

## Quick Reference

| Task | Command |
|------|---------|
| Build debug | `cargo build` |
| Build release | `cargo build --release` |
| Run tests | `cargo test` |
| Single test | `cargo test test_name` |
| Format | `cargo fmt` |
| Lint | `cargo clippy` |
| Check all | `cargo fmt --check && cargo clippy -- -D warnings && cargo test` |
