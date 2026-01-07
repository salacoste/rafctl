//! Live session monitor - watches Claude Code sessions in real-time

use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

use chrono::{DateTime, Local};
use colored::Colorize;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};

use crate::core::transcript::{get_global_transcripts_dir, list_sessions};
use crate::error::RafctlError;

pub fn handle_watch(profile: Option<&str>) -> Result<(), RafctlError> {
    let transcripts_dir = get_global_transcripts_dir().ok_or_else(|| RafctlError::ConfigRead {
        path: PathBuf::from("~/.claude/projects"),
        source: std::io::Error::new(std::io::ErrorKind::NotFound, "Home directory not found"),
    })?;

    if !transcripts_dir.exists() {
        println!(
            "{} No sessions found. Start Claude Code to create sessions.",
            "ℹ".cyan()
        );
        return Ok(());
    }

    let session_file = find_most_recent_session(&transcripts_dir)?;
    let session_id = session_file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    let profile_display = profile.unwrap_or("default");

    println!();
    println!(
        "{} {} — Profile: {} — Session: {}",
        "🔴 LIVE".red().bold(),
        "Session Monitor".bold(),
        profile_display.cyan(),
        shorten_id(session_id).cyan()
    );
    println!("{}", "─".repeat(60).dimmed());
    println!("{}", "Press Ctrl+C to stop watching".dimmed());
    println!();

    watch_session_file(&session_file)
}

fn find_most_recent_session(transcripts_dir: &std::path::Path) -> Result<PathBuf, RafctlError> {
    let mut all_sessions: Vec<PathBuf> = Vec::new();

    if let Ok(projects) = std::fs::read_dir(transcripts_dir) {
        for project in projects.flatten() {
            let project_path = project.path();
            if project_path.is_dir() {
                let sessions = list_sessions(&project_path);
                all_sessions.extend(sessions);
            }
        }
    }

    all_sessions.sort_by(|a, b| {
        let a_time = std::fs::metadata(a).and_then(|m| m.modified()).ok();
        let b_time = std::fs::metadata(b).and_then(|m| m.modified()).ok();
        b_time.cmp(&a_time)
    });

    all_sessions.into_iter().next().ok_or_else(|| {
        RafctlError::ProfileNotFound("No session files found. Start Claude Code first.".to_string())
    })
}

fn watch_session_file(path: &PathBuf) -> Result<(), RafctlError> {
    let mut file = File::open(path).map_err(|e| RafctlError::ConfigRead {
        path: path.clone(),
        source: e,
    })?;

    let mut seen_ids: HashSet<String> = HashSet::new();
    let initial_pos = read_existing_entries(&mut file, &mut seen_ids)?;
    file.seek(SeekFrom::Start(initial_pos)).ok();

    let (tx, rx) = channel();

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        Config::default().with_poll_interval(Duration::from_millis(100)),
    )
    .map_err(|e| RafctlError::ProfileNotFound(format!("Failed to create watcher: {}", e)))?;

    watcher
        .watch(path, RecursiveMode::NonRecursive)
        .map_err(|e| RafctlError::ProfileNotFound(format!("Failed to watch file: {}", e)))?;

    watch_loop(&rx, &mut file, &mut seen_ids)?;

    Ok(())
}

fn read_existing_entries(
    file: &mut File,
    seen_ids: &mut HashSet<String>,
) -> Result<u64, RafctlError> {
    let reader = BufReader::new(file.try_clone().unwrap());
    let mut last_pos = 0u64;

    for line in reader.lines().map_while(Result::ok) {
        last_pos += line.len() as u64 + 1;
        if let Ok(entry) = serde_json::from_str::<serde_json::Value>(&line) {
            if let Some(id) = extract_tool_id(&entry) {
                seen_ids.insert(id);
            }
        }
    }

    Ok(last_pos)
}

fn watch_loop(
    rx: &Receiver<Event>,
    file: &mut File,
    seen_ids: &mut HashSet<String>,
) -> Result<(), RafctlError> {
    loop {
        match rx.recv_timeout(Duration::from_millis(500)) {
            Ok(_event) => {
                read_new_lines(file, seen_ids)?;
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                continue;
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                break;
            }
        }
    }
    Ok(())
}

