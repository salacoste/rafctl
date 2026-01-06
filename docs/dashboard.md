# TUI Dashboard

The `rafctl dashboard` command provides an interactive terminal user interface for managing your profiles.

## Usage

```bash
rafctl dashboard
```

This launches a full-screen terminal UI that displays all your profiles in a navigable table.

## Interface

```
┌─ AI Coding Agent Profile Manager ☕ ─────────────────────────────────────┐
│ rafctl dashboard                                                          │
├─ Profiles ────────────────────────────────────────────────────────────────┤
│   Name         Tool      Auth Mode    Status       Last Used              │
│ ▶ work         claude    oauth        ✓ Auth       2026-01-06 14:30       │
│   personal     claude    api-key      ✓ Auth       2026-01-05 09:15       │
│   client-a     claude    oauth        ✗ No Auth    never                  │
├───────────────────────────────────────────────────────────────────────────┤
│ ↑/k up  ↓/j down  Enter/r run  l login  q/Esc quit                        │
└───────────────────────────────────────────────────────────────────────────┘
```

## Keyboard Controls

| Key | Action |
|-----|--------|
| `↑` or `k` | Move selection up |
| `↓` or `j` | Move selection down |
| `Enter` or `r` | Run tool with selected profile |
| `l` | Login to selected profile |
| `q` or `Esc` | Exit dashboard |

## Table Columns

| Column | Description |
|--------|-------------|
| **Name** | Profile name |
| **Tool** | Tool type (claude, codex) |
| **Auth Mode** | Authentication mode (oauth, api-key) |
| **Status** | Authentication status (✓ Auth / ✗ No Auth) |
| **Last Used** | When the profile was last used with `rafctl run` |

## Visual Indicators

- **▶** Arrow shows the currently selected row
- **✓ Auth** (green) - Profile is authenticated and ready to use
- **✗ No Auth** (red) - Profile needs authentication

## Workflow Example

1. Launch dashboard: `rafctl dashboard`
2. Navigate with `j`/`k` to select a profile
3. If not authenticated, press `l` to login
4. Press `r` or `Enter` to run the tool

> **Note**: The run and login actions currently show a preview message. Exit the dashboard first, then run the command shown.

## Requirements

- Terminal with Unicode support (for box-drawing characters)
- Terminal with color support (for status indicators)
- Minimum terminal size: 80x24 recommended

## Terminal Compatibility

Tested on:
- iTerm2 (macOS)
- Terminal.app (macOS)
- Alacritty
- Kitty
- Most modern terminal emulators

## Exiting

Press `q` or `Esc` to exit the dashboard. The terminal will be restored to its normal state.

## Troubleshooting

### Screen looks garbled

Try resizing your terminal window. The dashboard should redraw automatically.

### Colors not showing

Ensure your terminal supports ANSI colors. Try:

```bash
echo $TERM
# Should show something like xterm-256color
```

### No profiles shown

Create profiles first:

```bash
rafctl profile add work --tool claude
rafctl profile add personal --tool claude
```

## See Also

- [Profile Management](./profiles.md) - Creating and managing profiles
- [Authentication](./authentication.md) - Logging in to profiles
- [Quota Monitoring](./quota-monitoring.md) - Checking usage limits
