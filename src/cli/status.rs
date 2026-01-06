use colored::Colorize;
use comfy_table::{presets::UTF8_FULL, Cell, CellAlignment, Color, ContentArrangement, Table};
use serde::Serialize;

use crate::cli::output::print_json;
use crate::cli::OutputFormat;
use crate::core::config::load_global_config;
use crate::core::profile::{list_profiles, load_profile, AuthMode, ToolType};
use crate::error::RafctlError;
use crate::tools::is_authenticated;

#[derive(Serialize)]
struct ProfileStatus {
    name: String,
    tool: String,
    auth_mode: Option<String>,
    authenticated: bool,
    is_default: bool,
    is_last_used: bool,
    created_at: String,
    last_used: Option<String>,
}

#[derive(Serialize)]
struct StatusOutput {
    profiles: Vec<ProfileStatus>,
}

pub fn handle_status(profile_name: Option<&str>, format: OutputFormat) -> Result<(), RafctlError> {
    match profile_name {
        Some(name) => show_single_status(name, format),
        None => show_all_status(format),
    }
}

fn show_single_status(profile_name: &str, format: OutputFormat) -> Result<(), RafctlError> {
    let name_lower = profile_name.to_lowercase();
    let profile = load_profile(&name_lower)?;
    let config = load_global_config()?;
    let authenticated = is_authenticated(profile.tool, &name_lower)?;

    let is_default = config
        .default_profile
        .as_ref()
        .map(|d| d == &name_lower)
        .unwrap_or(false);
    let is_last_used = config
        .last_used_profile
        .as_ref()
        .map(|d| d == &name_lower)
        .unwrap_or(false);

    let status = ProfileStatus {
        name: profile.name.clone(),
        tool: profile.tool.to_string(),
        auth_mode: if profile.tool == ToolType::Claude {
            Some(profile.auth_mode.to_string())
        } else {
            None
        },
        authenticated,
        is_default,
        is_last_used,
        created_at: profile.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        last_used: profile
            .last_used
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
    };

    match format {
        OutputFormat::Json => {
            print_json(&status);
        }
        OutputFormat::Plain => {
            println!("Profile: {}", profile.name);
            if is_default {
                println!("  Status: default profile");
            } else if is_last_used {
                println!("  Status: last used");
            }
            println!("  Tool: {}", profile.tool);
            if profile.tool == ToolType::Claude {
                println!("  Auth mode: {}", profile.auth_mode);
                if profile.auth_mode == AuthMode::ApiKey {
                    let has_key = profile.api_key.is_some();
                    println!(
                        "  API key: {}",
                        if has_key { "configured" } else { "not set" }
                    );
                }
            }
            println!("  Auth: {}", if authenticated { "yes" } else { "no" });
            println!(
                "  Created: {}",
                profile.created_at.format("%Y-%m-%d %H:%M:%S")
            );
            let last_used_str = profile
                .last_used
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "never".to_string());
            println!("  Last used: {}", last_used_str);
        }
        OutputFormat::Human => {
            println!("{}", format!("Profile: {}", profile.name).bold());

            if is_default {
                println!("  Status:     {} default profile", "★".yellow());
            } else if is_last_used {
                println!("  Status:     {} last used", "→".cyan());
            }

            println!("  Tool:       {}", profile.tool);

            if profile.tool == ToolType::Claude {
                println!("  Auth mode:  {}", profile.auth_mode);
                if profile.auth_mode == AuthMode::ApiKey {
                    let has_key = profile.api_key.is_some();
                    let key_status = if has_key {
                        "configured".green()
                    } else {
                        "not set".red()
                    };
                    println!("  API key:    {}", key_status);
                }
            }

            let auth_status = if authenticated {
                format!("{} Authenticated", "✓".green())
            } else {
                format!("{} Not authenticated", "✗".red())
            };
            println!("  Auth:       {}", auth_status);

            println!(
                "  Created:    {}",
                profile.created_at.format("%Y-%m-%d %H:%M:%S")
            );

            let last_used_str = profile
                .last_used
                .map(|dt| {
                    let days_ago = (chrono::Utc::now() - dt).num_days();
                    if days_ago == 0 {
                        "today".to_string()
                    } else if days_ago == 1 {
                        "yesterday".to_string()
                    } else {
                        format!("{} days ago", days_ago)
                    }
                })
                .unwrap_or_else(|| "never".to_string());
            println!("  Last used:  {}", last_used_str);

            if !authenticated {
                println!();
                println!(
                    "{}",
                    format!("  Run: rafctl auth login {}", name_lower).dimmed()
                );
            }
        }
    }

    Ok(())
}

