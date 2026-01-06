use colored::Colorize;
use comfy_table::{presets::UTF8_FULL, Cell, CellAlignment, Color, ContentArrangement, Table};

use crate::core::config::load_global_config;
use crate::core::profile::{list_profiles, load_profile, AuthMode, ToolType};
use crate::error::RafctlError;
use crate::tools::is_authenticated;

pub fn handle_status(profile_name: Option<&str>) -> Result<(), RafctlError> {
    match profile_name {
        Some(name) => show_single_status(name),
        None => show_all_status(),
    }
}

fn show_single_status(profile_name: &str) -> Result<(), RafctlError> {
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

    let config = load_global_config()?;

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

    for name in profiles {
        match load_profile(&name) {
            Ok(profile) => {
                let authenticated = is_authenticated(profile.tool, &name).unwrap_or(false);

                let is_default = config
                    .default_profile
                    .as_ref()
                    .map(|d| d == &name)
                    .unwrap_or(false);
                let is_last_used = config
                    .last_used_profile
                    .as_ref()
                    .map(|d| d == &name)
                    .unwrap_or(false);

                let name_display = if is_default {
                    format!("{} (default)", profile.name)
                } else if is_last_used {
                    format!("{} →", profile.name)
                } else {
                    profile.name.clone()
                };

                let tool_display = if profile.tool == ToolType::Claude {
                    format!("{} {}", profile.tool, profile.auth_mode)
                } else {
                    profile.tool.to_string()
                };

                let auth_cell = if authenticated {
                    Cell::new("✓").fg(Color::Green)
                } else {
                    Cell::new("✗").fg(Color::Red)
                };

                let last_used = profile
                    .last_used
                    .map(|dt| {
                        let days_ago = (chrono::Utc::now() - dt).num_days();
                        if days_ago == 0 {
                            "today".to_string()
                        } else if days_ago == 1 {
                            "yesterday".to_string()
                        } else if days_ago < 7 {
                            format!("{}d ago", days_ago)
                        } else {
                            dt.format("%Y-%m-%d").to_string()
                        }
                    })
                    .unwrap_or_else(|| "never".to_string());

                table.add_row(vec![
                    Cell::new(name_display),
                    Cell::new(tool_display),
                    auth_cell,
                    Cell::new(last_used),
                ]);
            }
            Err(_) => {
                table.add_row(vec![
                    Cell::new(&name),
                    Cell::new("?"),
                    Cell::new("✗").fg(Color::Red),
                    Cell::new("corrupted"),
                ]);
            }
        }
    }

    println!("{table}");

    let unauthenticated: Vec<_> = list_profiles()?
        .iter()
        .filter(|name| {
            load_profile(name)
                .map(|p| !is_authenticated(p.tool, name).unwrap_or(false))
                .unwrap_or(false)
        })
        .cloned()
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

    Ok(())
}
