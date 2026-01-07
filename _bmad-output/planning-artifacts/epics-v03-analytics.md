# rafctl v0.3.0+ Epics — Usage Analytics & HUD Integration

> Based on comprehensive Oracle analysis of claude-hud (2026-01-07)
> Reference: `docs/claude-hud-analysis.md`, `docs/claude-hud-deep-analysis.md`
> Source: `_reference/claude-hud/` (TypeScript statusline plugin)

---

## Data Sources Summary

Before implementation, understand what data is available:

| Source | Data Available | Location |
|--------|----------------|----------|
| **stats-cache.json** | Daily activity, tokens by model, session counts | `~/.claude/stats-cache.json` |
| **Transcript JSONL** | Tool calls, agents, todos, timestamps | `~/.claude/transcripts/*.jsonl` |
| **Stdin JSON** | Context window %, model, cwd (real-time) | Piped from Claude Code to statusLine |
| **Quota API** | 5h/7d usage windows (already in rafctl) | `api.anthropic.com/api/oauth/usage` |

### Critical Constants

```rust
/// Reserved token buffer that Claude Code uses internally.
/// Must be added to user tokens to match /context output.
/// Source: claude-hud src/constants.ts
const AUTOCOMPACT_BUFFER: u64 = 45_000;
```

### Data NOT Available Locally

| Metric | Why Not | Workaround |
|--------|---------|------------|
| Output tokens | Not in stats-cache.json | Estimate from input tokens (3:1 ratio typical) |
| API costs | Not tracked locally | Calculate from token counts + pricing |
| Rate limits (RPM/TPM) | Not exposed | Use quota API (already have) |
| Real-time context % | Only via stdin to HUD | Story 15.1 (Native HUD) |

---

## Epic 10: Usage Analytics (v0.2.0) ✅ DONE

**Goal:** Quota monitoring via API

**Status:** Completed in v0.2.0
- `rafctl quota` command
- 5-hour and 7-day usage windows
- Visual progress bars

---

## Epic 11: TUI Dashboard (v0.2.0) ✅ DONE

**Goal:** Interactive profile management

**Status:** Completed in v0.2.0
- `rafctl dashboard` command
- Profile table with navigation
- Run/Login actions

---

## Epic 12: Local Usage Analytics (v0.3.0) 🆕

**Goal:** Read and display historical usage data from Claude Code's local `stats-cache.json` file without API calls.

**Value:** 
- Instant access (no network required)
- Daily trends and patterns
- Model usage breakdown
- Cost estimation (with caveats)
- Cross-profile comparison

### Story 12.1: Stats Cache Parser

As a **developer**,
I want **to parse Claude Code's stats-cache.json file**,
So that **I can display local usage statistics**.

**Acceptance Criteria:**

**Given** `~/.claude/stats-cache.json` exists
**When** I parse it
**Then** I get structured data with:
- `version` — schema version (currently 1)
- `lastComputedDate` — when stats were last updated
- `dailyActivity[]` — messageCount, sessionCount, toolCallCount per day
- `dailyModelTokens[]` — tokensByModel per day
- `totalSessions`, `totalMessages`

**Given** schema version is not 1
**When** I parse it
**Then** I log warning and attempt best-effort parsing

**Given** file is missing or malformed
**When** I try to parse
**Then** I return empty result, not error

**Technical Notes:**
```rust
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]  // JSON uses camelCase
pub struct StatsCache {
    pub version: Option<u32>,
    pub last_computed_date: Option<String>,
    pub daily_activity: Option<Vec<DailyActivity>>,
    pub daily_model_tokens: Option<Vec<DailyModelTokens>>,
    pub total_sessions: Option<u64>,
    pub total_messages: Option<u64>,
    // Note: modelUsage also exists but duplicates dailyModelTokens
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyActivity {
    pub date: String,  // "YYYY-MM-DD"
    pub message_count: u64,
    pub session_count: u64,
    pub tool_call_count: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyModelTokens {
    pub date: String,
    pub tokens_by_model: HashMap<String, u64>,  // e.g. {"claude-sonnet-4-5": 150000}
}
```

**File Location:**
- Global: `~/.claude/stats-cache.json`
- Per-profile: `~/.rafctl/profiles/<name>/claude/stats-cache.json`

