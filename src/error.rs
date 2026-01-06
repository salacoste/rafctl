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

    #[error("Home directory not found")]
    NoHomeDir,

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

    #[error("Profile '{0}' is not authenticated")]
    NotAuthenticated(String),
}
