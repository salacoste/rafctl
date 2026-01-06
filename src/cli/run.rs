use std::process::{Command, Stdio};

use chrono::Utc;
use colored::Colorize;

use crate::core::config::{get_default_profile, set_last_used_profile};
use crate::core::credentials::{self, CredentialType};
use crate::core::profile::{
    get_config_dir, list_profiles, load_profile, profile_exists, save_profile, AuthMode, Profile,
    ToolType,
};
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

    let exit_code = match (&profile.tool, &profile.auth_mode) {
        (ToolType::Claude, AuthMode::ApiKey) => launch_with_api_key(&profile, args)?,
        (ToolType::Claude, AuthMode::OAuth) => launch_with_oauth(&profile, args)?,
        (ToolType::Codex, _) => launch_default(&profile, args)?,
    };

    profile.last_used = Some(Utc::now());
    if let Err(e) = save_profile(&profile) {
        eprintln!("{} Failed to update profile: {}", "⚠".yellow(), e);
    }
    if let Err(e) = set_last_used_profile(&name_lower) {
        eprintln!("{} Failed to update last used: {}", "⚠".yellow(), e);
    }

    Ok(exit_code)
}

fn launch_with_api_key(profile: &Profile, args: &[String]) -> Result<i32, RafctlError> {
    #[allow(deprecated)]
    let api_key = if let Some(ref key) = profile.api_key {
        key.clone()
    } else {
        credentials::get_credential(&profile.name, CredentialType::ApiKey)?
            .ok_or_else(|| RafctlError::NoApiKey(profile.name.clone()))?
    };

    let config_dir = profile.tool.config_dir_for_profile(&profile.name)?;

    let mut cmd = Command::new(profile.tool.command_name());
    cmd.env("ANTHROPIC_API_KEY", api_key)
        .env(profile.tool.env_var_name(), &config_dir)
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

    Ok(status.code().unwrap_or(1))
}

#[cfg(target_os = "macos")]
fn launch_with_oauth(profile: &Profile, args: &[String]) -> Result<i32, RafctlError> {
    use fs2::FileExt;
    use std::fs::OpenOptions;

    let config_dir = get_config_dir()?;
    std::fs::create_dir_all(&config_dir).map_err(|e| RafctlError::ConfigWrite {
        path: config_dir.clone(),
        source: e,
    })?;
    let lock_path = config_dir.join("oauth.lock");

    let lock_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&lock_path)
        .map_err(|e| RafctlError::ConfigWrite {
            path: lock_path.clone(),
            source: e,
        })?;

    if lock_file.try_lock_exclusive().is_err() {
        return Err(RafctlError::OAuthConflict);
    }

    use std::io::Write;
    let mut lock_file = lock_file;
    let _ = writeln!(lock_file, "{}", profile.name);

    let token = credentials::get_credential(&profile.name, CredentialType::OAuthToken)?
        .ok_or_else(|| RafctlError::NotAuthenticated(profile.name.clone()))?;

    credentials::write_claude_system_token(&token)?;

    #[allow(clippy::let_and_return)]
    let result = launch_default(profile, args);
    result
}

#[cfg(not(target_os = "macos"))]
fn launch_with_oauth(profile: &Profile, args: &[String]) -> Result<i32, RafctlError> {
    eprintln!(
        "{} OAuth mode requires macOS for keychain support",
        "✗".red()
    );
    eprintln!(
        "{} Use API key mode instead: rafctl profile add {} --tool claude --auth-mode api-key",
        "ℹ".cyan(),
        profile.name
    );
    Err(RafctlError::KeychainError(
        "OAuth mode only available on macOS".to_string(),
    ))
}

fn launch_default(profile: &Profile, args: &[String]) -> Result<i32, RafctlError> {
    if !is_authenticated(profile.tool, &profile.name)? {
        eprintln!(
            "{} Profile '{}' is not authenticated",
            "✗".red(),
            profile.name
        );
        eprintln!(
            "{}",
            format!("Run: rafctl auth login {}", profile.name).dimmed()
        );
        return Err(RafctlError::NotAuthenticated(profile.name.clone()));
    }

    let config_dir = profile.tool.config_dir_for_profile(&profile.name)?;

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