**Effort:** S (2-3h)

---

### Story 12.2: Analytics Command

As a **user**,
I want **to see my historical usage statistics**,
So that **I can track my Claude Code usage over time**.

**Acceptance Criteria:**

**Given** I run `rafctl analytics`
**When** stats-cache.json exists for default/last-used profile
**Then** I see:
```
📊 Usage Analytics — Profile: work (last 7 days)

Date         Messages  Sessions  Tools    Tokens
─────────────────────────────────────────────────
2026-01-06        245        12    1,234   450,000
2026-01-05        189         8      892   320,000
2026-01-04        156         6      567   280,000
...

Totals: 1,234 messages · 45 sessions · 5,432 tool calls

Models Used:
  ████████░░ claude-sonnet-4-5   2.5M tokens (80%)
  ██░░░░░░░░ claude-opus-4-5     625K tokens (20%)
```

**Given** no profile specified and no default set
**When** I run `rafctl analytics`
**Then** I show global ~/.claude stats (fallback)

**Given** stats-cache.json doesn't exist
**When** I run `rafctl analytics`
**Then** I see: "No usage data found. Run Claude Code to generate statistics."

**Flags:**
- `--json` — JSON output for scripting
- `--plain` — No colors/emoji
- `--days N` — Change window (default: 7)

**Effort:** M (4-6h)

---

### Story 12.3: Per-Profile Analytics

As a **user**,
I want **to see analytics for a specific profile**,
So that **I can compare usage across accounts**.

**Acceptance Criteria:**

**Given** I run `rafctl analytics work`
**When** profile "work" exists
**Then** I see analytics from `~/.rafctl/profiles/work/claude/stats-cache.json`

**Given** I run `rafctl analytics --all`
**Then** I see aggregated summary:
```
📊 Cross-Profile Analytics (last 7 days)

Profile      Messages  Sessions  Tokens     Last Active
────────────────────────────────────────────────────────
work              845        32    2.1M     2026-01-06
personal          234        12    560K     2026-01-05
client-a           89         4    180K     2026-01-03
────────────────────────────────────────────────────────
Total           1,168        48    2.8M
```

**Given** I run `rafctl analytics work personal --compare`
**Then** I see side-by-side comparison

**Effort:** M (3-4h)

---

### Story 12.4: Cost Estimator

As a **user**,
I want **to see estimated costs based on my token usage**,
So that **I can budget my AI usage**.

**Acceptance Criteria:**

**Given** I run `rafctl analytics --cost`
**Then** I see estimated costs:
```
💰 Estimated Costs (last 7 days) — Profile: work

Model               Input Tokens    Est. Input     Est. Output*   Total
──────────────────────────────────────────────────────────────────────
claude-sonnet-4-5   2.5M            $7.50          ~$22.50        ~$30.00
claude-opus-4-5     625K            $9.38          ~$28.13        ~$37.50
                                                          ─────────────
                                              Estimated Total: ~$67.50

* Output tokens estimated at 3:1 ratio (not tracked locally)
```

**Pricing (hardcoded, updateable):**
```rust
struct ModelPricing {
    input_per_million: f64,
    output_per_million: f64,
}

const PRICING: &[(&str, ModelPricing)] = &[
    ("claude-sonnet-4-5", ModelPricing { input: 3.0, output: 15.0 }),
    ("claude-opus-4-5", ModelPricing { input: 15.0, output: 75.0 }),
    ("claude-haiku-3-5", ModelPricing { input: 0.25, output: 1.25 }),
];
```

**Caveat:** Output tokens not available locally. Use 3:1 estimate with clear warning.

**Effort:** S (2-3h)

---

### Story 12.5: Analytics in Dashboard (Enhancement)

As a **user**,
I want **to see quick usage stats in the TUI dashboard**,
So that **I get a complete overview without switching commands**.

**Acceptance Criteria:**

**Given** I open `rafctl dashboard`
**When** profiles are displayed
**Then** I see additional columns (if stats available):
```
Name         Tool    Auth      Today      7-Day      Last Used
───────────────────────────────────────────────────────────────
▶ work       claude  ✓ Auth    45 msgs    1.2M tok   2026-01-06 14:30
  personal   claude  ✓ Auth    12 msgs    320K tok   2026-01-05 09:15
```

