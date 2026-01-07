//! Stats cache parser for Claude Code usage statistics.
//!
//! Parses `stats-cache.json` files created by Claude Code to extract
//! historical usage data like daily activity, token counts by model, etc.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

use crate::core::profile::{get_profile_dir, ToolType};
use crate::error::RafctlError;

/// Current schema version of stats-cache.json
const EXPECTED_SCHEMA_VERSION: u32 = 1;

/// Stats cache from Claude Code's local storage.
///
/// Location: `~/.claude/stats-cache.json` (global)
///           or `~/.rafctl/profiles/<name>/claude/stats-cache.json` (per-profile)
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsCache {
    /// Schema version (currently 1)
    pub version: Option<u32>,
    /// Date when stats were last computed (YYYY-MM-DD)
    pub last_computed_date: Option<String>,
    /// Daily activity metrics
    #[serde(default)]
    pub daily_activity: Vec<DailyActivity>,
    /// Daily token usage by model
    #[serde(default)]
    pub daily_model_tokens: Vec<DailyModelTokens>,
    /// Total session count across all time
    pub total_sessions: Option<u64>,
    /// Total message count across all time
    pub total_messages: Option<u64>,
    /// Model usage summary (alternative to daily_model_tokens)
    #[serde(default)]
    pub model_usage: HashMap<String, ModelUsage>,
}

/// Daily activity metrics
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyActivity {
    /// Date in YYYY-MM-DD format
    pub date: String,
    /// Number of messages sent
    #[serde(default)]
    pub message_count: u64,
    /// Number of sessions
    #[serde(default)]
    pub session_count: u64,
    /// Number of tool calls
    #[serde(default)]
    pub tool_call_count: u64,
}

/// Daily token usage by model
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyModelTokens {
    /// Date in YYYY-MM-DD format
    pub date: String,
    /// Token counts per model ID
    #[serde(default)]
    pub tokens_by_model: HashMap<String, u64>,
}

/// Model usage summary
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelUsage {
    /// Input tokens used
    #[serde(default)]
    pub input_tokens: u64,
    /// Output tokens used
    #[serde(default)]
    pub output_tokens: u64,
    /// Estimated cost in USD (if tracked)
    #[serde(default)]
    pub cost_usd: f64,
}

impl StatsCache {
    /// Check if this stats cache is empty (no data)
    pub fn is_empty(&self) -> bool {
        self.daily_activity.is_empty()
            && self.daily_model_tokens.is_empty()
            && self.total_sessions.unwrap_or(0) == 0
    }

    /// Get total tokens for a specific date
    pub fn tokens_for_date(&self, date: &str) -> u64 {
        self.daily_model_tokens
            .iter()
            .find(|d| d.date == date)
            .map(|d| d.tokens_by_model.values().sum())
            .unwrap_or(0)
    }

    /// Get activity for a specific date
    pub fn activity_for_date(&self, date: &str) -> Option<&DailyActivity> {
        self.daily_activity.iter().find(|d| d.date == date)
    }

    /// Get the last N days of activity (most recent first)
    pub fn recent_activity(&self, days: usize) -> Vec<&DailyActivity> {
        let mut sorted: Vec<_> = self.daily_activity.iter().collect();
        sorted.sort_by(|a, b| b.date.cmp(&a.date));
        sorted.into_iter().take(days).collect()
    }

    /// Get the last N days of token usage (most recent first)
    pub fn recent_tokens(&self, days: usize) -> Vec<&DailyModelTokens> {
        let mut sorted: Vec<_> = self.daily_model_tokens.iter().collect();
        sorted.sort_by(|a, b| b.date.cmp(&a.date));
        sorted.into_iter().take(days).collect()
    }

    /// Aggregate tokens by model across all time (or specified days)
    pub fn aggregate_tokens_by_model(&self, days: Option<usize>) -> HashMap<String, u64> {
        let tokens_iter: Box<dyn Iterator<Item = &DailyModelTokens>> = match days {
            Some(n) => Box::new(self.recent_tokens(n).into_iter()),
            None => Box::new(self.daily_model_tokens.iter()),
        };

        let mut result: HashMap<String, u64> = HashMap::new();
        for daily in tokens_iter {
            for (model, count) in &daily.tokens_by_model {
                *result.entry(model.clone()).or_insert(0) += count;
            }
        }
        result
    }

    /// Calculate total tokens across all models for specified days
    pub fn total_tokens(&self, days: Option<usize>) -> u64 {
        self.aggregate_tokens_by_model(days).values().sum()
    }
}

/// Get the global Claude stats cache path (~/.claude/stats-cache.json)
pub fn get_global_stats_path() -> Result<PathBuf, RafctlError> {
    let home = dirs::home_dir().ok_or(RafctlError::NoHomeDir)?;
    Ok(home.join(".claude").join("stats-cache.json"))
}

/// Get the stats cache path for a specific profile
pub fn get_profile_stats_path(profile_name: &str, tool: ToolType) -> Result<PathBuf, RafctlError> {
    let profile_dir = get_profile_dir(profile_name)?;
    let tool_dir = match tool {
        ToolType::Claude => "claude",
        ToolType::Codex => "codex",
    };
    Ok(profile_dir.join(tool_dir).join("stats-cache.json"))
}

