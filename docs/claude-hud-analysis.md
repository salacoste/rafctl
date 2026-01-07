# Claude HUD Analysis for rafctl Integration

> Reference: `_reference/claude-hud/` - cloned from https://github.com/jarrodwatts/claude-hud
> Last updated: 2026-01-07 (comprehensive Oracle analysis)

## Executive Summary

Claude HUD is a statusline plugin for Claude Code that displays real-time session information. This document analyzes its features and implementation for **knowledge extraction** — understanding how to work with Claude Code's analytics data for integration into rafctl.

**Key Insight:** We are NOT adopting claude-hud directly. We are learning from its data access patterns to implement our own `rafctl analytics` and future HUD features.

---

## Feature Inventory

| Feature | Description | Data Source | Priority for rafctl |
|---------|-------------|-------------|---------------------|
| **Context Monitor** | Visual progress bar showing context window usage with color thresholds | stdin JSON | High |
| **Tool Activity** | Live tracking of running tools (Read, Write, Grep, etc.) | transcript JSONL | Medium |
| **Agent Tracking** | Sub-agent status (Explore, Librarian, etc.) with elapsed time | transcript JSONL | Medium |
| **Todo Progress** | Current task and completion ratio | transcript JSONL | Low |
| **Session Duration** | Time since session start | transcript JSONL | Low |
| **Model Display** | Active model name (e.g., "Opus 4.5") | stdin JSON | Low |
| **Git Branch** | Current branch name | git command | Low |
| **Config Counts** | Number of CLAUDE.md, rules, MCPs, hooks loaded | file system scan | Low |

---

## Data Architecture

### Data Flow
```
Claude Code → stdin JSON → claude-hud → stdout → terminal display
           ↘ transcript JSONL file
```

### Stdin Payload (from Claude Code)
```typescript
interface StdinData {
  transcript_path?: string;
  cwd?: string;
  model?: {
    id?: string;
    display_name?: string;
  };
  context_window?: {
    context_window_size?: number;      // total capacity (e.g., 200,000)
    current_usage?: {
      input_tokens?: number;
      cache_creation_input_tokens?: number;
      cache_read_input_tokens?: number;
    };
  };
}
```

### Transcript JSONL Format
Each line contains a JSON object with:
- `timestamp`: ISO 8601 datetime
- `message.content[]`: Array of content blocks
  - `tool_use`: Tool invocation (id, name, input)
  - `tool_result`: Tool completion (tool_use_id, is_error)

### Key Implementation Details

1. **AUTOCOMPACT_BUFFER = 45,000 tokens**
   - Claude Code reserves ~45k tokens for system prompts
   - Must add this to user usage to match `/context` output

2. **Streaming JSONL parsing**
   - Uses readline interface for memory efficiency
   - Skips malformed lines gracefully
   - Keeps only last 20 tools, 10 agents in memory

3. **Color thresholds**
   - Green: < 70%
   - Yellow: 70-85%
   - Red: > 85%

---

## What Statistics ARE Available

| Metric | How to Get |
|--------|-----------|
| Context window usage (%) | stdin: `current_usage / context_window_size` |
| Input tokens | stdin: `context_window.current_usage.input_tokens` |
| Cache tokens | stdin: `cache_creation_input_tokens + cache_read_input_tokens` |
| Session duration | transcript: first timestamp to now |
| Tool usage | transcript: parse `tool_use` / `tool_result` blocks |
| Active model | stdin: `model.display_name` |

## What Statistics are NOT Available

| Metric | Why Not |
|--------|---------|
| **Account quota limits** | Not in stdin/transcript - requires API call |
| **$/day spending** | Not exposed locally |
| **Rate limits (RPM/TPM)** | Not exposed |
| **Historical usage** | Not in this data stream |
| **Output tokens** | Only input/context tokens reported |

---

## Claude Code Local Files

| File | Purpose | Quota Relevance |
|------|---------|-----------------|
| `~/.claude/settings.oauth.token` | OAuth access token | Required for quota API |
| `~/.claude/stats-cache.json` | Historical usage stats | **HIGH** - daily tokens, model breakdown |
| `~/.claude/transcripts/*.jsonl` | Session logs | Medium - raw usage data |
| `~/.claude/history.jsonl` | Command history | Low |

### stats-cache.json Structure
```json
{
  "dailyActivity": [
    { "date": "2025-12-20", "messageCount": 3359, "toolCallCount": 950, "sessionCount": 8 }
  ],
  "dailyModelTokens": [
    { "date": "2025-12-20", "tokensByModel": { "claude-sonnet-4-5": 150000 } }
  ],
  "modelUsage": {
    "claude-sonnet-4-5": { "inputTokens": 2508205, "outputTokens": 15554917, "costUSD": 0 }
  }
}
```

---

## Integration Opportunities for rafctl

### 1. Auto-Provision claude-hud (High Impact)
When creating a new profile, offer to install and configure claude-hud automatically.
- Pre-seed `settings.json` with `statusLine.command`
- No manual user steps required

### 2. Profile-Aware HUD
Inject `RAFCTL_PROFILE_NAME` env var when spawning Claude Code.
- Fork/contribute to claude-hud to display profile name
- Visual distinction between work/personal profiles

### 3. Usage Analytics Command
New command: `rafctl analytics` or `rafctl usage --history`
- Read `stats-cache.json` from each profile
- Show daily trends, model breakdown
- Aggregate across all profiles

### 4. Cost Calculator
Multiply token counts from `stats-cache.json` by Anthropic pricing.
- Show "estimated spend" per profile
- Currently missing from Claude Code CLI

### 5. Cross-Profile Statistics
```bash
rafctl stats
# Total tokens today: 450,000 (work: 300k, personal: 150k)
# Most active: work (8 sessions)
# Models: Sonnet 4.5 (80%), Opus (20%)
```

---

## Plugin Registration

### settings.json Configuration
```json
{
  "statusLine": {
    "type": "command",
    "command": "bash -c '\"$(command -v node)\" \"$(ls -td ~/.claude/plugins/cache/claude-hud/claude-hud/*/ | head -1)dist/index.js\"'"
  }
}
```

### Plugin Files
- `.claude-plugin/plugin.json` - Plugin metadata
- `.claude-plugin/marketplace.json` - Marketplace listing
- `commands/setup.md` - Setup instructions (executed by Claude)

---

## Recommendations

### Short Term (v0.2.x)
1. ✅ Document this analysis
2. Add `rafctl analytics` command to read `stats-cache.json`
3. Show cost estimates based on token usage

### Medium Term (v0.3.x)
1. Create rafctl statusline plugin for profile-aware HUD
2. Transcript parsing for session-level analytics
3. Profile switching indicator in HUD

### Long Term
1. Unified dashboard with real-time context monitoring
2. Cross-profile quota aggregation
3. Usage alerts/notifications