**Effort:** M (3-4h)
**Depends on:** Story 12.1

---

## Epic 13: Profile-Aware HUD (v0.3.0) 🆕

**Goal:** Inject profile context into Claude Code sessions for visual identification.

### Story 13.1: Profile Environment Injection

As a **user**,
I want **to identify which profile is active in my terminal**,
So that **I don't accidentally use the wrong account**.

**Acceptance Criteria:**

**Given** I run `rafctl run work`
**When** Claude Code starts
**Then** these environment variables are set:
- `RAFCTL_PROFILE=work`
- `RAFCTL_PROFILE_TOOL=claude`
- `RAFCTL_VERSION=0.3.0`

**Technical Notes:**
- Already setting `CLAUDE_CONFIG_DIR` in `handle_run`
- Add 3 more env vars for HUD plugins to read
- Third-party HUD plugins can display `$RAFCTL_PROFILE`

**Effort:** XS (30min)

---

### Story 13.2: Profile Indicator in Terminal Title

As a **user**,
I want **my terminal title to show the active profile**,
So that **I can distinguish tabs at a glance**.

**Acceptance Criteria:**

**Given** I run `rafctl run work`
**When** Claude Code starts
**Then** terminal title is set to: `[rafctl:work] claude`

**Implementation:**
```rust
// Set terminal title via ANSI escape
print!("\x1b]0;[rafctl:{}] {}\x07", profile_name, tool_name);
```

**Effort:** XS (30min)

---

### Story 13.3: Plugin Configuration Helper (Optional)

As a **user**,
I want **rafctl to help configure statusline plugins**,
So that **I get the HUD working without manual JSON editing**.

**Acceptance Criteria:**

**Given** I run `rafctl config hud --enable`
**Then** profile's `settings.json` is updated with statusLine config

**Given** I run `rafctl config hud --disable`
**Then** statusLine config is removed

**Given** claude-hud plugin is not installed
**When** I enable HUD
**Then** I see: "Tip: Install claude-hud first with `/plugin install claude-hud`"

**Effort:** M (3-4h)

---

## Epic 14: Session Monitoring (v0.4.0) ✅ DONE

**Goal:** Parse transcript files for session-level analytics and real-time monitoring.

### Story 14.1: Transcript Parser Module ✅ DONE

As a **developer**,
I want **to parse Claude Code transcript JSONL files**,
So that **I can extract detailed session analytics**.

**Acceptance Criteria:**

**Given** a transcript JSONL file (one JSON object per line)
**When** I parse it
**Then** I extract:
- Session start time (first timestamp)
- Tool usage: name, target, status, duration
- Agent (Task) calls: subagent_type, description, status
- Todo states from TodoWrite calls
- Error count from tool_result.is_error

**Technical Notes (from claude-hud analysis):**
```rust
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct TranscriptEntry {
    pub timestamp: Option<String>,  // ISO 8601
    pub message: Option<TranscriptMessage>,
}

#[derive(Debug, Deserialize)]
pub struct TranscriptMessage {
    pub content: Option<Vec<ContentBlock>>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    ToolUse {
        id: String,
        name: String,
        input: Option<Value>,
    },
    ToolResult {
        tool_use_id: String,
        is_error: Option<bool>,
    },
    #[serde(other)]
    Other,
}

// Agent detection: name == "Task"
// Todo detection: name == "TodoWrite"
```

**Target Extraction Rules (from claude-hud):**
| Tool Name | Target Field |
|-----------|--------------|
| Read, Write, Edit | `input.file_path` OR `input.path` |
| Glob, Grep | `input.pattern` |
| Bash | `input.command` (truncate to 30 chars) |

**Effort:** M (4-6h)

---

### Story 14.2: Session List Command ✅ DONE

As a **user**,
I want **to see a list of my past sessions**,
So that **I can review my work history**.

**Acceptance Criteria:**

**Given** I run `rafctl sessions [profile]`
**Then** I see:
```
📋 Recent Sessions — Profile: work

Session ID        Started              Duration   Messages  Tools  Errors
──────────────────────────────────────────────────────────────────────────
ses_abc123...     2026-01-06 14:30     2h 15m         145    523       2
ses_def456...     2026-01-06 10:00     1h 30m          89    234       0
ses_789xyz...     2026-01-05 16:45     45m             34     89       1
```

