use std::io::{self, Write};
use std::process::{Command, Stdio};

use colored::Colorize;

use crate::core::profile::{
    list_profiles, load_profile, profile_exists, save_profile, AuthMode, ToolType,
};
use crate::error::RafctlError;
use crate::tools::{check_tool_available, is_authenticated};

pub fn handle_login(profile_name: &str) -> Result<(), RafctlError> {
    let name_lower = profile_name.to_lowercase();

    if !profile_exists(&name_lower)? {
        return Err(RafctlError::ProfileNotFound(name_lower));
    }

    let profile = load_profile(&name_lower)?;
    check_tool_available(profile.tool)?;

    let config_dir = profile.tool.config_dir_for_profile(&name_lower)?;

    let auth_args = profile.tool.auth_args();

    if auth_args.is_empty() {
        // Claude auto-authenticates on first run
        println!(
            "{} {} authenticates automatically on first run.",
            "ℹ".cyan(),
            profile.tool
        );
        println!(
            "{} Starting {}... Complete authentication in the browser.",
            "→".cyan(),
            profile.tool
        );
    } else {
        println!(
            "{} Opening browser for {} authorization...",
            "→".cyan(),
            profile.tool
        );
    }
    println!(
        "{} Waiting for authentication (Ctrl+C to cancel)...",
        "→".cyan()
    );

    let mut cmd = Command::new(profile.tool.command_name());
    for arg in auth_args {
        cmd.arg(arg);
    }
    let status = cmd
        .env(profile.tool.env_var_name(), &config_dir)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| RafctlError::ConfigWrite {
            path: config_dir.clone(),
            source: e,
        })?;

    if status.success() && is_authenticated(profile.tool, &name_lower)? {
        println!("{} Authenticated successfully!", "✓".green());
        Ok(())
    } else {
        println!("{} Authentication failed or was cancelled", "✗".red());
        Ok(())
    }
}

pub fn handle_status(profile_name: Option<&str>) -> Result<(), RafctlError> {
    match profile_name {
        Some(name) => show_single_status(name),
        None => show_all_status(),
    }
}

fn show_single_status(profile_name: &str) -> Result<(), RafctlError> {
    let name_lower = profile_name.to_lowercase();

    if !profile_exists(&name_lower)? {
        return Err(RafctlError::ProfileNotFound(name_lower));
    }

    let profile = load_profile(&name_lower)?;
    let authenticated = is_authenticated(profile.tool, &name_lower)?;

    println!("{}", format!("Profile: {}", profile.name).bold());
    println!("  Tool: {}", profile.tool);

    if authenticated {
        println!("  Auth: {} Authenticated", "✓".green());

        if let Some(last_used) = profile.last_used {
            let days_ago = (chrono::Utc::now() - last_used).num_days();
            if days_ago > 7 {
                println!(
                    "  {}",
                    format!("⚠ Last used {} days ago - auth may need refresh", days_ago).yellow()
                );
            }
        }
    } else {
        println!("  Auth: {} Not authenticated", "✗".red());
        println!(
            "  {}",
            format!("Run: rafctl auth login {}", name_lower).dimmed()
        );
    }

    Ok(())
}

fn show_all_status() -> Result<(), RafctlError> {
    let profiles = list_profiles()?;

    if profiles.is_empty() {
        println!(
            "No profiles found. Create one with: rafctl profile add <name> --tool <claude|codex>"
        );
        return Ok(());
    }

    println!("{}", "Auth Status:".bold());

    for name in profiles {
        match load_profile(&name) {
            Ok(profile) => {
                let authenticated = is_authenticated(profile.tool, &name).unwrap_or(false);
                let status_icon = if authenticated {
                    "✓".green()
                } else {
                    "✗".red()
                };
                let status_text = if authenticated {
                    "authenticated".to_string()
                } else {
                    "not authenticated".to_string()
                };
                println!(
                    "  {} {} [{}]: {}",
                    status_icon, profile.name, profile.tool, status_text
                );
            }
            Err(_) => {
                println!("  {} {} (corrupted)", "✗".red(), name);
            }
        }
    }

    Ok(())
}

pub fn handle_logout(profile_name: &str) -> Result<(), RafctlError> {
    let name_lower = profile_name.to_lowercase();

    if !profile_exists(&name_lower)? {
        return Err(RafctlError::ProfileNotFound(name_lower));
    }

    let profile = load_profile(&name_lower)?;
    let cred_path = profile.tool.credential_path(&name_lower)?;

    if !cred_path.exists() {
        println!(
            "{} Profile '{}' is not authenticated",
            "ℹ".cyan(),
            name_lower
        );
        return Ok(());
    }

    std::fs::remove_file(&cred_path).map_err(|e| RafctlError::ConfigWrite {
        path: cred_path,
        source: e,
    })?;

    println!("{} Logged out of '{}'", "✓".green(), name_lower);

    Ok(())
}

pub fn handle_set_key(profile_name: &str, api_key: Option<&str>) -> Result<(), RafctlError> {
    let name_lower = profile_name.to_lowercase();

    if !profile_exists(&name_lower)? {
        return Err(RafctlError::ProfileNotFound(name_lower));
    }

    let mut profile = load_profile(&name_lower)?;

    if profile.tool != ToolType::Claude {
        eprintln!(
            "{} API key mode only supported for Claude profiles",
            "✗".red()
        );
        return Ok(());
    }

    if profile.auth_mode != AuthMode::ApiKey {
        eprintln!(
            "{} Profile '{}' is in OAuth mode. Recreate with: rafctl profile add {} --tool claude --auth-mode api-key",
            "✗".red(),
            name_lower,
            name_lower
        );
        return Ok(());
    }

    let key = match api_key {
        Some(k) => k.to_string(),
        None => {
            print!("Enter API key: ");
            let _ = io::stdout().flush();
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .map_err(|e| RafctlError::ConfigRead {
                    path: std::path::PathBuf::from("stdin"),
                    source: e,
                })?;
            input.trim().to_string()
        }
    };

    if key.is_empty() {
        eprintln!("{} API key cannot be empty", "✗".red());
        return Ok(());
    }

    if !key.starts_with("sk-ant-api") {
        eprintln!(
            "{} Warning: API key doesn't look like an Anthropic key (should start with 'sk-ant-api')",
            "⚠".yellow()
        );
    }

    profile.api_key = Some(key);
    save_profile(&profile)?;

    println!("{} API key set for profile '{}'", "✓".green(), name_lower);

    Ok(())
}
