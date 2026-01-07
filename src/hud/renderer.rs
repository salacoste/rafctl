//! Statusline renderer for Claude Code HUD.

use std::path::Path;

use colored::Colorize;

use super::context_color;
use crate::core::transcript::SessionSummary;

const BAR_FILLED: char = '█';
const BAR_EMPTY: char = '░';
const BAR_WIDTH: usize = 10;

pub fn render_statusline(
    profile: Option<&str>,
    cwd: Option<&Path>,
    model: Option<&str>,
    context_percent: u8,
    git_branch: Option<&str>,
    config_count: usize,
    session: Option<&SessionSummary>,
) -> String {
    let mut parts: Vec<String> = Vec::new();

    if let Some(p) = profile {
        parts.push(format!("[{}]", p.cyan()));
    }

    if let Some(dir) = cwd {
        let name = dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project");
        parts.push(format!("📁 {}", name));
    }

    if let Some(m) = model {
        parts.push(format!("[{}]", m.bold()));
    }

    let bar = render_progress_bar(context_percent);
    let color = context_color(context_percent);
    let colored_bar = match color {
        "red" => bar.red().to_string(),
        "yellow" => bar.yellow().to_string(),
        _ => bar.green().to_string(),
    };
    parts.push(format!("{} {}%", colored_bar, context_percent));

    if let Some(branch) = git_branch {
        parts.push(format!("git:({})", branch.magenta()));
    }

    if config_count > 0 {
        parts.push(format!("⚙️{}", config_count));
    }

    if let Some(s) = session {
        if s.tool_calls > 0 {
            let error_str = if s.tool_errors > 0 {
                format!(" {}", format!("({}!)", s.tool_errors).red())
            } else {
                String::new()
            };
            parts.push(format!("🔧{}{}", s.tool_calls, error_str));
        }
    }

    parts.join(" | ")
}

fn render_progress_bar(percent: u8) -> String {
    let filled = ((percent as f64 / 100.0) * BAR_WIDTH as f64).round() as usize;
    let empty = BAR_WIDTH.saturating_sub(filled);

    format!(
        "{}{}",
        BAR_FILLED.to_string().repeat(filled),
        BAR_EMPTY.to_string().repeat(empty)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_progress_bar_empty() {
        assert_eq!(render_progress_bar(0), "░░░░░░░░░░");
    }

    #[test]
    fn test_render_progress_bar_half() {
        assert_eq!(render_progress_bar(50), "█████░░░░░");
    }

    #[test]
    fn test_render_progress_bar_full() {
        assert_eq!(render_progress_bar(100), "██████████");
    }

    #[test]
    fn test_render_statusline_minimal() {
        let output = render_statusline(None, None, None, 45, None, 0, None);
        assert!(output.contains("45%"));
    }

    #[test]
    fn test_render_statusline_with_profile() {
        let output = render_statusline(
            Some("work"),
            None,
            Some("sonnet-4-5"),
            70,
            Some("main"),
            2,
            None,
        );
        assert!(output.contains("work"));
        assert!(output.contains("sonnet-4-5"));
        assert!(output.contains("70%"));
        assert!(output.contains("main"));
    }
}