**Given** I run `rafctl sessions --today`
**Then** I see only today's sessions

**Effort:** M (4-6h)

---

### Story 14.3: Session Detail Command ✅ DONE

As a **user**,
I want **to see details of a specific session**,
So that **I can understand what happened**.

**Acceptance Criteria:**

**Given** I run `rafctl sessions ses_abc123`
**Then** I see:
```
📋 Session Details — ses_abc123

Started:    2026-01-06 14:30:15
Ended:      2026-01-06 16:45:32
Duration:   2h 15m 17s
Messages:   145
Tools:      523 calls (12 errors)

Tool Breakdown:
  Read         234 calls   ████████░░ 45%
  Bash         156 calls   ██████░░░░ 30%
  Write         89 calls   ███░░░░░░░ 17%
  Other         44 calls   █░░░░░░░░░  8%

Agents Used:
  explore       3 calls    avg 45s
  librarian     2 calls    avg 1m 30s
```

**Effort:** M (3-4h)

---

### Story 14.4: Live Session Monitor ✅ DONE

As a **user**,
I want **to watch session activity in real-time**,
So that **I can see what Claude is doing**.

**Acceptance Criteria:**

**Given** I run `rafctl watch [profile]`
**When** Claude Code is running with that profile
**Then** I see live updates (file tailing):
```
🔴 LIVE — Profile: work — Session: ses_abc123
Context: ████████░░ 78%

[14:32:15] 🔧 Read → src/main.rs
[14:32:17] 🔧 Grep → "fn main"
[14:32:20] ✏️  Edit → src/main.rs
[14:32:25] 🚀 Bash → cargo build (running...)
[14:32:45] ✓ Bash → cargo build (20s, success)
[14:32:50] 🤖 Task:explore → "Find auth module" (running...)
```

**Technical Notes:**
- Use `notify` crate for filesystem watching
- Find most recent `.jsonl` in transcripts dir
- Tail new lines, parse incrementally

**Effort:** L (8-12h)

---

## Epic 15: Native Rust HUD (v0.4.0+) 🆕

**Goal:** Create a Rust-native statusline plugin to replace Node.js claude-hud.

**Rationale:**
- Zero external dependencies (no Node.js required)
- Faster startup (single static binary)
- Profile-aware by default
- Full control over features

### Story 15.1: HUD Core Binary

As a **developer**,
I want **a Rust binary that implements Claude Code's statusLine protocol**,
So that **it works as a drop-in replacement for claude-hud**.

**Acceptance Criteria:**

**Given** Claude Code pipes JSON to stdin
**When** I parse and render
**Then** I output formatted statusline to stdout
**And** exit immediately (stateless, one-shot execution)

**Protocol:**
1. Read all of stdin (JSON)
2. Parse StdinData
3. Parse transcript file (if path provided)
4. Count configs (CLAUDE.md, rules, MCPs, hooks)
5. Get git branch
6. Render formatted output
7. Exit

**Key Constants:**
```rust
/// Context window reserved buffer (from claude-hud analysis)
const AUTOCOMPACT_BUFFER: u64 = 45_000;

/// Color thresholds for context usage
const THRESHOLD_YELLOW: u8 = 70;
const THRESHOLD_RED: u8 = 85;

/// Progress bar characters
const BAR_FILLED: char = '█';
const BAR_EMPTY: char = '░';
const BAR_WIDTH: usize = 10;
```

**Context Calculation:**
```rust
fn calculate_context_percent(window: &ContextWindow) -> u8 {
    let size = window.context_window_size;
    if size <= AUTOCOMPACT_BUFFER { return 0; }
    
    let usage = window.current_usage.as_ref();
    let tokens = usage.map(|u| {
        u.input_tokens.unwrap_or(0)
            + u.cache_creation_input_tokens.unwrap_or(0)
            + u.cache_read_input_tokens.unwrap_or(0)
    }).unwrap_or(0);
    
    let percent = ((tokens + AUTOCOMPACT_BUFFER) as f64 / size as f64) * 100.0;
    percent.round().min(100.0) as u8
}
```

**Performance:** Must complete in < 50ms

**Effort:** L (8-12h)

---

### Story 15.2: Profile-Aware Display

