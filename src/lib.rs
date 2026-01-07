//! rafctl library - core functionality for AI coding agent profile management.

pub mod cli;
pub mod core;
pub mod error;
pub mod hud;
pub mod tools;

use anyhow::Result;
use clap::Parser;

use crate::cli::analytics::handle_analytics;
use crate::cli::auth::{
    handle_login, handle_logout, handle_set_key, handle_status as handle_auth_status,
};
use crate::cli::config::{
    handle_clear_default, handle_hud as handle_config_hud, handle_path as handle_config_path,
    handle_set_default, handle_show as handle_config_show,
};
use crate::cli::dashboard::{run_dashboard, DashboardAction};
use crate::cli::hud::{handle_hud_install, handle_hud_status, handle_hud_uninstall};
use crate::cli::profile::{handle_add, handle_list, handle_remove, handle_show};
use crate::cli::quota::handle_quota;
use crate::cli::run::handle_run;
use crate::cli::sessions::handle_sessions;
use crate::cli::status::handle_status;
use crate::cli::watch::handle_watch;
use crate::cli::{AuthAction, Cli, Commands, ConfigAction, HudAction, ProfileAction};

/// Main entry point for the CLI application.
pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let format = cli.output_format();

    match cli.command {
        Commands::Profile { action } => match action {
            ProfileAction::Add {
                name,
                tool,
                auth_mode,
            } => {
                handle_add(&name, &tool, auth_mode.as_deref())?;
            }
            ProfileAction::List => {
                handle_list(format)?;
            }
            ProfileAction::Remove { name, yes } => {
                handle_remove(&name, yes)?;
            }
            ProfileAction::Show { name } => {
                handle_show(&name, format)?;
            }
        },
        Commands::Auth { action } => match action {
            AuthAction::Login { profile } => {
                handle_login(&profile)?;
            }
            AuthAction::Logout { profile } => {
                handle_logout(&profile)?;
            }
            AuthAction::Status { profile } => {
                handle_auth_status(profile.as_deref())?;
            }
            AuthAction::SetKey { profile, key } => {
                handle_set_key(&profile, key.as_deref())?;
            }
        },
        Commands::Run { profile, args } => {
            let exit_code = handle_run(profile.as_deref(), &args)?;
            if exit_code != 0 {
                std::process::exit(exit_code);
            }
        }
        Commands::Status { profile } => {
            handle_status(profile.as_deref(), format)?;
        }
        Commands::Quota { profile } => {
            handle_quota(profile.as_deref(), format)?;
        }
        Commands::Config { action } => match action {
            ConfigAction::Show => {
                handle_config_show(format)?;
            }
            ConfigAction::SetDefault { profile } => {
                handle_set_default(&profile)?;
            }
            ConfigAction::ClearDefault => {
                handle_clear_default()?;
            }
            ConfigAction::Path => {
                handle_config_path()?;
            }
            ConfigAction::Hud {
                enable,
                disable,
                profile,
            } => {
                handle_config_hud(enable, disable, profile.as_deref())?;
            }
        },
        Commands::Completion { shell } => {
            cli::generate_completions(shell);
        }
        Commands::Dashboard => {
            let action = run_dashboard()?;
            match action {
                DashboardAction::None => {}
                DashboardAction::Run(profile) => {
                    let exit_code = handle_run(Some(&profile), &[])?;
                    if exit_code != 0 {
                        std::process::exit(exit_code);
                    }
                }
                DashboardAction::Login(profile) => {
                    handle_login(&profile)?;
                }
            }
        }
        Commands::Switch { profile } => {
            handle_set_default(&profile)?;
            handle_status(Some(&profile), format)?;
        }
        Commands::Analytics {
            profile,
            days,
            all,
            cost,
        } => {
            handle_analytics(profile.as_deref(), days, all, cost, format)?;
        }
        Commands::Sessions {
            session_id,
            today,
            limit,
        } => {
            handle_sessions(session_id.as_deref(), today, limit, format)?;
        }
        Commands::Watch { profile } => {
            handle_watch(profile.as_deref())?;
        }
        Commands::Hud { action } => match action {
            HudAction::Install { profile } => {
                handle_hud_install(profile.as_deref())?;
            }
            HudAction::Uninstall { profile } => {
                handle_hud_uninstall(profile.as_deref())?;
            }
            HudAction::Status { profile } => {
                handle_hud_status(profile.as_deref())?;
            }
        },
    }

    Ok(())
}
