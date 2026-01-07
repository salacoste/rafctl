//! HUD installation and management commands.

use std::fs;
use std::path::PathBuf;

use colored::Colorize;
use serde_json::{json, Value};

use crate::error::RafctlError;

pub fn handle_hud_install(profile: Option<&str>) -> Result<(), RafctlError> {
    let bin_path = get_hud_binary_path()?;
    let settings_path = get_settings_path(profile)?;

    if !bin_path.exists() {
        return Err(RafctlError::ProfileNotFound(format!(
            "rafctl-hud binary not found at {}. Build with 'cargo build --release' first.",
            bin_path.display()
        )));
    }

    let mut settings = read_settings(&settings_path)?;

    let status_line_config = json!({
        "command": bin_path.to_string_lossy()
    });

    settings["statusLine"] = status_line_config;

    write_settings(&settings_path, &settings)?;

    println!(
        "{} HUD installed successfully for {}",
        "✓".green(),
        profile.unwrap_or("global Claude Code")
    );
    println!("  {} {}", "Binary:".dimmed(), bin_path.display());
    println!("  {} {}", "Config:".dimmed(), settings_path.display());
    println!();
    println!("{}", "Restart Claude Code to see the HUD.".cyan());

    Ok(())
}

pub fn handle_hud_uninstall(profile: Option<&str>) -> Result<(), RafctlError> {
    let settings_path = get_settings_path(profile)?;

    let mut settings = read_settings(&settings_path)?;

    if settings.get("statusLine").is_some() {
        settings.as_object_mut().map(|obj| obj.remove("statusLine"));

        write_settings(&settings_path, &settings)?;

        println!(
            "{} HUD uninstalled for {}",
            "✓".green(),
            profile.unwrap_or("global Claude Code")
        );
    } else {
        println!(
            "{} HUD was not installed for {}",
            "ℹ".cyan(),
            profile.unwrap_or("global Claude Code")
        );
    }

    Ok(())
}

pub fn handle_hud_status(profile: Option<&str>) -> Result<(), RafctlError> {
    let settings_path = get_settings_path(profile)?;
    let bin_path = get_hud_binary_path()?;

    println!("\n{} HUD Status\n", "📊".cyan());

    let binary_exists = bin_path.exists();
    let binary_status = if binary_exists {
        "✓ Installed".green().to_string()
    } else {
        "✗ Not found".red().to_string()
    };

    println!("Binary:   {} ({})", binary_status, bin_path.display());

    if settings_path.exists() {
        let settings = read_settings(&settings_path)?;
        if let Some(status_line) = settings.get("statusLine") {
            let command = status_line
                .get("command")
                .and_then(|c| c.as_str())
                .unwrap_or("unknown");

            println!(
                "Config:   {} ({})",
                "✓ Enabled".green(),
                settings_path.display()
            );
            println!("Command:  {}", command.cyan());
        } else {
            println!(
                "Config:   {} ({})",
                "○ Not configured".yellow(),
                settings_path.display()
            );
        }
    } else {
        println!(
            "Config:   {} ({})",
            "○ File not found".yellow(),
            settings_path.display()
        );
    }

    println!();

    Ok(())
}

fn get_hud_binary_path() -> Result<PathBuf, RafctlError> {
    let current_exe = std::env::current_exe().map_err(|e| RafctlError::ConfigRead {
        path: PathBuf::from("current_exe"),
        source: e,
    })?;

    let bin_dir = current_exe
        .parent()
        .ok_or_else(|| RafctlError::ProfileNotFound("Cannot determine binary directory".into()))?;

    Ok(bin_dir.join("rafctl-hud"))
}

fn get_settings_path(profile: Option<&str>) -> Result<PathBuf, RafctlError> {
    let home = dirs::home_dir()
        .ok_or_else(|| RafctlError::ProfileNotFound("Home directory not found".into()))?;

    let path = match profile {
        Some(name) => home
            .join(".rafctl")
            .join("profiles")
            .join(name)
            .join("claude")
            .join("settings.json"),
        None => home.join(".claude").join("settings.json"),
    };

    Ok(path)
}

fn read_settings(path: &PathBuf) -> Result<Value, RafctlError> {
    if path.exists() {
        let content = fs::read_to_string(path).map_err(|e| RafctlError::ConfigRead {
            path: path.clone(),
            source: e,
        })?;

        serde_json::from_str(&content).map_err(|_| RafctlError::ConfigRead {
            path: path.clone(),
            source: std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid JSON"),
        })
    } else {
        Ok(json!({}))
    }
}

fn write_settings(path: &PathBuf, settings: &Value) -> Result<(), RafctlError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| RafctlError::ConfigWrite {
            path: parent.to_path_buf(),
            source: e,
        })?;
    }

    let content = serde_json::to_string_pretty(settings).map_err(|_| RafctlError::ConfigWrite {
        path: path.clone(),
        source: std::io::Error::new(std::io::ErrorKind::InvalidData, "JSON serialization failed"),
    })?;

    fs::write(path, content).map_err(|e| RafctlError::ConfigWrite {
        path: path.clone(),
        source: e,
    })?;

    Ok(())
}
