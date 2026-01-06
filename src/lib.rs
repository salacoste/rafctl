//! rafctl library - core functionality for AI coding agent profile management.

pub mod cli;
pub mod core;
pub mod error;
pub mod tools;

use anyhow::Result;
use clap::Parser;

use crate::cli::{AuthAction, Cli, Commands, ProfileAction};

/// Main entry point for the CLI application.
pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Profile { action } => match action {
            ProfileAction::Add { name, tool } => {
                println!("Profile add: {} --tool {} (not implemented)", name, tool);
            }
            ProfileAction::List => {
                println!("Profile list (not implemented)");
            }
            ProfileAction::Remove { name } => {
                println!("Profile remove: {} (not implemented)", name);
            }
            ProfileAction::Show { name } => {
                println!("Profile show: {} (not implemented)", name);
            }
        },
        Commands::Auth { action } => match action {
            AuthAction::Login { profile } => {
                println!("Auth login: {} (not implemented)", profile);
            }
            AuthAction::Logout { profile } => {
                println!("Auth logout: {} (not implemented)", profile);
            }
            AuthAction::Status { profile } => {
                println!(
                    "Auth status: {} (not implemented)",
                    profile.unwrap_or_else(|| "all".to_string())
                );
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
