use colored::Colorize;

use crate::core::profile::{
    delete_profile, find_similar_profile, list_profiles, load_profile, profile_exists,
    save_profile, validate_profile_name, Profile, ToolType,
};
use crate::error::RafctlError;

pub fn handle_add(name: &str, tool: &str) -> Result<(), RafctlError> {
    validate_profile_name(name)?;

    let name_lower = name.to_lowercase();

    if profile_exists(&name_lower)? {
        return Err(RafctlError::ProfileAlreadyExists(name_lower));
    }

    let tool_type: ToolType = tool
        .parse()
        .map_err(|e: String| RafctlError::InvalidProfileName(e))?;

    let profile = Profile::new(name_lower.clone(), tool_type);
    save_profile(&profile)?;

    println!(
        "{} Profile '{}' created for {}",
        "✓".green(),
        name_lower,
        tool_type
    );

    Ok(())
}

pub fn handle_list() -> Result<(), RafctlError> {
    let profiles = list_profiles()?;

    if profiles.is_empty() {
        println!(
            "No profiles found. Create one with: rafctl profile add <name> --tool <claude|codex>"
        );
        return Ok(());
    }

    println!("{}", "Profiles:".bold());
    for name in profiles {
        match load_profile(&name) {
            Ok(profile) => {
                let last_used = profile
                    .last_used
                    .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|| "never".to_string());
                println!(
                    "  {} {} (last used: {})",
                    "•".cyan(),
                    format!("{} [{}]", profile.name, profile.tool).white(),
                    last_used.dimmed()
                );
            }
            Err(_) => {
                println!("  {} {} (corrupted)", "•".red(), name);
            }
        }
    }

    Ok(())
}

pub fn handle_show(name: &str) -> Result<(), RafctlError> {
    let name_lower = name.to_lowercase();
    let profile = load_profile(&name_lower).map_err(|e| {
        if let RafctlError::ProfileNotFound(_) = e {
            if let Ok(profiles) = list_profiles() {
                if let Some(suggestion) = find_similar_profile(name, &profiles) {
                    eprintln!(
                        "{} Profile '{}' not found. Did you mean '{}'?",
                        "✗".red(),
                        name,
                        suggestion.green()
                    );
                }
            }
        }
        e
    })?;

    println!("{}", format!("Profile: {}", profile.name).bold());
    println!("  Tool:       {}", profile.tool);
    println!(
        "  Created:    {}",
        profile.created_at.format("%Y-%m-%d %H:%M:%S")
    );
    println!(
        "  Last used:  {}",
        profile
            .last_used
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "never".to_string())
    );

    Ok(())
}

pub fn handle_remove(name: &str) -> Result<(), RafctlError> {
    let name_lower = name.to_lowercase();

    if !profile_exists(&name_lower)? {
        if let Ok(profiles) = list_profiles() {
            if let Some(suggestion) = find_similar_profile(name, &profiles) {
                eprintln!(
                    "{} Profile '{}' not found. Did you mean '{}'?",
                    "✗".red(),
                    name,
                    suggestion.green()
                );
            }
        }
        return Err(RafctlError::ProfileNotFound(name_lower));
    }

    delete_profile(&name_lower)?;

    println!("{} Profile '{}' removed", "✓".green(), name_lower);

    Ok(())
}
