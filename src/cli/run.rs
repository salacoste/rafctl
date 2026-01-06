use std::process::{Command, Stdio};

use chrono::Utc;
use colored::Colorize;

use crate::core::config::{get_default_profile, set_last_used_profile};
use crate::core::profile::{list_profiles, load_profile, profile_exists, save_profile};
use crate::error::RafctlError;
use crate::tools::{check_tool_available, is_authenticated};

pub fn handle_run(profile_name: Option<&str>, args: &[String]) -> Result<i32, RafctlError> {
    let name = resolve_profile_name(profile_name)?;
    let name_lower = name.to_lowercase();

    if !profile_exists(&name_lower)? {
        return Err(RafctlError::ProfileNotFound(name_lower));
    }

    let mut profile = load_profile(&name_lower)?;
    check_tool_available(profile.tool)?;

    if !is_authenticated(profile.tool, &name_lower)? {
        eprintln!(
            "{} Profile '{}' is not authenticated",
            "✗".red(),
            name_lower
        );
        eprintln!(
            "{}",
            format!("Run: rafctl auth login {}", name_lower).dimmed()
        );
        return Err(RafctlError::NotAuthenticated(name_lower));
    }

    let config_dir = profile.tool.config_dir_for_profile(&name_lower)?;

    let mut cmd = Command::new(profile.tool.command_name());
    cmd.env(profile.tool.env_var_name(), &config_dir)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    for arg in args {
        cmd.arg(arg);
    }

    let status = cmd.status().map_err(|e| RafctlError::ConfigWrite {
        path: config_dir,
        source: e,
    })?;

    profile.last_used = Some(Utc::now());
    let _ = save_profile(&profile);
    let _ = set_last_used_profile(&name_lower);

    Ok(status.code().unwrap_or(1))
}

fn resolve_profile_name(profile_name: Option<&str>) -> Result<String, RafctlError> {
    if let Some(name) = profile_name {
        return Ok(name.to_string());
    }

    if let Some(default) = get_default_profile()? {
        return Ok(default);
    }

    let profiles = list_profiles()?;
    if profiles.is_empty() {
        eprintln!(
            "{} No profiles found. Create one with: rafctl profile add <name> --tool <claude|codex>",
            "✗".red()
        );
    } else {
        eprintln!(
            "{} No default profile. Specify a profile or run one first.",
            "✗".red()
        );
        eprintln!("{}", "Available profiles:".dimmed());
        for p in &profiles {
            eprintln!("  • {}", p);
        }
    }

    Err(RafctlError::ProfileNotFound("(no default)".to_string()))
}
