//! rafctl - AI Coding Agent Profile Manager
//!
//! Smooth as vanilla raf ☕

use std::process::ExitCode;

use colored::Colorize;
use rafctl::run;

fn main() -> ExitCode {
    if let Err(e) = run() {
        // Check if it's our custom error type for better formatting
        if let Some(rafctl_err) = e.downcast_ref::<rafctl::error::RafctlError>() {
            eprintln!("{} {}", "✗".red(), rafctl_err);

            // Provide helpful hints for common errors
            match rafctl_err {
                rafctl::error::RafctlError::ProfileNotFound(name) => {
                    eprintln!(
                        "{}",
                        "  Run 'rafctl profile list' to see available profiles".dimmed()
                    );
                    // Try to suggest similar profile
                    if let Ok(profiles) = rafctl::core::profile::list_profiles() {
                        if let Some(suggestion) =
                            rafctl::core::profile::find_similar_profile(name, &profiles)
                        {
                            eprintln!("{}", format!("  Did you mean '{}'?", suggestion).dimmed());
                        }
                    }
                }
                rafctl::error::RafctlError::NotAuthenticated(name) => {
                    eprintln!(
                        "{}",
                        format!("  Run 'rafctl auth login {}' to authenticate", name).dimmed()
                    );
                }
                rafctl::error::RafctlError::NoApiKey(name) => {
                    eprintln!(
                        "{}",
                        format!("  Run 'rafctl auth set-key {}' to configure API key", name)
                            .dimmed()
                    );
                }
                rafctl::error::RafctlError::ToolNotFound { tool, install_url } => {
                    eprintln!(
                        "{}",
                        format!("  Install {}: {}", tool, install_url).dimmed()
                    );
                }
                rafctl::error::RafctlError::OAuthConflict => {
                    eprintln!("{}", "  Another OAuth profile is already running.".dimmed());
                    eprintln!(
                        "{}",
                        "  Close the other instance first, or use API key mode for parallel execution.".dimmed()
                    );
                }
                _ => {}
            }
        } else {
            // Generic error fallback
            eprintln!("{} {}", "✗".red(), e);

            // Print source chain if available
            let mut source = e.source();
            while let Some(err) = source {
                eprintln!("  {} {}", "caused by:".dimmed(), err);
                source = err.source();
            }
        }
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
