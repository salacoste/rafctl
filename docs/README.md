# Documentation

Detailed guides for rafctl features.

> [!NOTE]
> For core features like installation, profile management, and authentication, please refer to the main [README.md](../README.md).

## Advanced Features

- **[Quota Monitoring](./quota-monitoring.md)** - Track Claude API usage limits
- **[TUI Dashboard](./dashboard.md)** - Interactive terminal interface
- **[Sessions & Analytics](./sessions.md)** - Usage analytics and session monitoring
- **[HUD Statusline](./hud.md)** - Native statusline plugin for Claude Code

## Quick Links

| Task | Command |
|------|---------|
| Create profile | `rafctl profile add <name> --tool claude` |
| Login | `rafctl auth login <profile>` |
| Run tool | `rafctl run <profile>` |
| Check quota | `rafctl quota` |
| View analytics | `rafctl analytics` |
| List sessions | `rafctl sessions` |
| Live watch | `rafctl watch` |
| Setup HUD | `rafctl hud install` |
| Open dashboard | `rafctl dashboard` |
| View status | `rafctl status` |
