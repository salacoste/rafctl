pub mod claude;
pub mod codex;

use std::path::PathBuf;
use std::process::Command;

use crate::core::profile::{get_profile_dir, ToolType};
use crate::error::RafctlError;

impl ToolType {
    pub fn env_var_name(&self) -> &'static str {
        match self {
            ToolType::Claude => claude::ENV_VAR_NAME,
            ToolType::Codex => codex::ENV_VAR_NAME,
        }
    }

    pub fn command_name(&self) -> &'static str {
        match self {
            ToolType::Claude => claude::COMMAND_NAME,
            ToolType::Codex => codex::COMMAND_NAME,
        }
    }

    pub fn install_url(&self) -> &'static str {
        match self {
            ToolType::Claude => claude::INSTALL_URL,
            ToolType::Codex => codex::INSTALL_URL,
        }
    }

    pub fn credential_file(&self) -> &'static str {
        match self {
            ToolType::Claude => claude::CREDENTIAL_FILE,
            ToolType::Codex => codex::CREDENTIAL_FILE,
        }
    }

    pub fn credential_path(&self, profile_name: &str) -> Result<PathBuf, RafctlError> {
        let profile_dir = get_profile_dir(profile_name)?;
        Ok(profile_dir.join(self.credential_file()))
    }

    pub fn config_dir_for_profile(&self, profile_name: &str) -> Result<PathBuf, RafctlError> {
        get_profile_dir(profile_name)
    }

    pub fn auth_args(&self) -> &'static [&'static str] {
        match self {
            ToolType::Claude => claude::AUTH_ARGS,
            ToolType::Codex => codex::AUTH_ARGS,
        }
    }
}

pub fn check_tool_available(tool: ToolType) -> Result<(), RafctlError> {
    let cmd_name = tool.command_name();

    match Command::new(cmd_name).arg("--version").output() {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(RafctlError::ToolNotFound {
            tool: cmd_name.to_string(),
            install_url: tool.install_url().to_string(),
        }),
        Err(_) => Ok(()),
    }
}

pub fn is_authenticated(tool: ToolType, profile_name: &str) -> Result<bool, RafctlError> {
    let cred_path = tool.credential_path(profile_name)?;
    Ok(cred_path.exists())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_env_var() {
        assert_eq!(ToolType::Claude.env_var_name(), "CLAUDE_CONFIG_DIR");
    }

    #[test]
    fn test_codex_env_var() {
        assert_eq!(ToolType::Codex.env_var_name(), "CODEX_HOME");
    }

    #[test]
    fn test_command_names() {
        assert_eq!(ToolType::Claude.command_name(), "claude");
        assert_eq!(ToolType::Codex.command_name(), "codex");
    }

    #[test]
    fn test_install_urls() {
        assert!(ToolType::Claude.install_url().contains("claude"));
        assert!(ToolType::Codex.install_url().contains("codex"));
    }

    #[test]
    fn test_credential_files() {
        assert_eq!(ToolType::Claude.credential_file(), ".claude.json");
        assert_eq!(ToolType::Codex.credential_file(), "auth.json");
    }

    #[test]
    fn test_auth_args() {
        // Claude auto-authenticates, no explicit auth command
        assert!(ToolType::Claude.auth_args().is_empty());
        // Codex uses "codex login"
        assert_eq!(ToolType::Codex.auth_args(), &["login"]);
    }
}