/// Load stats cache from a file path.
/// Returns empty StatsCache if file doesn't exist or is malformed (graceful degradation).
pub fn load_stats_cache(path: &PathBuf) -> StatsCache {
    if !path.exists() {
        return StatsCache::default();
    }

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "Warning: Failed to read stats cache at {}: {}",
                path.display(),
                e
            );
            return StatsCache::default();
        }
    };

    match serde_json::from_str::<StatsCache>(&content) {
        Ok(stats) => {
            // Warn if schema version is unexpected
            if let Some(version) = stats.version {
                if version != EXPECTED_SCHEMA_VERSION {
                    eprintln!(
                        "Warning: stats-cache.json has version {}, expected {}. Parsing anyway.",
                        version, EXPECTED_SCHEMA_VERSION
                    );
                }
            }
            stats
        }
        Err(e) => {
            eprintln!(
                "Warning: Failed to parse stats cache at {}: {}",
                path.display(),
                e
            );
            StatsCache::default()
        }
    }
}

/// Load stats cache for a profile, falling back to global if not found
pub fn load_profile_stats(profile_name: &str, tool: ToolType) -> StatsCache {
    // Try profile-specific first
    if let Ok(profile_path) = get_profile_stats_path(profile_name, tool) {
        if profile_path.exists() {
            return load_stats_cache(&profile_path);
        }
    }

    // Fall back to global
    if let Ok(global_path) = get_global_stats_path() {
        return load_stats_cache(&global_path);
    }

    StatsCache::default()
}

/// Load global stats cache (~/.claude/stats-cache.json)
pub fn load_global_stats() -> StatsCache {
    match get_global_stats_path() {
        Ok(path) => load_stats_cache(&path),
        Err(_) => StatsCache::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_STATS_JSON: &str = r#"{
        "version": 1,
        "lastComputedDate": "2026-01-06",
        "dailyActivity": [
            {"date": "2026-01-06", "messageCount": 245, "sessionCount": 12, "toolCallCount": 1234},
            {"date": "2026-01-05", "messageCount": 189, "sessionCount": 8, "toolCallCount": 892}
        ],
        "dailyModelTokens": [
            {"date": "2026-01-06", "tokensByModel": {"claude-sonnet-4-5": 450000, "claude-opus-4-5": 50000}},
            {"date": "2026-01-05", "tokensByModel": {"claude-sonnet-4-5": 320000}}
        ],
        "totalSessions": 556,
        "totalMessages": 137728,
        "modelUsage": {
            "claude-sonnet-4-5": {"inputTokens": 2508205, "outputTokens": 15554917, "costUsd": 0}
        }
    }"#;

    #[test]
    fn test_parse_stats_cache() {
        let stats: StatsCache = serde_json::from_str(SAMPLE_STATS_JSON).unwrap();

        assert_eq!(stats.version, Some(1));
        assert_eq!(stats.last_computed_date, Some("2026-01-06".to_string()));
        assert_eq!(stats.daily_activity.len(), 2);
        assert_eq!(stats.daily_model_tokens.len(), 2);
        assert_eq!(stats.total_sessions, Some(556));
        assert_eq!(stats.total_messages, Some(137728));
    }

    #[test]
    fn test_daily_activity() {
        let stats: StatsCache = serde_json::from_str(SAMPLE_STATS_JSON).unwrap();
        let activity = stats.activity_for_date("2026-01-06").unwrap();

        assert_eq!(activity.message_count, 245);
        assert_eq!(activity.session_count, 12);
        assert_eq!(activity.tool_call_count, 1234);
    }

    #[test]
    fn test_tokens_for_date() {
        let stats: StatsCache = serde_json::from_str(SAMPLE_STATS_JSON).unwrap();

        assert_eq!(stats.tokens_for_date("2026-01-06"), 500000);
        assert_eq!(stats.tokens_for_date("2026-01-05"), 320000);
        assert_eq!(stats.tokens_for_date("2026-01-04"), 0);
    }

    #[test]
    fn test_recent_activity() {
        let stats: StatsCache = serde_json::from_str(SAMPLE_STATS_JSON).unwrap();
        let recent = stats.recent_activity(1);

        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].date, "2026-01-06");
    }

    #[test]
    fn test_aggregate_tokens_by_model() {
        let stats: StatsCache = serde_json::from_str(SAMPLE_STATS_JSON).unwrap();
        let aggregated = stats.aggregate_tokens_by_model(None);

        assert_eq!(aggregated.get("claude-sonnet-4-5"), Some(&770000));
        assert_eq!(aggregated.get("claude-opus-4-5"), Some(&50000));
    }

    #[test]
    fn test_total_tokens() {
        let stats: StatsCache = serde_json::from_str(SAMPLE_STATS_JSON).unwrap();

        assert_eq!(stats.total_tokens(None), 820000);
        assert_eq!(stats.total_tokens(Some(1)), 500000);
    }

    #[test]
    fn test_is_empty() {
        let empty = StatsCache::default();
        assert!(empty.is_empty());

        let stats: StatsCache = serde_json::from_str(SAMPLE_STATS_JSON).unwrap();
        assert!(!stats.is_empty());
    }

    #[test]
    fn test_parse_empty_json() {
        let empty: StatsCache = serde_json::from_str("{}").unwrap();
        assert!(empty.is_empty());
        assert!(empty.version.is_none());
    }

    #[test]
    fn test_parse_partial_json() {
        let partial = r#"{"dailyActivity": [{"date": "2026-01-06", "messageCount": 100}]}"#;
        let stats: StatsCache = serde_json::from_str(partial).unwrap();

        assert_eq!(stats.daily_activity.len(), 1);
        assert_eq!(stats.daily_activity[0].message_count, 100);
        assert_eq!(stats.daily_activity[0].session_count, 0); // default
    }

    #[test]
    fn test_model_usage() {
        let stats: StatsCache = serde_json::from_str(SAMPLE_STATS_JSON).unwrap();

        let usage = stats.model_usage.get("claude-sonnet-4-5").unwrap();
        assert_eq!(usage.input_tokens, 2508205);
        assert_eq!(usage.output_tokens, 15554917);
    }
}