As a **user**,
I want **my HUD to show which rafctl profile is active**,
So that **I always know which account I'm using**.

**Acceptance Criteria:**

**Given** `RAFCTL_PROFILE=work` environment variable is set
**When** HUD renders
**Then** profile name appears first:
```
[work] 📁 my-project | [Opus 4.5] ████░░░░░░ 45% | git:(main) | ⏱️ 15m
```

**Given** `RAFCTL_PROFILE` is not set
**Then** profile section is omitted (behave like standard claude-hud)

**Color coding for profile:**
- Default profile: Cyan
- Non-default: Yellow

**Effort:** S (2-3h)

---

### Story 15.3: HUD Installation Helper

As a **user**,
I want **rafctl to install the HUD plugin automatically**,
So that **I don't need manual configuration**.

**Acceptance Criteria:**

**Given** I run `rafctl hud install`
**Then:**
1. Binary is copied to `~/.rafctl/bin/rafctl-hud`
2. Profile's `settings.json` is updated with statusLine config
3. Success message shown

**Given** I run `rafctl hud uninstall`
**Then** statusLine config is removed

**Effort:** M (3-4h)

---

## Priority Matrix Summary

| Epic | Feature | Effort | Value | Target | Dependencies |
|------|---------|--------|-------|--------|--------------|
| **12.1** | Stats Cache Parser | S | High | v0.3.0 | None |
| **12.2** | Analytics Command | M | High | v0.3.0 | 12.1 |
| **12.3** | Per-Profile Analytics | M | High | v0.3.0 | 12.2 |
| **12.4** | Cost Estimator | S | Medium | v0.3.0 | 12.1 |
| **12.5** | Dashboard Stats | M | Medium | v0.3.0 | 12.1 |
| **13.1** | Profile Env Injection | XS | Medium | v0.3.0 | None |
| **13.2** | Terminal Title | XS | Low | v0.3.0 | None |
| **13.3** | Plugin Config Helper | M | Low | v0.3.x | None |
| **14.1** | Transcript Parser | M | Medium | v0.4.0 | None |
| **14.2** | Session List | M | Medium | v0.4.0 | 14.1 |
| **14.3** | Session Detail | M | Medium | v0.4.0 | 14.1 |
| **14.4** | Live Monitor | L | Low | v0.4.0+ | 14.1 |
| **15.1** | Native HUD Core | L | Medium | v0.4.0+ | 14.1 |
| **15.2** | Profile Display | S | Medium | v0.4.0+ | 15.1 |
| **15.3** | HUD Installer | M | Low | v0.4.0+ | 15.1 |

---

## v0.3.0 Release Scope (Recommended)

**Must Have:**
- Story 12.1: Stats Cache Parser
- Story 12.2: Analytics Command
- Story 12.3: Per-Profile Analytics
- Story 13.1: Profile Env Injection

**Should Have:**
- Story 12.4: Cost Estimator
- Story 13.2: Terminal Title

**Could Have:**
- Story 12.5: Dashboard Stats
- Story 13.3: Plugin Config Helper

**Total Effort:** ~20-25 hours

---

## Dependency Graph

```
v0.3.0 Core
├── Story 12.1 (Stats Parser) ← Foundation
│   ├── Story 12.2 (Analytics Command)
│   │   └── Story 12.3 (Per-Profile)
│   ├── Story 12.4 (Cost Estimator)
│   └── Story 12.5 (Dashboard Stats)
├── Story 13.1 (Env Injection) ← Independent
└── Story 13.2 (Terminal Title) ← Independent

v0.4.0 Sessions
├── Story 14.1 (Transcript Parser) ← Foundation
│   ├── Story 14.2 (Session List)
│   ├── Story 14.3 (Session Detail)
│   └── Story 14.4 (Live Monitor)

v0.4.0+ Native HUD
└── Story 15.1 (HUD Core) ← Can start after 14.1
    ├── Story 15.2 (Profile Display)
    └── Story 15.3 (HUD Installer)
```

---

## Definition of Done

Same as main epics.md:
- [ ] All acceptance criteria met
- [ ] Unit tests written and passing
- [ ] Integration test coverage where applicable
- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Documentation updated
- [ ] No unwrap() or expect() in production code
- [ ] Error messages include suggestions
- [ ] Supports `--json` and `--plain` output (where applicable)
