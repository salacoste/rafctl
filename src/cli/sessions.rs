//! Sessions command handler - displays past Claude Code sessions

use std::path::PathBuf;

use chrono::{DateTime, Local, Utc};
use colored::Colorize;
use comfy_table::{presets::UTF8_FULL_CONDENSED, Cell, Color, Table};
use serde::Serialize;

use super::output::print_json;
use super::OutputFormat;
use crate::core::transcript::{
    get_global_transcripts_dir, list_sessions, parse_transcript, SessionDetail,
};
use crate::error::RafctlError;

#[derive(Debug, Serialize)]
struct SessionsListOutput {
    sessions: Vec<SessionRow>,
    total: usize,
}

#[derive(Debug, Serialize)]
struct SessionRow {
    session_id: String,
    started_at: Option<String>,
    duration: Option<String>,
    messages: u64,
    tool_calls: u64,
    errors: u64,
    model: Option<String>,
}

#[derive(Debug, Serialize)]
struct SessionDetailOutput {
    session_id: String,
    started_at: Option<String>,
    ended_at: Option<String>,
    duration: Option<String>,
    cwd: Option<String>,
    git_branch: Option<String>,
    model: Option<String>,
    messages: u64,
    tool_calls: u64,
    tool_errors: u64,
    agent_calls: u64,
    tool_breakdown: Vec<ToolBreakdownEntry>,
}

#[derive(Debug, Serialize)]
struct ToolBreakdownEntry {
    tool: String,
    count: u64,
    percentage: f64,
}

pub fn handle_sessions(
    session_id: Option<&str>,
    today_only: bool,
    limit: usize,
    format: OutputFormat,
) -> Result<(), RafctlError> {
    if let Some(sid) = session_id {
        show_session_detail(sid, format)
    } else {
        show_session_list(today_only, limit, format)
    }
}

fn show_session_list(
    today_only: bool,
    limit: usize,
    format: OutputFormat,
) -> Result<(), RafctlError> {
    let transcripts_dir = get_global_transcripts_dir().ok_or_else(|| RafctlError::ConfigRead {
        path: PathBuf::from("~/.claude/projects"),
        source: std::io::Error::new(std::io::ErrorKind::NotFound, "Home directory not found"),
    })?;

    if !transcripts_dir.exists() {
        match format {
            OutputFormat::Json => {
                print_json(&SessionsListOutput {
                    sessions: vec![],
                    total: 0,
                });
            }
            _ => {
                println!(
                    "{} No sessions found. Run Claude Code to create sessions.",
                    "ℹ".cyan()
                );
            }
        }
        return Ok(());
    }

    let mut all_sessions: Vec<(PathBuf, SessionDetail)> = Vec::new();

    if let Ok(projects) = std::fs::read_dir(&transcripts_dir) {
        for project in projects.flatten() {
            let project_path = project.path();
            if project_path.is_dir() {
                let session_files = list_sessions(&project_path);
                for file in session_files {
                    if let Some(detail) = parse_transcript(&file) {
                        if today_only {
                            if let Some(started) = detail.summary.started_at {
                                let today = Utc::now().date_naive();
                                if started.date_naive() != today {
                                    continue;
                                }
                            } else {
                                continue;
                            }
                        }
                        all_sessions.push((file, detail));
                    }
                }
            }
        }
    }

    all_sessions.sort_by(|a, b| {
        let a_time = a.1.summary.started_at;
        let b_time = b.1.summary.started_at;
        b_time.cmp(&a_time)
    });

    let sessions: Vec<SessionRow> = all_sessions
        .iter()
        .take(limit)
        .map(|(_, detail)| {
            let duration = calculate_duration(detail.summary.started_at, detail.summary.ended_at);

            SessionRow {
                session_id: shorten_session_id(&detail.summary.session_id),
                started_at: detail.summary.started_at.map(|dt| {
                    dt.with_timezone(&Local)
                        .format("%Y-%m-%d %H:%M")
                        .to_string()
                }),
                duration,
                messages: detail.summary.message_count,
                tool_calls: detail.summary.tool_calls,
                errors: detail.summary.tool_errors,
                model: detail.summary.model.as_ref().map(|m| shorten_model(m)),
            }
        })
        .collect();

    let total = all_sessions.len();

    match format {
        OutputFormat::Json => {
            print_json(&SessionsListOutput { sessions, total });
        }
        OutputFormat::Plain => {
            println!("SESSION_ID\tSTARTED\tDURATION\tMESSAGES\tTOOLS\tERRORS");
            for s in &sessions {
                println!(
                    "{}\t{}\t{}\t{}\t{}\t{}",
                    s.session_id,
                    s.started_at.as_deref().unwrap_or("-"),
                    s.duration.as_deref().unwrap_or("-"),
                    s.messages,
                    s.tool_calls,
                    s.errors
                );
            }
        }
        OutputFormat::Human => {
            let title = if today_only {
                "Today's Sessions"
            } else {
                "Recent Sessions"
            };

            println!("\n{} {} ({} total)\n", "📋".cyan(), title.bold(), total);

            if sessions.is_empty() {
                println!("No sessions found.");
                return Ok(());
            }

            let mut table = Table::new();
            table.load_preset(UTF8_FULL_CONDENSED);
            table.set_header(vec![
                "Session ID",
                "Started",
                "Duration",
                "Messages",
                "Tools",
                "Errors",
            ]);

            for s in &sessions {
                let error_cell = if s.errors > 0 {
                    Cell::new(s.errors).fg(Color::Red)
                } else {
                    Cell::new(s.errors).fg(Color::Green)
                };

                table.add_row(vec![
                    Cell::new(&s.session_id).fg(Color::Cyan),
                    Cell::new(s.started_at.as_deref().unwrap_or("-")),
                    Cell::new(s.duration.as_deref().unwrap_or("-")),
                    Cell::new(s.messages),
                    Cell::new(s.tool_calls),
                    error_cell,
                ]);
            }

            println!("{table}\n");

            if total > limit {
                println!(
                    "{}",
                    format!(
                        "Showing {} of {} sessions. Use --limit to see more.",
                        limit, total
                    )
                    .dimmed()
                );
            }
        }
    }

    Ok(())
}