fn show_all_status(format: OutputFormat) -> Result<(), RafctlError> {
    let profiles = list_profiles()?;

    if profiles.is_empty() {
        match format {
            OutputFormat::Json => print_json(&StatusOutput { profiles: vec![] }),
            OutputFormat::Plain => {
                println!("No profiles found.");
            }
            OutputFormat::Human => {
                println!(
                    "No profiles found. Create one with: rafctl profile add <name> --tool <claude|codex>"
                );
            }
        }
        return Ok(());
    }

    let config = load_global_config()?;

    let mut status_list: Vec<ProfileStatus> = Vec::new();

    for name in &profiles {
        if let Ok(profile) = load_profile(name) {
            let authenticated = is_authenticated(profile.tool, name).unwrap_or(false);
            let is_default = config
                .default_profile
                .as_ref()
                .map(|d| d == name)
                .unwrap_or(false);
            let is_last_used = config
                .last_used_profile
                .as_ref()
                .map(|d| d == name)
                .unwrap_or(false);

            status_list.push(ProfileStatus {
                name: profile.name.clone(),
                tool: profile.tool.to_string(),
                auth_mode: if profile.tool == ToolType::Claude {
                    Some(profile.auth_mode.to_string())
                } else {
                    None
                },
                authenticated,
                is_default,
                is_last_used,
                created_at: profile.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                last_used: profile
                    .last_used
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            });
        }
    }

    match format {
        OutputFormat::Json => {
            print_json(&StatusOutput {
                profiles: status_list,
            });
        }
        OutputFormat::Plain => {
            println!("NAME\tTOOL\tAUTH\tLAST_USED");
            for s in &status_list {
                let auth = if s.authenticated { "yes" } else { "no" };
                let last_used = s.last_used.as_deref().unwrap_or("never");
                println!("{}\t{}\t{}\t{}", s.name, s.tool, auth, last_used);
            }
        }
        OutputFormat::Human => {
            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .set_content_arrangement(ContentArrangement::Dynamic)
                .set_header(vec![
                    Cell::new("Name").set_alignment(CellAlignment::Left),
                    Cell::new("Tool").set_alignment(CellAlignment::Center),
                    Cell::new("Auth").set_alignment(CellAlignment::Center),
                    Cell::new("Last Used").set_alignment(CellAlignment::Right),
                ]);

            for s in &status_list {
                let name_display = if s.is_default {
                    format!("{} (default)", s.name)
                } else if s.is_last_used {
                    format!("{} →", s.name)
                } else {
                    s.name.clone()
                };

                let tool_display = if let Some(ref auth_mode) = s.auth_mode {
                    format!("{} {}", s.tool, auth_mode)
                } else {
                    s.tool.clone()
                };

                let auth_cell = if s.authenticated {
                    Cell::new("✓").fg(Color::Green)
                } else {
                    Cell::new("✗").fg(Color::Red)
                };

                let last_used = s.last_used.as_deref().unwrap_or("never");

                table.add_row(vec![
                    Cell::new(name_display),
                    Cell::new(tool_display),
                    auth_cell,
                    Cell::new(last_used),
                ]);
            }

            println!("{table}");

            let unauthenticated: Vec<_> = status_list
                .iter()
                .filter(|s| !s.authenticated)
                .map(|s| s.name.clone())
                .collect();

            if !unauthenticated.is_empty() {
                println!();
                println!(
                    "{}",
                    format!(
                        "Unauthenticated: {}. Run 'rafctl auth login <profile>' to authenticate.",
                        unauthenticated.join(", ")
                    )
                    .dimmed()
                );
            }
        }
    }

    Ok(())
}
