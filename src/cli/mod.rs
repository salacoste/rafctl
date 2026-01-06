mod auth;
pub mod profile;
mod run;
mod status;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use std::io;

#[derive(Parser)]
#[command(name = "rafctl", version, about = "AI Coding Agent Profile Manager ☕")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Manage profiles")]
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },
    #[command(about = "Authentication commands")]
    Auth {
        #[command(subcommand)]
        action: AuthAction,
    },
    #[command(about = "Run tool with profile")]
    Run {
        #[arg(help = "Profile name (uses last used if not specified)")]
        profile: Option<String>,
    },
    #[command(about = "Show status of profiles")]
    Status {
        #[arg(help = "Specific profile (shows all if not specified)")]
        profile: Option<String>,
    },
    #[command(about = "Generate shell completions")]
    Completion {
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[derive(Subcommand)]
pub enum ProfileAction {
    #[command(about = "Add a new profile")]
    Add {
        name: String,
        #[arg(long, help = "Tool type: claude or codex")]
        tool: String,
    },
    #[command(about = "List all profiles")]
    List,
    #[command(about = "Remove a profile")]
    Remove { name: String },
    #[command(about = "Show profile details")]
    Show { name: String },
}

#[derive(Subcommand)]
pub enum AuthAction {
    #[command(about = "Login to a profile")]
    Login { profile: String },
    #[command(about = "Logout from a profile")]
    Logout { profile: String },
    #[command(about = "Check auth status")]
    Status {
        #[arg(help = "Profile name (shows all if not specified)")]
        profile: Option<String>,
    },
}

pub fn generate_completions(shell: Shell) {
    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "rafctl", &mut io::stdout());
}
