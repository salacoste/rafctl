//! Transcript parser for Claude Code JSONL session files.
//!
//! Parses transcript files from `~/.claude/projects/<project>/` to extract:
//! - Session metadata (id, timestamps, cwd, git branch)
//! - Tool usage (name, target, status, duration)
//! - Agent calls (subagent_type, description)
//! - Error counts

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct SessionSummary {
    pub session_id: String,
    pub project_path: Option<String>,
    pub cwd: Option<String>,
    pub git_branch: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub message_count: u64,
    pub tool_calls: u64,
    pub tool_errors: u64,
    pub agent_calls: u64,
    pub model: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub target: Option<String>,
    pub timestamp: Option<DateTime<Utc>>,
    pub is_error: bool,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct AgentCall {
    pub subagent_type: Option<String>,
    pub description: Option<String>,
    pub timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct SessionDetail {
    pub summary: SessionSummary,
    pub tool_calls: Vec<ToolCall>,
    pub agent_calls: Vec<AgentCall>,
    pub tool_breakdown: HashMap<String, u64>,
}

#[derive(Debug, Deserialize)]
struct TranscriptEntry {
    #[serde(rename = "type")]
    entry_type: Option<String>,
    timestamp: Option<String>,
    #[serde(rename = "sessionId")]
    session_id: Option<String>,
    cwd: Option<String>,
    #[serde(rename = "gitBranch")]
    git_branch: Option<String>,
    message: Option<TranscriptMessage>,
}

#[derive(Debug, Deserialize)]
struct TranscriptMessage {
    #[allow(dead_code)]
    role: Option<String>,
    model: Option<String>,
    content: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct ToolUseBlock {
    id: Option<String>,
    name: Option<String>,
    input: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct ToolResultBlock {
    tool_use_id: Option<String>,
    is_error: Option<bool>,
}

pub fn parse_transcript(path: &Path) -> Option<SessionDetail> {
    let file = File::open(path).ok()?;
    let reader = BufReader::new(file);

    let mut summary = SessionSummary {
        session_id: String::new(),
        project_path: None,
        cwd: None,
        git_branch: None,
        started_at: None,
        ended_at: None,
        message_count: 0,
        tool_calls: 0,
        tool_errors: 0,
        agent_calls: 0,
        model: None,
    };

    let mut tool_calls: Vec<ToolCall> = Vec::new();
    let mut agent_calls: Vec<AgentCall> = Vec::new();
    let mut tool_breakdown: HashMap<String, u64> = HashMap::new();
    let mut pending_tools: HashMap<String, ToolCall> = HashMap::new();

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };

        if line.trim().is_empty() {
            continue;
        }

        let entry: TranscriptEntry = match serde_json::from_str(&line) {
            Ok(e) => e,
            Err(_) => continue,
        };

        let timestamp = entry
            .timestamp
            .as_ref()
            .and_then(|t| DateTime::parse_from_rfc3339(t).ok())
            .map(|dt| dt.with_timezone(&Utc));

        if let Some(sid) = &entry.session_id {
            if summary.session_id.is_empty() {
                summary.session_id = sid.clone();
            }
        }

        if summary.cwd.is_none() {
            summary.cwd = entry.cwd.clone();
        }

        if summary.git_branch.is_none() {
            summary.git_branch = entry.git_branch.clone();
        }

        if summary.started_at.is_none() {
            summary.started_at = timestamp;
        }
        summary.ended_at = timestamp;

        let entry_type = entry.entry_type.as_deref().unwrap_or("");

        if entry_type == "user" || entry_type == "assistant" {
            summary.message_count += 1;
        }

        if let Some(msg) = &entry.message {
            if summary.model.is_none() {
                summary.model = msg.model.clone();
            }

            if let Some(content) = &msg.content {
                if let Some(blocks) = content.as_array() {
                    for block in blocks {
                        let block_type = block.get("type").and_then(|t| t.as_str());

                        match block_type {
                            Some("tool_use") => {
                                if let Ok(tool_use) =
                                    serde_json::from_value::<ToolUseBlock>(block.clone())
                                {
                                    let name = tool_use.name.unwrap_or_default();
                                    let id = tool_use.id.unwrap_or_default();
                                    let target = extract_tool_target(&name, &tool_use.input);

                                    if name == "Task" {
                                        summary.agent_calls += 1;
                                        let agent_call = AgentCall {
                                            subagent_type: tool_use
                                                .input
                                                .as_ref()
                                                .and_then(|i| i.get("subagent_type"))
                                                .and_then(|v| v.as_str())
                                                .map(|s| s.to_string()),
                                            description: tool_use
                                                .input
                                                .as_ref()
                                                .and_then(|i| i.get("description"))
                                                .and_then(|v| v.as_str())
                                                .map(|s| s.to_string()),
                                            timestamp,
                                        };
                                        agent_calls.push(agent_call);
                                    } else {
                                        summary.tool_calls += 1;
                                        *tool_breakdown.entry(name.clone()).or_insert(0) += 1;

                                        let tool_call = ToolCall {
                                            id: id.clone(),
                                            name,
                                            target,
                                            timestamp,
                                            is_error: false,
                                            duration_ms: None,
                                        };
                                        pending_tools.insert(id, tool_call);
                                    }
                                }
                            }
                            Some("tool_result") => {
                                if let Ok(result) =
                                    serde_json::from_value::<ToolResultBlock>(block.clone())
                                {
                                    if let Some(tool_id) = result.tool_use_id {
                                        if let Some(mut tool_call) = pending_tools.remove(&tool_id)
                                        {
                                            let is_error = result.is_error.unwrap_or(false);
                                            tool_call.is_error = is_error;
                                            if is_error {
                                                summary.tool_errors += 1;
                                            }
                                            if let (Some(start), Some(end)) =
                                                (tool_call.timestamp, timestamp)
                                            {
                                                tool_call.duration_ms =
                                                    Some((end - start).num_milliseconds().max(0)
                                                        as u64);
                                            }
                                            tool_calls.push(tool_call);
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    for (_, tool_call) in pending_tools {
        tool_calls.push(tool_call);
    }

    if summary.session_id.is_empty() {
        return None;
    }

    Some(SessionDetail {
        summary,
        tool_calls,
        agent_calls,
        tool_breakdown,
    })
}

fn extract_tool_target(tool_name: &str, input: &Option<Value>) -> Option<String> {
    let input = input.as_ref()?;

    match tool_name {
        "Read" | "Write" | "Edit" => input
            .get("file_path")
            .or_else(|| input.get("filePath"))
            .or_else(|| input.get("path"))
            .and_then(|v| v.as_str())
            .map(truncate_path),
        "Glob" | "Grep" => input
            .get("pattern")
            .and_then(|v| v.as_str())
            .map(|s| truncate_str(s, 40)),
        "Bash" => input
            .get("command")
            .and_then(|v| v.as_str())
            .map(|s| truncate_str(s, 30)),
        "Task" => input
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| truncate_str(s, 40)),
        _ => None,
    }
}

fn truncate_path(path: &str) -> String {
    if let Some(filename) = Path::new(path).file_name() {
        filename.to_string_lossy().to_string()
    } else {
        truncate_str(path, 30)
    }
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_len.saturating_sub(3)).collect();
        format!("{}...", truncated)
    }
}

pub fn list_sessions(project_dir: &Path) -> Vec<PathBuf> {
    let mut sessions: Vec<PathBuf> = Vec::new();

    if let Ok(entries) = std::fs::read_dir(project_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "jsonl").unwrap_or(false) {
                let filename = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                if !filename.starts_with("agent-") {
                    sessions.push(path);
                }
            }
        }
    }

    sessions.sort_by(|a, b| {
        let a_time = std::fs::metadata(a).and_then(|m| m.modified()).ok();
        let b_time = std::fs::metadata(b).and_then(|m| m.modified()).ok();
        b_time.cmp(&a_time)
    });

    sessions
}

pub fn get_global_transcripts_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude").join("projects"))
}

pub fn get_profile_transcripts_dir(profile_name: &str) -> Option<PathBuf> {
    dirs::home_dir().map(|h| {
        h.join(".rafctl")
            .join("profiles")
            .join(profile_name)
            .join("claude")
            .join("projects")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_str() {
        assert_eq!(truncate_str("hello", 10), "hello");
        assert_eq!(truncate_str("hello world", 8), "hello...");
        // UTF-8 test: Cyrillic characters should not panic
        assert_eq!(truncate_str("Привет мир", 8), "Приве...");
        // Edge cases
        assert_eq!(truncate_str("", 5), "");
        assert_eq!(truncate_str("abc", 3), "abc");
    }

    #[test]
    fn test_truncate_path() {
        assert_eq!(truncate_path("/foo/bar/baz.rs"), "baz.rs");
        assert_eq!(truncate_path("baz.rs"), "baz.rs");
    }

    #[test]
    fn test_extract_tool_target_read() {
        let input = Some(serde_json::json!({
            "file_path": "/Users/test/project/src/main.rs"
        }));
        let target = extract_tool_target("Read", &input);
        assert_eq!(target, Some("main.rs".to_string()));
    }

    #[test]
    fn test_extract_tool_target_bash() {
        let input = Some(serde_json::json!({
            "command": "cargo build --release"
        }));
        let target = extract_tool_target("Bash", &input);
        assert_eq!(target, Some("cargo build --release".to_string()));
    }

    #[test]
    fn test_extract_tool_target_grep() {
        let input = Some(serde_json::json!({
            "pattern": "fn main"
        }));
        let target = extract_tool_target("Grep", &input);
        assert_eq!(target, Some("fn main".to_string()));
    }

    #[test]
    fn test_session_summary_default() {
        let summary = SessionSummary {
            session_id: "test-123".to_string(),
            project_path: None,
            cwd: Some("/test".to_string()),
            git_branch: Some("main".to_string()),
            started_at: None,
            ended_at: None,
            message_count: 10,
            tool_calls: 5,
            tool_errors: 1,
            agent_calls: 2,
            model: Some("claude-sonnet".to_string()),
        };

        assert_eq!(summary.session_id, "test-123");
        assert_eq!(summary.message_count, 10);
        assert_eq!(summary.tool_errors, 1);
    }
}
