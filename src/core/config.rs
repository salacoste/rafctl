use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::core::profile::{atomic_write, get_config_dir};
use crate::error::RafctlError;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlobalConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used_profile: Option<String>,
}

fn get_config_path() -> Result<PathBuf, RafctlError> {
    Ok(get_config_dir()?.join("config.yaml"))
}

pub fn load_global_config() -> Result<GlobalConfig, RafctlError> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        return Ok(GlobalConfig::default());
    }

    let content = fs::read_to_string(&config_path).map_err(|e| RafctlError::ConfigRead {
        path: config_path.clone(),
        source: e,
    })?;

    serde_yaml::from_str(&content).map_err(|e| RafctlError::ConfigRead {
        path: config_path,
        source: std::io::Error::new(std::io::ErrorKind::InvalidData, e),
    })
}

pub fn save_global_config(config: &GlobalConfig) -> Result<(), RafctlError> {
    let config_dir = get_config_dir()?;
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).map_err(|e| RafctlError::ConfigWrite {
            path: config_dir.clone(),
            source: e,
        })?;
    }

    let config_path = get_config_path()?;
    let yaml = serde_yaml::to_string(config).map_err(|e| RafctlError::ConfigWrite {
        path: config_path.clone(),
        source: std::io::Error::new(std::io::ErrorKind::InvalidData, e),
    })?;

    atomic_write(&config_path, &yaml)
}

pub fn set_last_used_profile(profile_name: &str) -> Result<(), RafctlError> {
    let mut config = load_global_config()?;
    config.last_used_profile = Some(profile_name.to_lowercase());
    save_global_config(&config)
}

pub fn get_default_profile() -> Result<Option<String>, RafctlError> {
    if let Ok(env_profile) = std::env::var("RAFCTL_DEFAULT_PROFILE") {
        if !env_profile.is_empty() {
            return Ok(Some(env_profile.to_lowercase()));
        }
    }

    let config = load_global_config()?;

    if let Some(default) = config.default_profile {
        return Ok(Some(default));
    }

    if let Some(last_used) = config.last_used_profile {
        return Ok(Some(last_used));
    }

    Ok(None)
}
