//! Centralized constants for rafctl.
//!
//! This module contains all magic strings and configuration values
//! used throughout the application.

// =============================================================================
// Directory & File Names
// =============================================================================

/// Default rafctl configuration directory name (under home)
pub const RAFCTL_DIR_NAME: &str = ".rafctl";

/// Profile metadata filename
pub const PROFILE_META_FILE: &str = "meta.yaml";

/// Global config filename
pub const GLOBAL_CONFIG_FILE: &str = "config.yaml";

/// Stats cache filename (Claude Code)
pub const STATS_CACHE_FILE: &str = "stats-cache.json";

/// Transcripts directory name
pub const TRANSCRIPTS_DIR: &str = "transcripts";

// =============================================================================
// Environment Variables
// =============================================================================

/// Override for rafctl config directory
pub const ENV_RAFCTL_CONFIG_DIR: &str = "RAFCTL_CONFIG_DIR";

/// Default profile override
pub const ENV_RAFCTL_DEFAULT_PROFILE: &str = "RAFCTL_DEFAULT_PROFILE";

/// Active profile name (set when running tools)
pub const ENV_RAFCTL_PROFILE: &str = "RAFCTL_PROFILE";

/// Active profile tool type (set when running tools)
pub const ENV_RAFCTL_PROFILE_TOOL: &str = "RAFCTL_PROFILE_TOOL";

/// rafctl version (set when running tools)
pub const ENV_RAFCTL_VERSION: &str = "RAFCTL_VERSION";

/// Anthropic API key environment variable
pub const ENV_ANTHROPIC_API_KEY: &str = "ANTHROPIC_API_KEY";

/// Claude config directory environment variable
pub const ENV_CLAUDE_CONFIG_DIR: &str = "CLAUDE_CONFIG_DIR";

/// Codex home environment variable
pub const ENV_CODEX_HOME: &str = "CODEX_HOME";

// =============================================================================
// Keychain / Credentials
// =============================================================================

/// Service prefix for keyring entries
pub const KEYRING_SERVICE_PREFIX: &str = "rafctl-profile-";

/// macOS Keychain service name for Claude OAuth
pub const CLAUDE_KEYCHAIN_SERVICE: &str = "claude.ai";

// =============================================================================
// API Configuration
// =============================================================================

/// Anthropic OAuth usage API endpoint
pub const ANTHROPIC_USAGE_API: &str = "https://api.anthropic.com/api/oauth/usage";

/// API request timeout in seconds
pub const API_TIMEOUT_SECS: u64 = 30;

// =============================================================================
// Tool Commands
// =============================================================================

/// Claude Code CLI command name
pub const CLAUDE_COMMAND: &str = "claude";

/// Codex CLI command name
pub const CODEX_COMMAND: &str = "codex";

// =============================================================================
// Reserved Names
// =============================================================================

/// Profile names that cannot be used (reserved for system use)
pub const RESERVED_PROFILE_NAMES: &[&str] = &["default", "config", "cache", "profiles", "oauth"];

// =============================================================================
// Version
// =============================================================================

/// Current rafctl version from Cargo.toml
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_not_empty() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_reserved_names_not_empty() {
        assert!(!RESERVED_PROFILE_NAMES.is_empty());
    }
}
