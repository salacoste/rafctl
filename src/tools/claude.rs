//! Claude Code tool constants and configuration.

/// Environment variable for Claude config directory override.
pub const ENV_VAR_NAME: &str = "CLAUDE_CONFIG_DIR";

/// Command name to invoke Claude Code.
pub const COMMAND_NAME: &str = "claude";

/// Installation URL for Claude Code.
pub const INSTALL_URL: &str = "https://claude.ai/download";

/// Credential file name within the config directory.
/// Claude stores auth in the main config file.
pub const CREDENTIAL_FILE: &str = ".claude.json";

/// Auth command args for Claude (empty - just run claude for auto-auth).
pub const AUTH_ARGS: &[&str] = &[];
