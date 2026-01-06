use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::error::RafctlError;

const PROFILE_NAME_PATTERN: &str = r"^[a-zA-Z0-9_-]+$";
const MAX_PROFILE_NAME_LENGTH: usize = 64;
const RESERVED_NAMES: &[&str] = &["default", "config", "cache", "profiles", "oauth"];

/// Authentication mode for Claude Code profiles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AuthMode {
    /// OAuth-based authentication (subscription features, single instance)
    /// Tokens stored in macOS Keychain, swapped before each launch
    #[default]
    OAuth,
    /// API Key authentication (full isolation, parallel instances)
    /// Uses ANTHROPIC_API_KEY env var
    ApiKey,
}

impl std::fmt::Display for AuthMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthMode::OAuth => write!(f, "oauth"),
            AuthMode::ApiKey => write!(f, "api-key"),
        }
    }
}

impl std::str::FromStr for AuthMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "oauth" => Ok(AuthMode::OAuth),
            "api-key" | "apikey" | "api_key" => Ok(AuthMode::ApiKey),
            _ => Err(format!(
                "Invalid auth mode '{}'. Valid options: oauth, api-key",
                s
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolType {
    Claude,
    Codex,
}

impl std::fmt::Display for ToolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolType::Claude => write!(f, "claude"),
            ToolType::Codex => write!(f, "codex"),
        }
    }
}

impl std::str::FromStr for ToolType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "claude" => Ok(ToolType::Claude),
            "codex" => Ok(ToolType::Codex),
            _ => Err(format!(
                "Invalid tool type '{}'. Valid options: claude, codex",
                s
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub tool: ToolType,
    /// Authentication mode (OAuth or API Key)
    /// Only applicable for Claude - Codex always uses OAuth
    #[serde(default)]
    pub auth_mode: AuthMode,
    /// API key for API Key mode (stored encrypted in profile)
    /// Only used when auth_mode is ApiKey
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
}

impl Profile {
    pub fn new(name: String, tool: ToolType) -> Self {
        Self {
            name,
            tool,
            auth_mode: AuthMode::default(),
            api_key: None,
            created_at: Utc::now(),
            last_used: None,
        }
    }

    pub fn new_with_auth(name: String, tool: ToolType, auth_mode: AuthMode) -> Self {
        Self {
            name,
            tool,
            auth_mode,
            api_key: None,
            created_at: Utc::now(),
            last_used: None,
        }
    }

    /// Check if this profile supports parallel instances
    pub fn supports_parallel(&self) -> bool {
        matches!(
            (&self.tool, &self.auth_mode),
            (ToolType::Claude, AuthMode::ApiKey) | (ToolType::Codex, _)
        )
    }
}

pub fn validate_profile_name(name: &str) -> Result<(), RafctlError> {
    if name.is_empty() {
        return Err(RafctlError::InvalidProfileName(name.to_string()));
    }
    if name.len() > MAX_PROFILE_NAME_LENGTH {
        return Err(RafctlError::InvalidProfileName(name.to_string()));
    }
    let re = Regex::new(PROFILE_NAME_PATTERN).unwrap();
    if !re.is_match(name) {
        return Err(RafctlError::InvalidProfileName(name.to_string()));
    }
    // Check for reserved names
    let name_lower = name.to_lowercase();
    if RESERVED_NAMES.contains(&name_lower.as_str()) {
        return Err(RafctlError::ReservedProfileName(name.to_string()));
    }
    Ok(())
}

pub fn get_config_dir() -> Result<PathBuf, RafctlError> {
    let home = dirs::home_dir().ok_or(RafctlError::NoHomeDir)?;
    Ok(home.join(".rafctl"))
}

pub fn get_profiles_dir() -> Result<PathBuf, RafctlError> {
    Ok(get_config_dir()?.join("profiles"))
}

pub fn get_profile_dir(name: &str) -> Result<PathBuf, RafctlError> {
    Ok(get_profiles_dir()?.join(name.to_lowercase()))
}

pub fn get_profile_meta_path(name: &str) -> Result<PathBuf, RafctlError> {
    Ok(get_profile_dir(name)?.join("meta.yaml"))
}

fn ensure_dir_with_permissions(path: &Path) -> Result<(), RafctlError> {
    if !path.exists() {
        fs::create_dir_all(path).map_err(|e| RafctlError::ConfigWrite {
            path: path.to_path_buf(),
            source: e,
        })?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(|e| {
                RafctlError::ConfigWrite {
                    path: path.to_path_buf(),
                    source: e,
                }
            })?;
        }
    }
    Ok(())
}

