# HUD Statusline Plugin

rafctl includes a native Rust HUD (Heads-Up Display) plugin for Claude Code that shows real-time session information in your terminal statusline.

## Features

- **Profile Display**: Shows current rafctl profile name
- **Context Usage**: Visual progress bar showing token usage
- **Model Info**: Current Claude model
- **Git Branch**: Current git branch
- **Tool Stats**: Number of tool calls and errors
- **Config Count**: Number of active configurations

## Installation

### Build the HUD Binary

```bash
# Build release binaries
cargo build --release

# Install to your system
cargo install --path .
```

### Enable HUD for Claude Code

```bash
# Install globally
rafctl hud install

# Or install for a specific profile
rafctl hud install work

# Check installation status
rafctl hud status
```

### Uninstall HUD

```bash
# Remove from global Claude Code
rafctl hud uninstall

# Remove from specific profile
rafctl hud uninstall work
```

## Sample Output

When active, the HUD displays a statusline like:

```
[work] | 📁 my-project | [sonnet-4-5] | ████████░░ 78% | git:(main) | ⚙️3 | 🔧45
```

### Components

| Component | Description |
|-----------|-------------|
| `[work]` | Active rafctl profile (from `RAFCTL_PROFILE` env) |
| `📁 my-project` | Current working directory |
| `[sonnet-4-5]` | Active Claude model |
| `████████░░ 78%` | Context window usage (color-coded) |
| `git:(main)` | Current git branch |
| `⚙️3` | Number of active configs (CLAUDE.md, rules, MCPs) |
| `🔧45` | Tool call count (with errors in red if any) |

### Context Usage Colors

| Color | Threshold | Meaning |
|-------|-----------|---------|
| Green | 0-69% | Healthy usage |
| Yellow | 70-84% | Approaching limit |
| Red | 85-100% | Near autocompact threshold |

## Environment Variables

The HUD reads these environment variables (set automatically by `rafctl run`):

| Variable | Description |
|----------|-------------|
| `RAFCTL_PROFILE` | Current profile name |
| `RAFCTL_PROFILE_TOOL` | Tool type (claude/codex) |
| `RAFCTL_VERSION` | rafctl version |

## How It Works

The HUD binary (`rafctl-hud`) implements Claude Code's statusLine protocol:

1. Claude Code pipes JSON to stdin with session info
2. HUD parses the payload (context window, model, transcript path)
3. HUD reads transcript file for tool/agent stats
4. HUD counts active configurations
5. HUD gets git branch
6. HUD outputs formatted statusline
7. Process exits (stateless, one-shot execution)

## Manual Installation

If you prefer manual setup, add this to your Claude Code settings:

```json
{
  "statusLine": {
    "command": "/path/to/rafctl-hud"
  }
}
```

Settings file locations:
- Global: `~/.claude/settings.json`
- Per-profile: `~/.rafctl/profiles/<name>/claude/settings.json`

## Performance

The HUD is designed for minimal latency:

- Single static binary (no Node.js runtime)
- ~10ms typical execution time
- Stateless design (no memory accumulation)
- Efficient JSON parsing

## Troubleshooting

### HUD Not Showing

1. Check installation: `rafctl hud status`
2. Restart Claude Code after installation
3. Verify binary path in settings.json

### Wrong Profile Showing

Ensure you're using `rafctl run <profile>` to start Claude Code, which sets the `RAFCTL_PROFILE` environment variable.

### No Git Branch

Git must be installed and the directory must be a git repository.