fn read_new_lines(file: &mut File, seen_ids: &mut HashSet<String>) -> Result<(), RafctlError> {
    let reader = BufReader::new(file.try_clone().unwrap());

    for line in reader.lines().map_while(Result::ok) {
        if line.is_empty() {
            continue;
        }

        if let Ok(entry) = serde_json::from_str::<serde_json::Value>(&line) {
            if let Some(id) = extract_tool_id(&entry) {
                if seen_ids.contains(&id) {
                    continue;
                }
                seen_ids.insert(id);
            }

            print_entry(&entry);
        }
    }

    Ok(())
}

fn extract_tool_id(entry: &serde_json::Value) -> Option<String> {
    entry
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|block| block.get("id"))
        .and_then(|id| id.as_str())
        .map(|s| s.to_string())
}

fn print_entry(entry: &serde_json::Value) {
    let timestamp = entry
        .get("timestamp")
        .and_then(|t| t.as_str())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Local).format("%H:%M:%S").to_string())
        .unwrap_or_else(|| "??:??:??".to_string());

    let entry_type = entry.get("type").and_then(|t| t.as_str()).unwrap_or("");

    match entry_type {
        "user" => {
            println!("[{}] {} User message", timestamp.dimmed(), "💬".cyan());
        }
        "assistant" => {
            if let Some(content) = entry.get("message").and_then(|m| m.get("content")) {
                if let Some(blocks) = content.as_array() {
                    for block in blocks {
                        print_content_block(&timestamp, block);
                    }
                }
            }
        }
        _ => {}
    }
}

fn print_content_block(timestamp: &str, block: &serde_json::Value) {
    let block_type = block.get("type").and_then(|t| t.as_str()).unwrap_or("");

    match block_type {
        "tool_use" => {
            let tool_name = block
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("Unknown");
            let target = extract_target(tool_name, block.get("input"));
            let icon = tool_icon(tool_name);

            let target_display = target.map(|t| format!(" → {}", t)).unwrap_or_default();

            println!(
                "[{}] {} {}{}",
                timestamp.dimmed(),
                icon,
                tool_name.yellow(),
                target_display.dimmed()
            );
        }
        "tool_result" => {
            let is_error = block
                .get("is_error")
                .and_then(|e| e.as_bool())
                .unwrap_or(false);

            if is_error {
                println!("[{}] {} Tool error", timestamp.dimmed(), "✗".red());
            }
        }
        "text" => {
            // Skip text blocks in live view
        }
        _ => {}
    }
}

fn extract_target(tool_name: &str, input: Option<&serde_json::Value>) -> Option<String> {
    let input = input?;

    match tool_name {
        "Read" | "Write" | "Edit" => input
            .get("file_path")
            .or_else(|| input.get("path"))
            .or_else(|| input.get("filePath"))
            .and_then(|v| v.as_str())
            .map(truncate_path),
        "Glob" | "Grep" => input
            .get("pattern")
            .and_then(|v| v.as_str())
            .map(|s| truncate_str(s, 30)),
        "Bash" => input
            .get("command")
            .and_then(|v| v.as_str())
            .map(|s| truncate_str(s, 40)),
        "Task" => input
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| truncate_str(s, 40)),
        "TodoWrite" => Some("updating todos".to_string()),
        _ => None,
    }
}

fn truncate_path(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or(path)
        .to_string()
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_len.saturating_sub(3)).collect();
        format!("{}...", truncated)
    }
}

fn tool_icon(name: &str) -> &'static str {
    match name {
        "Read" => "📖",
        "Write" => "📝",
        "Edit" => "✏️",
        "Bash" => "🚀",
        "Glob" => "🔍",
        "Grep" => "🔎",
        "Task" => "🤖",
        "TodoWrite" => "📋",
        "TodoRead" => "📋",
        _ => "🔧",
    }
}

fn shorten_id(id: &str) -> String {
    if id.len() > 12 {
        format!("{}...", &id[..8])
    } else {
        id.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_icon() {
        assert_eq!(tool_icon("Read"), "📖");
        assert_eq!(tool_icon("Bash"), "🚀");
        assert_eq!(tool_icon("Unknown"), "🔧");
    }

    #[test]
    fn test_shorten_id() {
        assert_eq!(shorten_id("abc"), "abc");
        assert_eq!(shorten_id("abcdef123456789"), "abcdef12...");
    }

    #[test]
    fn test_truncate_str_watch() {
        assert_eq!(truncate_str("hello", 10), "hello");
        assert_eq!(truncate_str("hello world test", 10), "hello w...");
    }
}
