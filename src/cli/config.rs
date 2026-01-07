use std::collections::HashMap;
use std::path::PathBuf;

use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::output::print_json;
use super::OutputFormat;
use crate::core::config::{get_default_profile, load_global_config, save_global_config};
use crate::core::profile::{get_config_dir, load_profile, profile_exists, ToolType};
use crate::error::RafctlError;

#[derive(Serialize)]
struct ConfigOutput {
    default_profile: Option<String>,
    last_used_profile: Option<String>,
    config_directory: String,
}

pub fn handle_show(format: OutputFormat) -> Result<(), RafctlError> {
    let config = load_global_config()?;
    let config_dir = get_config_dir()?;

    let output = ConfigOutput {
        default_profile: config.default_profile.clone(),
        last_used_profile: config.last_used_profile.clone(),
        config_directory: config_dir.display().to_string(),
    };

    match format {
        OutputFormat::Json => {
            print_json(&output);
        }
        OutputFormat::Plain => {
            let default = config.default_profile.as_deref().unwrap_or("(not set)");
            let last_used = config.last_used_profile.as_deref().unwrap_or("(none)");
            println!("default_profile={}", default);
            println!("last_used_profile={}", last_used);
            println!("config_directory={}", config_dir.display());
        }
        OutputFormat::Human => {
            println!("{}", "Configuration:".bold());

            let default_profile = config.default_profile.as_deref().unwrap_or("(not set)");
            println!("  Default profile:   {}", default_profile);

            let last_used = config.last_used_profile.as_deref().unwrap_or("(none)");
            println!("  Last used profile: {}", last_used);

            println!("  Config directory:  {}", config_dir.display());
        }
    }

    Ok(())
}

pub fn handle_set_default(profile_name: &str) -> Result<(), RafctlError> {
    let name_lower = profile_name.to_lowercase();

    if !profile_exists(&name_lower)? {
        return Err(RafctlError::ProfileNotFound(name_lower));
    }

    let mut config = load_global_config()?;
    config.default_profile = Some(name_lower.clone());
    save_global_config(&config)?;

    println!("{} Default profile set to '{}'", "✓".green(), name_lower);

    Ok(())
}

pub fn handle_clear_default() -> Result<(), RafctlError> {
    let mut config = load_global_config()?;

    if config.default_profile.is_none() {
        println!("{} No default profile was set", "ℹ".cyan());
        return Ok(());
    }

    config.default_profile = None;
    save_global_config(&config)?;

    println!("{} Default profile cleared", "✓".green());

    Ok(())
}

pub fn handle_path() -> Result<(), RafctlError> {
    let config_dir = get_config_dir()?;
    println!("{}", config_dir.display());
    Ok(())
}

pub fn handle_hud(
    enable: bool,
    disable: bool,
    profile_name: Option<&str>,
) -> Result<(), RafctlError> {
    if !enable && !disable {
        println!("{} Usage: rafctl config hud --enable [profile]", "ℹ".cyan());
        println!("        rafctl config hud --disable [profile]");
        return Ok(());
    }

    if enable && disable {
        println!("{} Cannot use both --enable and --disable", "✗".red());
        return Ok(());
    }

    let name = resolve_profile_for_hud(profile_name)?;
    let profile = load_profile(&name)?;

    if profile.tool != ToolType::Claude {
        println!("{} HUD is only available for Claude profiles", "✗".red());
        return Ok(());
    }

    let settings_path = get_profile_settings_path(&name, profile.tool)?;

    if enable {
        enable_hud(&settings_path, &name)?;
    } else {
        disable_hud(&settings_path, &name)?;
    }

    Ok(())
}

fn resolve_profile_for_hud(profile_name: Option<&str>) -> Result<String, RafctlError> {
    if let Some(name) = profile_name {
        let name_lower = name.to_lowercase();
        if !profile_exists(&name_lower)? {
            return Err(RafctlError::ProfileNotFound(name_lower));
        }
        return Ok(name_lower);
    }

    if let Some(default) = get_default_profile()? {
        return Ok(default);
    }

    Err(RafctlError::ProfileNotFound(
        "(no profile specified and no default set)".to_string(),
    ))
}

fn get_profile_settings_path(profile_name: &str, tool: ToolType) -> Result<PathBuf, RafctlError> {
    let config_dir = tool.config_dir_for_profile(profile_name)?;
    Ok(config_dir.join("settings.json"))
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct ClaudeSettings {
    #[serde(default)]
    status_line: Option<StatusLineConfig>,
    #[serde(flatten)]
    other: HashMap<String, Value>,
}

#[derive(Debug, Deserialize, Serialize)]
struct StatusLineConfig {
    command: String,
}

fn enable_hud(settings_path: &PathBuf, profile_name: &str) -> Result<(), RafctlError> {
    let mut settings = load_settings(settings_path)?;

    if settings.status_line.is_some() {
        println!(
            "{} HUD already enabled for profile '{}'",
            "ℹ".cyan(),
            profile_name
        );
        return Ok(());
    }

    settings.status_line = Some(StatusLineConfig {
        command: "rafctl-hud".to_string(),
    });

    save_settings(settings_path, &settings)?;

    println!("{} HUD enabled for profile '{}'", "✓".green(), profile_name);
    println!(
        "{}",
        "Tip: Make sure rafctl-hud is in your PATH or run 'rafctl hud install'".dimmed()
    );

    Ok(())
}

fn disable_hud(settings_path: &PathBuf, profile_name: &str) -> Result<(), RafctlError> {
    let mut settings = load_settings(settings_path)?;

    if settings.status_line.is_none() {
        println!(
            "{} HUD not enabled for profile '{}'",
            "ℹ".cyan(),
            profile_name
        );
        return Ok(());
    }

    settings.status_line = None;

    save_settings(settings_path, &settings)?;

    println!(
        "{} HUD disabled for profile '{}'",
        "✓".green(),
        profile_name
    );

    Ok(())
}

fn load_settings(path: &PathBuf) -> Result<ClaudeSettings, RafctlError> {
    if !path.exists() {
        return Ok(ClaudeSettings::default());
    }

    let content = std::fs::read_to_string(path).map_err(|e| RafctlError::ConfigRead {
        path: path.clone(),
        source: e,
    })?;

    serde_json::from_str(&content).map_err(|e| RafctlError::ConfigRead {
        path: path.clone(),
        source: std::io::Error::other(e),
    })
}

fn save_settings(path: &PathBuf, settings: &ClaudeSettings) -> Result<(), RafctlError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| RafctlError::ConfigWrite {
            path: path.clone(),
            source: e,
        })?;
    }

    let content = serde_json::to_string_pretty(settings).map_err(|e| RafctlError::ConfigWrite {
        path: path.clone(),
        source: std::io::Error::other(e),
    })?;

    std::fs::write(path, content).map_err(|e| RafctlError::ConfigWrite {
        path: path.clone(),
        source: e,
    })
}
