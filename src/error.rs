use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RafctlError {
    #[error("Profile '{0}' not found")]
    ProfileNotFound(String),

    #[error("Profile '{0}' already exists")]
    ProfileAlreadyExists(String),

    #[error("Invalid profile name '{0}': must match [a-zA-Z0-9_-]+")]
    InvalidProfileName(String),

    #[error("Reserved profile name '{0}': cannot use system names (default, config, cache)")]
    ReservedProfileName(String),

    #[error("Home directory not found")]
    NoHomeDir,

    #[error("No default profile configured. Set one with: rafctl config set-default <profile>")]
    NoDefaultProfile,

    #[error("Failed to read config '{path}'")]
    ConfigRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to write config '{path}'")]
    ConfigWrite {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Tool '{tool}' not found. Install: {install_url}")]
    ToolNotFound { tool: String, install_url: String },

    #[error("Failed to spawn '{tool}': {message}")]
    ProcessSpawn { tool: String, message: String },

    #[error("Profile '{0}' is not authenticated")]
    NotAuthenticated(String),

    #[error("Keychain error: {0}")]
    KeychainError(String),

    #[error("API key not configured for profile '{0}'")]
    NoApiKey(String),

    #[error("OAuth mode conflict: another OAuth instance is already running")]
    OAuthConflict,
}
