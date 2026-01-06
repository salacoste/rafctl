use colored::Colorize;
use serde::Serialize;

use super::output::print_json;
use super::OutputFormat;
use crate::core::credentials;
use crate::core::profile::{
    delete_profile, find_similar_profile, list_profiles, load_profile, profile_exists,
    save_profile, validate_profile_name, AuthMode, Profile, ToolType,
};
use crate::error::RafctlError;

#[derive(Serialize)]
struct ProfileInfo {
    name: String,
    tool: String,
    auth_mode: Option<String>,
    api_key_configured: Option<bool>,
    created_at: String,
    last_used: Option<String>,
}

#[derive(Serialize)]
struct ProfileListOutput {
    profiles: Vec<ProfileInfo>,
}

pub fn handle_add(name: &str, tool: &str, auth_mode: Option<&str>) -> Result<(), RafctlError> {
    validate_profile_name(name)?;

    let name_lower = name.to_lowercase();

    if profile_exists(&name_lower)? {
        return Err(RafctlError::ProfileAlreadyExists(name_lower));
    }

    let tool_type: ToolType = tool
        .parse()
        .map_err(|e: String| RafctlError::InvalidProfileName(e))?;

    let auth = match auth_mode {
        Some(mode) => mode
            .parse::<AuthMode>()
            .map_err(RafctlError::InvalidProfileName)?,
        None => AuthMode::default(),
    };

    if tool_type == ToolType::Codex && auth == AuthMode::ApiKey {
        eprintln!("{} Codex only supports OAuth authentication", "⚠".yellow());
    }

    let profile = Profile::new_with_auth(name_lower.clone(), tool_type, auth);
    save_profile(&profile)?;

    let mode_info = if tool_type == ToolType::Claude {
        format!(" ({})", auth)
    } else {
        String::new()
    };

    println!(
        "{} Profile '{}' created for {}{}",
        "✓".green(),
        name_lower,
        tool_type,
        mode_info
    );

    if auth == AuthMode::ApiKey {
        println!(
            "{} Set API key with: rafctl auth set-key {}",
            "ℹ".cyan(),
            name_lower
        );
    }

    Ok(())
}

pub fn handle_list(format: OutputFormat) -> Result<(), RafctlError> {
    let profiles = list_profiles()?;

    if profiles.is_empty() {
        match format {
            OutputFormat::Json => print_json(&ProfileListOutput { profiles: vec![] }),
            OutputFormat::Plain => println!("No profiles found."),
            OutputFormat::Human => {
                println!(
                    "No profiles found. Create one with: rafctl profile add <name> --tool <claude|codex>"
                );
            }
        }
        return Ok(());
    }

    let mut profile_list: Vec<ProfileInfo> = Vec::new();

    for name in &profiles {
        if let Ok(profile) = load_profile(name) {
            profile_list.push(ProfileInfo {
                name: profile.name.clone(),
                tool: profile.tool.to_string(),
                auth_mode: if profile.tool == ToolType::Claude {
                    Some(profile.auth_mode.to_string())
                } else {
                    None
                },
                api_key_configured: if profile.tool == ToolType::Claude
                    && profile.auth_mode == AuthMode::ApiKey
                {
                    #[allow(deprecated)]
                    Some(credentials::has_api_key_configured(name, &profile.api_key))
                } else {
                    None
                },
                created_at: profile.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                last_used: profile
                    .last_used
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            });
        }
    }

    match format {
        OutputFormat::Json => {
            print_json(&ProfileListOutput {
                profiles: profile_list,
            });
        }
        OutputFormat::Plain => {
            println!("NAME\tTOOL\tAUTH_MODE\tLAST_USED");
            for p in &profile_list {
                let auth_mode = p.auth_mode.as_deref().unwrap_or("-");
                let last_used = p.last_used.as_deref().unwrap_or("never");
                println!("{}\t{}\t{}\t{}", p.name, p.tool, auth_mode, last_used);
            }
        }
        OutputFormat::Human => {
            println!("{}", "Profiles:".bold());
            for name in profiles {
                match load_profile(&name) {
                    Ok(profile) => {
                        let last_used = profile
                            .last_used
                            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                            .unwrap_or_else(|| "never".to_string());
                        let auth_info = if profile.tool == ToolType::Claude {
                            format!(" {}", profile.auth_mode)
                        } else {
                            String::new()
                        };
                        println!(
                            "  {} {} (last used: {})",
                            "•".cyan(),
                            format!("{} [{}{}]", profile.name, profile.tool, auth_info).white(),
                            last_used.dimmed()
                        );
                    }
                    Err(_) => {
                        println!("  {} {} (corrupted)", "•".red(), name);
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn handle_show(name: &str, format: OutputFormat) -> Result<(), RafctlError> {
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

    let info = ProfileInfo {
        name: profile.name.clone(),
        tool: profile.tool.to_string(),
        auth_mode: if profile.tool == ToolType::Claude {
            Some(profile.auth_mode.to_string())
        } else {
            None
        },
        api_key_configured: if profile.tool == ToolType::Claude
            && profile.auth_mode == AuthMode::ApiKey
        {
            #[allow(deprecated)]
            Some(credentials::has_api_key_configured(
                &name_lower,
                &profile.api_key,
            ))
        } else {
            None
        },
        created_at: profile.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        last_used: profile
            .last_used
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
    };

    match format {
        OutputFormat::Json => {
            print_json(&info);
        }
        OutputFormat::Plain => {
            println!("Profile: {}", profile.name);
            println!("Tool: {}", profile.tool);
            if profile.tool == ToolType::Claude {
                println!("Auth mode: {}", profile.auth_mode);
                if profile.auth_mode == AuthMode::ApiKey {
                    #[allow(deprecated)]
                    let has_key =
                        credentials::has_api_key_configured(&name_lower, &profile.api_key);
                    println!(
                        "API key: {}",
                        if has_key { "configured" } else { "not set" }
                    );
                }
            }
            println!(
                "Created: {}",
                profile.created_at.format("%Y-%m-%d %H:%M:%S")
            );
            println!(
                "Last used: {}",
                profile
                    .last_used
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "never".to_string())
            );
        }
        OutputFormat::Human => {
            println!("{}", format!("Profile: {}", profile.name).bold());
            println!("  Tool:       {}", profile.tool);
            if profile.tool == ToolType::Claude {
                println!("  Auth mode:  {}", profile.auth_mode);
                if profile.auth_mode == AuthMode::ApiKey {
                    #[allow(deprecated)]
                    let has_key =
                        credentials::has_api_key_configured(&name_lower, &profile.api_key);
                    let key_status = if has_key {
                        "configured".green()
                    } else {
                        "not set".red()
                    };
                    println!("  API key:    {}", key_status);
                }
            }
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
        }
    }

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
