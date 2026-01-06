//! rafctl library - core functionality for AI coding agent profile management.

pub mod cli;
pub mod core;
pub mod error;
pub mod tools;

use anyhow::Result;
use clap::Parser;

use crate::cli::auth::{
    handle_login, handle_logout, handle_set_key, handle_status as handle_auth_status,
};
use crate::cli::config::{
    handle_clear_default, handle_path as handle_config_path, handle_set_default,
    handle_show as handle_config_show,
};
use crate::cli::profile::{handle_add, handle_list, handle_remove, handle_show};
use crate::cli::run::handle_run;
use crate::cli::status::handle_status;
use crate::cli::{AuthAction, Cli, Commands, ConfigAction, ProfileAction};

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
            ProfileAction::Remove { name } => {
                handle_remove(&name)?;
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
        },
        Commands::Completion { shell } => {
            cli::generate_completions(shell);
        }
    }

    Ok(())
}
