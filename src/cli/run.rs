use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, ExitStatus, Stdio};

use chrono::Utc;
use colored::Colorize;

use crate::cli::debug;
use crate::core::config::{get_default_profile, set_last_used_profile};
use crate::core::constants::{
    ENV_ANTHROPIC_API_KEY, ENV_RAFCTL_PROFILE, ENV_RAFCTL_PROFILE_TOOL, ENV_RAFCTL_VERSION, VERSION,
};
use crate::core::credentials::{self, CredentialType};
use crate::core::profile::{
    get_config_dir, list_profiles, load_profile, profile_exists, resolve_profile_alias,
    save_profile, AuthMode, Profile, ToolType,
};
use crate::error::RafctlError;
use crate::tools::{check_tool_available, is_authenticated};

pub fn handle_run(profile_name: Option<&str>, args: &[String]) -> Result<i32, RafctlError> {
    let name = resolve_profile_name(profile_name)?;
    let name_lower = name.to_lowercase();

    debug::debug_labeled("profile", &name_lower);

    if !profile_exists(&name_lower)? {
        return Err(RafctlError::ProfileNotFound(name_lower));
    }

    let mut profile = load_profile(&name_lower)?;
    debug::debug_labeled("tool", &profile.tool.to_string());
    debug::debug_labeled("auth_mode", &profile.auth_mode.to_string());

    check_tool_available(profile.tool)?;

    set_terminal_title(&profile.name, profile.tool.command_name());

    let exit_code = match (&profile.tool, &profile.auth_mode) {
        (ToolType::Claude, AuthMode::ApiKey) => {
            debug::debug("launching with API key mode");
            launch_with_api_key(&profile, args)?
        }
        (ToolType::Claude, AuthMode::OAuth) => {
            debug::debug("launching with OAuth mode");
            launch_with_oauth(&profile, args)?
        }
        (ToolType::Codex, _) => {
            debug::debug("launching with default mode");
            launch_default(&profile, args)?
        }
    };

    update_profile_usage(&mut profile, &name_lower);

    Ok(exit_code)
}

fn update_profile_usage(profile: &mut Profile, name_lower: &str) {
    profile.last_used = Some(Utc::now());
    if let Err(e) = save_profile(profile) {
        eprintln!("{} Failed to update profile: {}", "⚠".yellow(), e);
    }
    if let Err(e) = set_last_used_profile(name_lower) {
        eprintln!("{} Failed to update last used: {}", "⚠".yellow(), e);
    }
}

fn build_rafctl_env(profile: &Profile) -> HashMap<String, String> {
    let mut env = HashMap::new();
    env.insert(ENV_RAFCTL_PROFILE.to_string(), profile.name.clone());
    env.insert(
        ENV_RAFCTL_PROFILE_TOOL.to_string(),
        profile.tool.to_string(),
    );
    env.insert(ENV_RAFCTL_VERSION.to_string(), VERSION.to_string());
    env
}

fn spawn_tool(
    profile: &Profile,
    args: &[String],
    extra_env: HashMap<String, String>,
) -> Result<i32, RafctlError> {
    let config_dir = profile.tool.config_dir_for_profile(&profile.name)?;

    debug::debug_path("config_dir", &config_dir);

    let mut cmd = Command::new(profile.tool.command_name());

    cmd.env(profile.tool.env_var_name(), &config_dir)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    for (key, value) in build_rafctl_env(profile) {
        debug::debug_env(&key, &value);
        cmd.env(key, value);
    }

    for (key, value) in extra_env {
        debug::debug_env(
            &key,
            if key == ENV_ANTHROPIC_API_KEY {
                "***"
            } else {
                &value
            },
        );
        cmd.env(key, value);
    }

    if !args.is_empty() {
        debug::debug_labeled("args", &args.join(" "));
    }

    for arg in args {
        cmd.arg(arg);
    }

    let status = execute_command(&mut cmd, profile.tool.command_name())?;
    Ok(status.code().unwrap_or(1))
}

fn execute_command(cmd: &mut Command, tool_name: &str) -> Result<ExitStatus, RafctlError> {
    cmd.status().map_err(|e| RafctlError::ProcessSpawn {
        tool: tool_name.to_string(),
        message: e.to_string(),
    })
}

fn launch_with_api_key(profile: &Profile, args: &[String]) -> Result<i32, RafctlError> {
    #[allow(deprecated)]
    let api_key = if let Some(ref key) = profile.api_key {
        key.clone()
    } else {
        credentials::get_credential(&profile.name, CredentialType::ApiKey)?
            .ok_or_else(|| RafctlError::NoApiKey(profile.name.clone()))?
    };

    let mut extra_env = HashMap::new();
    extra_env.insert(ENV_ANTHROPIC_API_KEY.to_string(), api_key);

    spawn_tool(profile, args, extra_env)
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

    launch_default(profile, args)
}

#[cfg(not(target_os = "macos"))]
fn launch_with_oauth(profile: &Profile, _args: &[String]) -> Result<i32, RafctlError> {
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

    spawn_tool(profile, args, HashMap::new())
}

fn set_terminal_title(profile_name: &str, tool_name: &str) {
    let _ = write!(
        std::io::stdout(),
        "\x1b]0;[rafctl:{}] {}\x07",
        profile_name,
        tool_name
    );
    let _ = std::io::stdout().flush();
}

fn resolve_profile_name(profile_name: Option<&str>) -> Result<String, RafctlError> {
    if let Some(name) = profile_name {
        return resolve_profile_alias(name);
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

    Err(RafctlError::NoDefaultProfile)
}
