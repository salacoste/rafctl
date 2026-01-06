use colored::Colorize;
use serde::Serialize;

use super::output::print_json;
use super::OutputFormat;
use crate::core::config::{load_global_config, save_global_config};
use crate::core::profile::{get_config_dir, profile_exists};
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
