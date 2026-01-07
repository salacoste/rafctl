//! Stdin JSON parser for Claude Code HUD protocol.

use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct StdinPayload {
    pub transcript_path: Option<PathBuf>,
    pub cwd: Option<PathBuf>,
    pub model: Option<ModelInfo>,
    pub context_window: Option<ContextWindow>,
}

#[derive(Debug, Deserialize)]
pub struct ModelInfo {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct ContextWindow {
    pub context_window_size: u64,
    pub current_usage: Option<TokenUsage>,
}

#[derive(Debug, Deserialize, Default)]
pub struct TokenUsage {
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub cache_creation_input_tokens: u64,
    #[serde(default)]
    pub cache_read_input_tokens: u64,
}

pub fn parse_stdin(input: &str) -> Result<StdinPayload, serde_json::Error> {
    serde_json::from_str(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_payload() {
        let json = r#"{}"#;
        let payload = parse_stdin(json).unwrap();
        assert!(payload.transcript_path.is_none());
        assert!(payload.cwd.is_none());
        assert!(payload.model.is_none());
        assert!(payload.context_window.is_none());
    }

    #[test]
    fn test_parse_full_payload() {
        let json = r#"{
            "transcript_path": "/tmp/session.jsonl",
            "cwd": "/home/user/project",
            "model": {"name": "claude-sonnet-4-5-20250929"},
            "context_window": {
                "context_window_size": 200000,
                "current_usage": {
                    "input_tokens": 50000,
                    "cache_creation_input_tokens": 10000,
                    "cache_read_input_tokens": 5000
                }
            }
        }"#;

        let payload = parse_stdin(json).unwrap();
        assert_eq!(
            payload.transcript_path,
            Some(PathBuf::from("/tmp/session.jsonl"))
        );
        assert_eq!(payload.cwd, Some(PathBuf::from("/home/user/project")));
        assert_eq!(
            payload.model.as_ref().unwrap().name,
            "claude-sonnet-4-5-20250929"
        );

        let ctx = payload.context_window.as_ref().unwrap();
        assert_eq!(ctx.context_window_size, 200000);

        let usage = ctx.current_usage.as_ref().unwrap();
        assert_eq!(usage.input_tokens, 50000);
        assert_eq!(usage.cache_creation_input_tokens, 10000);
        assert_eq!(usage.cache_read_input_tokens, 5000);
    }

    #[test]
    fn test_parse_partial_usage() {
        let json = r#"{
            "context_window": {
                "context_window_size": 100000,
                "current_usage": {
                    "input_tokens": 25000
                }
            }
        }"#;

        let payload = parse_stdin(json).unwrap();
        let usage = payload
            .context_window
            .as_ref()
            .unwrap()
            .current_usage
            .as_ref()
            .unwrap();

        assert_eq!(usage.input_tokens, 25000);
        assert_eq!(usage.cache_creation_input_tokens, 0);
        assert_eq!(usage.cache_read_input_tokens, 0);
    }
}
