# Claude HUD Deep Analysis (v2)

> Second-pass analysis for implementation planning

## 1. Hidden Features & Architecture

### Zero-Dependency Design
The project has **no runtime dependencies**. It relies entirely on Node.js native modules (`node:fs`, `node:child_process`, `node:readline`). This makes it extremely portable and fast to install.

### Ephemeral "One-Shot" Execution
The tool is **stateless**. It does not run as a daemon. Every time the status line updates (~300ms), the process:
1. Starts
2. Reads stdin
3. Parses entire transcript file
4. Prints output
5. Exits

### AUTOCOMPACT_BUFFER = 45,000 tokens
This accounts for a reserved token buffer that Claude Code internally uses, ensuring the percentage calculation matches Claude's native `/context` command (approx. 22.5% of a 200k window).

### Comprehensive Config Discovery
The configuration counting logic scans:
- **User Scope**: `~/.claude/CLAUDE.md`, `~/.claude/rules/`, `~/.claude/settings.json`, `~/.claude.json`
- **Project Scope**: `cwd/CLAUDE.md`, `cwd/CLAUDE.local.md`, `cwd/.claude/CLAUDE.md`, `cwd/.mcp.json`, `cwd/.claude/settings.local.json`
- Recursively counts rules in nested directories

---

## 2. Critical Performance Consideration

### The "Re-read World" Problem
Because the tool is stateless, it uses `fs.createReadStream` to parse the **entire** transcript JSONL file from the beginning on *every single execution*.

**Impact**: In a long-running session where the transcript grows to tens of megabytes, this will cause increasing CPU usage and latency.

**Mitigation in code**: Uses `readline` with `crlfDelay: Infinity` (efficient), but algorithmic complexity is still O(N) where N is session length. There is no file tailing or offset tracking.

---

## 3. Edge Cases & Reliability

| Scenario | Behavior |
|----------|----------|
| Git timeout | 1000ms timeout, silently omits branch if slow |
| Missing stdin | Returns "Initializing..." if TTY |
| Malformed JSONL line | Skips line, doesn't crash |
| Small context window | Guards against `size <= AUTOCOMPACT_BUFFER`, returns 0% |

---

## 4. Data Parsing Details

### Agent Detection
- Tool named `Task` → treated as Agent
- Extracts `subagent_type` from input
- Missing field → reports "unknown"

### Todo State Management
`TodoWrite` tool **replaces** entire `latestTodos` list (not cumulative)

### Target Extraction Rules
| Tool | Target Source |
|------|---------------|
| Bash | `input.command` (truncated to 30 chars) |
| Read, Write, Edit | `input.file_path` OR `input.path` |
| Glob, Grep | `input.pattern` |

---

## 5. Rust Port Specification

### Stdin Schema (Rust structs)
```rust
#[derive(Deserialize)]
struct StdinPayload {
    transcript_path: Option<PathBuf>,
    cwd: Option<PathBuf>,
    model: Option<ModelInfo>,
    context_window: Option<ContextWindow>,
}

#[derive(Deserialize)]
struct ContextWindow {
    context_window_size: u64,
    current_usage: Option<TokenUsage>,
}

#[derive(Deserialize)]
struct TokenUsage {
    #[serde(default)]
    input_tokens: u64,
    #[serde(default)]
    cache_creation_input_tokens: u64,
    #[serde(default)]
    cache_read_input_tokens: u64,
}
```

### Context Percentage Formula
```rust
const AUTOCOMPACT_BUFFER: u64 = 45_000;

fn calculate_percent(size: u64, usage: &TokenUsage) -> u8 {
    if size <= AUTOCOMPACT_BUFFER { return 0; }
    
    let total_tokens = usage.input_tokens 
        + usage.cache_creation_input_tokens 
        + usage.cache_read_input_tokens;
        
    let percent = ((total_tokens + AUTOCOMPACT_BUFFER) as f64 / size as f64) * 100.0;
    percent.round().min(100.0) as u8
}
```

### Color Thresholds
- Green: < 70%
- Yellow: 70-84%
- Red: >= 85%

### Recommended Rust Crates
| Purpose | Crate |
|---------|-------|
| JSON parsing | `serde`, `serde_json` |
| Filesystem walk | `walkdir` |
| Home directory | `dirs` |
| Colors | `colored` |
| Errors | `anyhow` |
| Time | `chrono` |

---

## 6. Claude Code Internal Files

### Primary Files
| File | Purpose |
|------|---------|
| `~/.claude.json` | Global app state, OAuth account, projects |
| `~/.claude/settings.json` | User preferences, hooks, permissions |
| `~/.claude/settings.oauth.token` | **Sensitive** Bearer token |
| `~/.claude/stats-cache.json` | Usage statistics (daily tokens, costs) |
| `~/.claude/transcripts/*.jsonl` | Full conversation logs |

### stats-cache.json Schema
```json
{
  "version": 1,
  "lastComputedDate": "YYYY-MM-DD",
  "dailyActivity": [
    { "date": "YYYY-MM-DD", "messageCount": 123, "sessionCount": 5, "toolCallCount": 45 }
  ],
  "dailyModelTokens": [
    { "date": "YYYY-MM-DD", "tokensByModel": { "claude-sonnet-4-5": 150000 } }
  ],
  "totalSessions": 556,
  "totalMessages": 137728
}
```

---

## 7. Integration Opportunities for rafctl

### Profile Isolation Strategy
1. **Swap Target 1**: `~/.claude.json` (for `oauthAccount`)
2. **Swap Target 2**: `~/.claude/settings.oauth.token` (for auth)
3. **Swap Target 3**: `~/.claude/stats-cache.json` (optional, per-profile stats)

### New Commands to Implement

#### `rafctl analytics`
Read `stats-cache.json` to show:
- Daily token usage
- Model breakdown
- Session counts
- Estimated costs

#### `rafctl hud` (future)
Rust-native statusline plugin:
- Profile-aware (show current profile name)
- Cross-platform
- Zero external dependencies

### Priority Matrix

| Feature | Effort | Value | Priority |
|---------|--------|-------|----------|
| `rafctl analytics` | M | High | v0.3.0 |
| Stats aggregation across profiles | M | High | v0.3.0 |
| Transcript watching | H | Medium | v0.4.0 |
| Native HUD plugin | H | Medium | v0.4.0 |
| Cost calculator | L | Medium | v0.3.0 |