fn atomic_write(path: &Path, content: &str) -> Result<(), RafctlError> {
    let tmp_path = path.with_extension("yaml.tmp");

    fs::write(&tmp_path, content).map_err(|e| RafctlError::ConfigWrite {
        path: tmp_path.clone(),
        source: e,
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&tmp_path, fs::Permissions::from_mode(0o600));
    }

    fs::rename(&tmp_path, path).map_err(|e| RafctlError::ConfigWrite {
        path: path.to_path_buf(),
        source: e,
    })?;

    Ok(())
}

pub fn save_profile(profile: &Profile) -> Result<(), RafctlError> {
    let profile_dir = get_profile_dir(&profile.name)?;
    ensure_dir_with_permissions(&profile_dir)?;

    let meta_path = get_profile_meta_path(&profile.name)?;
    let yaml = serde_yaml::to_string(profile).map_err(|e| RafctlError::ConfigWrite {
        path: meta_path.clone(),
        source: std::io::Error::new(std::io::ErrorKind::InvalidData, e),
    })?;

    atomic_write(&meta_path, &yaml)
}

pub fn load_profile(name: &str) -> Result<Profile, RafctlError> {
    let meta_path = get_profile_meta_path(name)?;

    if !meta_path.exists() {
        return Err(RafctlError::ProfileNotFound(name.to_string()));
    }

    let content = fs::read_to_string(&meta_path).map_err(|e| RafctlError::ConfigRead {
        path: meta_path.clone(),
        source: e,
    })?;

    serde_yaml::from_str(&content).map_err(|e| RafctlError::ConfigRead {
        path: meta_path,
        source: std::io::Error::new(std::io::ErrorKind::InvalidData, e),
    })
}

pub fn profile_exists(name: &str) -> Result<bool, RafctlError> {
    let meta_path = get_profile_meta_path(name)?;
    Ok(meta_path.exists())
}

pub fn list_profiles() -> Result<Vec<String>, RafctlError> {
    let profiles_dir = get_profiles_dir()?;

    if !profiles_dir.exists() {
        return Ok(Vec::new());
    }

    let mut profiles = Vec::new();
    let entries = fs::read_dir(&profiles_dir).map_err(|e| RafctlError::ConfigRead {
        path: profiles_dir.clone(),
        source: e,
    })?;

    for entry in entries.flatten() {
        if entry.path().is_dir() {
            let meta_path = entry.path().join("meta.yaml");
            if meta_path.exists() {
                if let Some(name) = entry.file_name().to_str() {
                    profiles.push(name.to_string());
                }
            }
        }
    }

    profiles.sort();
    Ok(profiles)
}

pub fn delete_profile(name: &str) -> Result<(), RafctlError> {
    let profile_dir = get_profile_dir(name)?;

    if !profile_dir.exists() {
        return Err(RafctlError::ProfileNotFound(name.to_string()));
    }

    fs::remove_dir_all(&profile_dir).map_err(|e| RafctlError::ConfigWrite {
        path: profile_dir,
        source: e,
    })
}

