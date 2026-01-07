pub mod analytics;
pub mod auth;
pub mod config;
pub mod dashboard;
pub mod hud;
pub mod output;
pub mod profile;
pub mod quota;
pub mod run;
pub mod sessions;
pub mod status;
pub mod watch;

use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{generate, Shell};
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
pub enum OutputFormat {
    #[default]
    Human,
    Json,
    Plain,
}

#[derive(Parser)]
#[command(name = "rafctl", version, about = "AI Coding Agent Profile Manager ☕")]
pub struct Cli {
    #[arg(long, global = true, help = "Output as JSON")]
    pub json: bool,

    #[arg(long, global = true, help = "Plain output (no colors or emoji)")]
    pub plain: bool,

    #[command(subcommand)]
    pub command: Commands,
}

impl Cli {
    pub fn output_format(&self) -> OutputFormat {
        if self.json {
            OutputFormat::Json
        } else if self.plain || std::env::var("NO_COLOR").is_ok() {
            OutputFormat::Plain
        } else {
            OutputFormat::Human
        }
    }
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
        #[arg(last = true, help = "Arguments to pass to the tool")]
        args: Vec<String>,
    },
    #[command(about = "Show status of profiles")]
    Status {
        #[arg(help = "Specific profile (shows all if not specified)")]
        profile: Option<String>,
    },
    #[command(about = "Show quota/usage limits")]
    Quota {
        #[arg(help = "Specific profile (shows all if not specified)")]
        profile: Option<String>,
    },
    #[command(about = "Configuration management")]
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    #[command(about = "Generate shell completions")]
    Completion {
        #[arg(value_enum)]
        shell: Shell,
    },
    #[command(about = "Interactive TUI dashboard")]
    Dashboard,
    #[command(about = "Switch to a profile (set as default and show status)")]
    Switch {
        #[arg(help = "Profile name to switch to")]
        profile: String,
    },
    #[command(about = "Show usage analytics from local stats")]
    Analytics {
        #[arg(help = "Profile name (uses default if not specified)")]
        profile: Option<String>,
        #[arg(long, default_value = "7", help = "Number of days to show")]
        days: usize,
        #[arg(long, help = "Show all profiles")]
        all: bool,
        #[arg(long, help = "Show estimated costs")]
        cost: bool,
    },
    #[command(about = "View past Claude Code sessions")]
    Sessions {
        #[arg(help = "Session ID to show details (lists recent if not specified)")]
        session_id: Option<String>,
        #[arg(long, help = "Show only today's sessions")]
        today: bool,
        #[arg(long, default_value = "10", help = "Number of sessions to show")]
        limit: usize,
    },
    #[command(about = "Watch Claude Code session in real-time")]
    Watch {
        #[arg(help = "Profile name (uses most recent session if not specified)")]
        profile: Option<String>,
    },
    #[command(about = "Manage HUD statusline plugin")]
    Hud {
        #[command(subcommand)]
        action: HudAction,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    #[command(about = "Show current configuration")]
    Show,
    #[command(about = "Set default profile")]
    SetDefault { profile: String },
    #[command(about = "Clear default profile")]
    ClearDefault,
    #[command(about = "Show configuration file path")]
    Path,
    #[command(about = "Configure HUD statusline integration")]
    Hud {
        #[arg(long, help = "Enable HUD for profile")]
        enable: bool,
        #[arg(long, help = "Disable HUD for profile")]
        disable: bool,
        #[arg(help = "Profile name (uses default if not specified)")]
        profile: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ProfileAction {
    #[command(about = "Add a new profile")]
    Add {
        name: String,
        #[arg(long, help = "Tool type: claude or codex")]
        tool: String,
        #[arg(long, help = "Auth mode for Claude: oauth (default) or api-key")]
        auth_mode: Option<String>,
    },
    #[command(about = "List all profiles")]
    List,
    #[command(about = "Remove a profile")]
    Remove {
        name: String,
        #[arg(long, short = 'y', help = "Skip confirmation prompt")]
        yes: bool,
    },
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
    #[command(about = "Set API key for a profile")]
    SetKey {
        profile: String,
        #[arg(long, help = "API key (prompts if not provided)")]
        key: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum HudAction {
    #[command(about = "Install HUD statusline plugin")]
    Install {
        #[arg(help = "Profile name (installs globally if not specified)")]
        profile: Option<String>,
    },
    #[command(about = "Uninstall HUD statusline plugin")]
    Uninstall {
        #[arg(help = "Profile name (uninstalls globally if not specified)")]
        profile: Option<String>,
    },
    #[command(about = "Show HUD installation status")]
    Status {
        #[arg(help = "Profile name (shows global if not specified)")]
        profile: Option<String>,
    },
}

pub fn generate_completions(shell: Shell) {
    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "rafctl", &mut io::stdout());
}