fn show_session_detail(session_id: &str, format: OutputFormat) -> Result<(), RafctlError> {
    let transcripts_dir = get_global_transcripts_dir().ok_or_else(|| RafctlError::ConfigRead {
        path: PathBuf::from("~/.claude/projects"),
        source: std::io::Error::new(std::io::ErrorKind::NotFound, "Home directory not found"),
    })?;

    let mut found_detail: Option<SessionDetail> = None;

    if let Ok(projects) = std::fs::read_dir(&transcripts_dir) {
        'outer: for project in projects.flatten() {
            let project_path = project.path();
            if project_path.is_dir() {
                let session_files = list_sessions(&project_path);
                for file in session_files {
                    if let Some(detail) = parse_transcript(&file) {
                        if detail.summary.session_id.starts_with(session_id)
                            || detail.summary.session_id.ends_with(session_id)
                            || detail.summary.session_id.contains(session_id)
                        {
                            found_detail = Some(detail);
                            break 'outer;
                        }
                    }
                }
            }
        }
    }

    let detail = found_detail.ok_or_else(|| {
        RafctlError::ProfileNotFound(format!("Session '{}' not found", session_id))
    })?;

    let duration = calculate_duration(detail.summary.started_at, detail.summary.ended_at);

    let mut tool_breakdown: Vec<ToolBreakdownEntry> = detail
        .tool_breakdown
        .iter()
        .map(|(tool, &count)| {
            let percentage = if detail.summary.tool_calls > 0 {
                (count as f64 / detail.summary.tool_calls as f64) * 100.0
            } else {
                0.0
            };
            ToolBreakdownEntry {
                tool: tool.clone(),
                count,
                percentage,
            }
        })
        .collect();

    tool_breakdown.sort_by(|a, b| b.count.cmp(&a.count));

    let output = SessionDetailOutput {
        session_id: detail.summary.session_id.clone(),
        started_at: detail.summary.started_at.map(|dt| {
            dt.with_timezone(&Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        }),
        ended_at: detail.summary.ended_at.map(|dt| {
            dt.with_timezone(&Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        }),
        duration,
        cwd: detail.summary.cwd.clone(),
        git_branch: detail.summary.git_branch.clone(),
        model: detail.summary.model.clone(),
        messages: detail.summary.message_count,
        tool_calls: detail.summary.tool_calls,
        tool_errors: detail.summary.tool_errors,
        agent_calls: detail.summary.agent_calls,
        tool_breakdown,
    };

    match format {
        OutputFormat::Json => {
            print_json(&output);
        }
        OutputFormat::Plain => {
            println!("SESSION_ID\t{}", output.session_id);
            println!("STARTED\t{}", output.started_at.as_deref().unwrap_or("-"));
            println!("ENDED\t{}", output.ended_at.as_deref().unwrap_or("-"));
            println!("DURATION\t{}", output.duration.as_deref().unwrap_or("-"));
            println!("CWD\t{}", output.cwd.as_deref().unwrap_or("-"));
            println!("BRANCH\t{}", output.git_branch.as_deref().unwrap_or("-"));
            println!("MODEL\t{}", output.model.as_deref().unwrap_or("-"));
            println!("MESSAGES\t{}", output.messages);
            println!("TOOLS\t{}", output.tool_calls);
            println!("ERRORS\t{}", output.tool_errors);
            println!("AGENTS\t{}", output.agent_calls);
        }
        OutputFormat::Human => {
            println!(
                "\n{} Session Details — {}\n",
                "📋".cyan(),
                shorten_session_id(&output.session_id).bold()
            );

            println!(
                "Started:     {}",
                output.started_at.as_deref().unwrap_or("-")
            );
            println!("Ended:       {}", output.ended_at.as_deref().unwrap_or("-"));
            println!("Duration:    {}", output.duration.as_deref().unwrap_or("-"));
            println!("Directory:   {}", output.cwd.as_deref().unwrap_or("-"));
            println!(
                "Git Branch:  {}",
                output.git_branch.as_deref().unwrap_or("-")
            );
            println!("Model:       {}", output.model.as_deref().unwrap_or("-"));
            println!();

            println!("Messages:    {}", output.messages.to_string().cyan());
            println!(
                "Tool Calls:  {} ({} errors)",
                output.tool_calls.to_string().cyan(),
                if output.tool_errors > 0 {
                    output.tool_errors.to_string().red().to_string()
                } else {
                    output.tool_errors.to_string().green().to_string()
                }
            );
            println!("Agent Calls: {}", output.agent_calls.to_string().cyan());
            println!();

            if !output.tool_breakdown.is_empty() {
                println!("{}", "Tool Breakdown:".bold());
                for entry in &output.tool_breakdown {
                    let bar = progress_bar(entry.percentage, 10);
                    println!(
                        "  {} {:<12} {:>4} calls ({:.0}%)",
                        bar, entry.tool, entry.count, entry.percentage
                    );
                }
                println!();
            }
        }
    }

    Ok(())
}

fn calculate_duration(start: Option<DateTime<Utc>>, end: Option<DateTime<Utc>>) -> Option<String> {
    match (start, end) {
        (Some(s), Some(e)) => {
            let duration = e - s;
            let secs = duration.num_seconds();
            if secs < 60 {
                Some(format!("{}s", secs))
            } else if secs < 3600 {
                Some(format!("{}m", secs / 60))
            } else {
                let hours = secs / 3600;
                let mins = (secs % 3600) / 60;
                Some(format!("{}h {}m", hours, mins))
            }
        }
        _ => None,
    }
}

fn shorten_session_id(id: &str) -> String {
    if id.len() > 12 {
        format!("{}...", &id[..12])
    } else {
        id.to_string()
    }
}

fn shorten_model(model: &str) -> String {
    model
        .replace("claude-", "")
        .replace("-20", " ")
        .split_whitespace()
        .next()
        .unwrap_or(model)
        .to_string()
}

fn progress_bar(percentage: f64, width: usize) -> String {
    let filled = ((percentage / 100.0) * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}