pub fn find_similar_profile(input: &str, profiles: &[String]) -> Option<String> {
    let input_lower = input.to_lowercase();
    profiles
        .iter()
        .find(|p| p.to_lowercase().starts_with(&input_lower))
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_mode_serialization() {
        let mode = AuthMode::OAuth;
        let yaml = serde_yaml::to_string(&mode).unwrap();
        assert_eq!(yaml.trim(), "oauth");

        let mode = AuthMode::ApiKey;
        let yaml = serde_yaml::to_string(&mode).unwrap();
        assert_eq!(yaml.trim(), "apikey");
    }

    #[test]
    fn test_auth_mode_from_str() {
        assert_eq!("oauth".parse::<AuthMode>().unwrap(), AuthMode::OAuth);
        assert_eq!("api-key".parse::<AuthMode>().unwrap(), AuthMode::ApiKey);
        assert_eq!("apikey".parse::<AuthMode>().unwrap(), AuthMode::ApiKey);
        assert_eq!("api_key".parse::<AuthMode>().unwrap(), AuthMode::ApiKey);
        assert!("invalid".parse::<AuthMode>().is_err());
    }

    #[test]
    fn test_tool_type_serialization() {
        let tool = ToolType::Claude;
        let yaml = serde_yaml::to_string(&tool).unwrap();
        assert_eq!(yaml.trim(), "claude");

        let tool = ToolType::Codex;
        let yaml = serde_yaml::to_string(&tool).unwrap();
        assert_eq!(yaml.trim(), "codex");
    }

    #[test]
    fn test_tool_type_deserialization() {
        let tool: ToolType = serde_yaml::from_str("claude").unwrap();
        assert_eq!(tool, ToolType::Claude);

        let tool: ToolType = serde_yaml::from_str("codex").unwrap();
        assert_eq!(tool, ToolType::Codex);
    }

    #[test]
    fn test_tool_type_from_str() {
        assert_eq!("claude".parse::<ToolType>().unwrap(), ToolType::Claude);
        assert_eq!("Claude".parse::<ToolType>().unwrap(), ToolType::Claude);
        assert_eq!("CODEX".parse::<ToolType>().unwrap(), ToolType::Codex);
        assert!("invalid".parse::<ToolType>().is_err());
    }

    #[test]
    fn test_profile_creation() {
        let profile = Profile::new("work".to_string(), ToolType::Claude);
        assert_eq!(profile.name, "work");
        assert_eq!(profile.tool, ToolType::Claude);
        assert_eq!(profile.auth_mode, AuthMode::OAuth);
        assert!(profile.api_key.is_none());
        assert!(profile.last_used.is_none());
    }

    #[test]
    fn test_profile_creation_with_auth() {
        let profile = Profile::new_with_auth(
            "api-profile".to_string(),
            ToolType::Claude,
            AuthMode::ApiKey,
        );
        assert_eq!(profile.auth_mode, AuthMode::ApiKey);
        assert!(profile.api_key.is_none());
    }

    #[test]
    fn test_profile_supports_parallel() {
        let oauth_claude = Profile::new("oauth".to_string(), ToolType::Claude);
        assert!(!oauth_claude.supports_parallel());

        let api_claude =
            Profile::new_with_auth("api".to_string(), ToolType::Claude, AuthMode::ApiKey);
        assert!(api_claude.supports_parallel());

        let codex = Profile::new("codex".to_string(), ToolType::Codex);
        assert!(codex.supports_parallel());
    }

    #[test]
    fn test_profile_serialization_roundtrip() {
        let profile = Profile::new("test-profile".to_string(), ToolType::Codex);
        let yaml = serde_yaml::to_string(&profile).unwrap();
        let restored: Profile = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(restored.name, profile.name);
        assert_eq!(restored.tool, profile.tool);
        assert_eq!(restored.auth_mode, profile.auth_mode);
    }

    #[test]
    fn test_profile_backwards_compatibility() {
        let old_yaml = r#"
name: old-profile
tool: claude
created_at: 2024-01-01T00:00:00Z
last_used: null
"#;
        let profile: Profile = serde_yaml::from_str(old_yaml).unwrap();
        assert_eq!(profile.auth_mode, AuthMode::OAuth);
        assert!(profile.api_key.is_none());
    }

    #[test]
    fn test_validate_profile_name_valid() {
        assert!(validate_profile_name("work").is_ok());
        assert!(validate_profile_name("my-profile").is_ok());
        assert!(validate_profile_name("profile_123").is_ok());
        assert!(validate_profile_name("Test-Profile_01").is_ok());
    }

    #[test]
    fn test_validate_profile_name_invalid() {
        assert!(validate_profile_name("").is_err());
        assert!(validate_profile_name("work@home").is_err());
        assert!(validate_profile_name("my profile").is_err());
        assert!(validate_profile_name("profile/test").is_err());

        let long_name = "a".repeat(65);
        assert!(validate_profile_name(&long_name).is_err());
    }

    #[test]
    fn test_validate_profile_name_reserved() {
        assert!(validate_profile_name("default").is_err());
        assert!(validate_profile_name("Default").is_err());
        assert!(validate_profile_name("CONFIG").is_err());
        assert!(validate_profile_name("cache").is_err());
        assert!(validate_profile_name("profiles").is_err());
        assert!(validate_profile_name("oauth").is_err());
    }

    #[test]
    fn test_find_similar_profile() {
        let profiles = vec![
            "work".to_string(),
            "personal".to_string(),
            "client-a".to_string(),
        ];

        assert_eq!(
            find_similar_profile("wor", &profiles),
            Some("work".to_string())
        );
        assert_eq!(
            find_similar_profile("per", &profiles),
            Some("personal".to_string())
        );
        assert_eq!(find_similar_profile("xyz", &profiles), None);
    }
}
