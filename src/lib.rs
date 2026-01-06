//! rafctl library - core functionality for AI coding agent profile management.

pub mod cli;
pub mod core;
pub mod error;
pub mod tools;

use anyhow::Result;
use clap::Parser;

use crate::cli::auth::{handle_login, handle_logout, handle_status as handle_auth_status};
use crate::cli::profile::{handle_add, handle_list, handle_remove, handle_show};
use crate::cli::{AuthAction, Cli, Commands, ProfileAction};

/// Main entry point for the CLI application.
pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Profile { action } => match action {
            ProfileAction::Add { name, tool } => {
                handle_add(&name, &tool)?;
            }
            ProfileAction::List => {
                handle_list()?;
            }
            ProfileAction::Remove { name } => {
                handle_remove(&name)?;
            }
            ProfileAction::Show { name } => {
                handle_show(&name)?;
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
        },
        Commands::Run { profile } => {
            println!(
                "Run: {} (not implemented)",
                profile.unwrap_or_else(|| "default".to_string())
            );
        }
        Commands::Status { profile } => {
            println!(
                "Status: {} (not implemented)",
                profile.unwrap_or_else(|| "all".to_string())
            );
        }
        Commands::Completion { shell } => {
            cli::generate_completions(shell);
        }
    }

    Ok(())
}
