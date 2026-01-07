# Sessions & Analytics

rafctl provides comprehensive session monitoring and usage analytics for Claude Code.

## Usage Analytics

View historical usage statistics from Claude Code's local cache:

```bash
# Show usage for default/last-used profile (7 days)
rafctl analytics

# Show usage for specific profile
rafctl analytics work

# Show all profiles comparison
rafctl analytics --all

# Custom time range
rafctl analytics --days 30

# Include cost estimates
rafctl analytics --cost

# JSON output for scripting
rafctl analytics --json
```

### Sample Output

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

### Cost Estimation

With `--cost` flag, rafctl estimates API costs based on token usage:

```
💰 Estimated Costs (last 7 days)

Model               Input Tokens    Est. Cost
──────────────────────────────────────────────
claude-sonnet-4-5   2.5M            ~$30.00
claude-opus-4-5     625K            ~$37.50
                              ─────────────
                    Estimated Total: ~$67.50

* Output tokens estimated at 3:1 ratio (not tracked locally)
```

## Session History

View past Claude Code sessions:

```bash
# List recent 10 sessions
rafctl sessions

# Show only today's sessions
rafctl sessions --today

# Show more sessions
rafctl sessions --limit 20

# JSON output
rafctl sessions --json

# View specific session details
rafctl sessions <session-id>
```

### Session List

```
📋 Recent Sessions (392 total)

┌─────────────────┬──────────────────┬──────────┬──────────┬───────┬────────┐
│ Session ID      ┆ Started          ┆ Duration ┆ Messages ┆ Tools ┆ Errors │
╞═════════════════╪══════════════════╪══════════╪══════════╪═══════╪════════╡
│ efb00c6e-839... ┆ 2026-01-06 21:12 ┆ 14h 24m  ┆ 781      ┆ 226   ┆ 12     │
│ 1d542c9c-b5a... ┆ 2026-01-06 13:36 ┆ 4s       ┆ 2        ┆ 0     ┆ 0      │
│ 74d5ba35-412... ┆ 2026-01-06 13:32 ┆ 7h 40m   ┆ 1536     ┆ 444   ┆ 28     │
└─────────────────┴──────────────────┴──────────┴──────────┴───────┴────────┘
```

### Session Details

Use partial session ID to view details:

```bash
rafctl sessions efb00c6e
```

```
📋 Session Details — efb00c6e-839...

Started:     2026-01-06 21:12:38
Ended:       2026-01-07 11:36:50
Duration:    14h 24m
Directory:   /Users/user/project
Git Branch:  main
Model:       claude-sonnet-4-5-20250929

Messages:    781
Tool Calls:  226 (12 errors)
Agent Calls: 0

Tool Breakdown:
  ████░░░░░░ Edit           91 calls (40%)
  ██░░░░░░░░ Read           46 calls (20%)
  ██░░░░░░░░ Bash           43 calls (19%)
  █░░░░░░░░░ TodoWrite      29 calls (13%)
  ░░░░░░░░░░ Write          11 calls (5%)
```

## Live Session Monitor

Watch Claude Code activity in real-time:

```bash
# Watch most recent session
rafctl watch

# Watch with profile context
rafctl watch work
```

### Live Output

```
🔴 LIVE Session Monitor — Profile: work — Session: efb00c6e...
────────────────────────────────────────────────────────────────
Press Ctrl+C to stop watching

[14:32:15] 📖 Read → main.rs
[14:32:17] 🔎 Grep → "fn main"
[14:32:20] ✏️  Edit → main.rs
[14:32:25] 🚀 Bash → cargo build --release
[14:32:45] ✗ Tool error
[14:32:50] 🤖 Task → "Find auth module"
[14:33:05] 📋 TodoWrite → updating todos
```

### Tool Icons

| Icon | Tool |
|------|------|
| 📖 | Read |
| 📝 | Write |
| ✏️ | Edit |
| 🚀 | Bash |
| 🔍 | Glob |
| 🔎 | Grep |
| 🤖 | Task (Agent) |
| 📋 | TodoWrite/Read |
| 🔧 | Other tools |

## Data Sources

rafctl reads from Claude Code's local data:

| Source | Location | Data |
|--------|----------|------|
| Stats Cache | `~/.claude/stats-cache.json` | Daily activity, tokens by model |
| Transcripts | `~/.claude/projects/<project>/*.jsonl` | Session details, tool calls |

For per-profile data, rafctl reads from profile directories:
- `~/.rafctl/profiles/<name>/claude/stats-cache.json`

## Scripting Examples

```bash
# Get today's message count
rafctl analytics --json | jq '.summary.messages'

# List sessions with errors
rafctl sessions --json | jq '.sessions[] | select(.errors > 0)'

# Get most active session
rafctl sessions --json | jq '.sessions | max_by(.tool_calls)'
```
