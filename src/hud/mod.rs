//! HUD (Heads-Up Display) module for Claude Code statusline integration.
//!
//! This module provides a native Rust implementation of the Claude Code statusline protocol,
//! designed to be a drop-in replacement for Node.js-based HUD plugins.

mod renderer;
mod stdin;

pub use renderer::render_statusline;
pub use stdin::{parse_stdin, StdinPayload};

use std::io::{self, Read};
use std::path::Path;
use std::process::Command;

use crate::core::transcript::parse_transcript;

const AUTOCOMPACT_BUFFER: u64 = 45_000;
const THRESHOLD_YELLOW: u8 = 70;
const THRESHOLD_RED: u8 = 85;

pub fn run_hud() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    if input.trim().is_empty() {
        println!("Initializing...");
        return Ok(());
    }

    let payload = parse_stdin(&input)?;
    let context_percent = calculate_context_percent(&payload);
    let git_branch = get_git_branch(payload.cwd.as_deref());
    let config_count = count_configs(payload.cwd.as_deref());
    let model_name = extract_model_name(&payload);
    let profile = std::env::var("RAFCTL_PROFILE").ok();

    let session_summary = payload
        .transcript_path
        .as_ref()
        .and_then(|p| parse_transcript(p))
        .map(|d| d.summary);

    let output = render_statusline(
        profile.as_deref(),
        payload.cwd.as_deref(),
        model_name.as_deref(),
        context_percent,
        git_branch.as_deref(),
        config_count,
        session_summary.as_ref(),
    );

    println!("{}", output);
    Ok(())
}

fn calculate_context_percent(payload: &StdinPayload) -> u8 {
    let context = match &payload.context_window {
        Some(c) => c,
        None => return 0,
    };

    let size = context.context_window_size;
    if size <= AUTOCOMPACT_BUFFER {
        return 0;
    }

    let usage = match &context.current_usage {
        Some(u) => u,
        None => return 0,
    };

    let total_tokens =
        usage.input_tokens + usage.cache_creation_input_tokens + usage.cache_read_input_tokens;

    let percent = ((total_tokens + AUTOCOMPACT_BUFFER) as f64 / size as f64) * 100.0;
    percent.round().min(100.0) as u8
}

fn get_git_branch(cwd: Option<&Path>) -> Option<String> {
    let dir = cwd?;

    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(dir)
        .output()
        .ok()?;

    if output.status.success() {
        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !branch.is_empty() && branch != "HEAD" {
            return Some(branch);
        }
    }

    None
}

fn count_configs(cwd: Option<&Path>) -> usize {
    let mut count = 0;

    if let Some(home) = dirs::home_dir() {
        let claude_dir = home.join(".claude");

        if claude_dir.join("CLAUDE.md").exists() {
            count += 1;
        }
        if claude_dir.join("settings.json").exists() {
            count += 1;
        }

        if let Ok(entries) = std::fs::read_dir(claude_dir.join("rules")) {
            count += entries.filter(|e| e.is_ok()).count();
        }
    }

    if let Some(dir) = cwd {
        if dir.join("CLAUDE.md").exists() {
            count += 1;
        }
        if dir.join("CLAUDE.local.md").exists() {
            count += 1;
        }
        if dir.join(".claude").join("CLAUDE.md").exists() {
            count += 1;
        }
        if dir.join(".mcp.json").exists() {
            count += 1;
        }
        if dir.join(".claude").join("settings.local.json").exists() {
            count += 1;
        }
    }

    count
}

fn extract_model_name(payload: &StdinPayload) -> Option<String> {
    payload.model.as_ref().map(|m| {
        m.name
            .replace("claude-", "")
            .replace("-20", " ")
            .split_whitespace()
            .next()
            .unwrap_or(&m.name)
            .to_string()
    })
}

pub fn context_color(percent: u8) -> &'static str {
    if percent >= THRESHOLD_RED {
        "red"
    } else if percent >= THRESHOLD_YELLOW {
        "yellow"
    } else {
        "green"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_color_green() {
        assert_eq!(context_color(0), "green");
        assert_eq!(context_color(50), "green");
        assert_eq!(context_color(69), "green");
    }

    #[test]
    fn test_context_color_yellow() {
        assert_eq!(context_color(70), "yellow");
        assert_eq!(context_color(80), "yellow");
        assert_eq!(context_color(84), "yellow");
    }

    #[test]
    fn test_context_color_red() {
        assert_eq!(context_color(85), "red");
        assert_eq!(context_color(90), "red");
        assert_eq!(context_color(100), "red");
    }

    #[test]
    fn test_count_configs_empty() {
        assert_eq!(count_configs(None), count_configs(None));
    }
}
