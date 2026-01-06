# Documentation

User guides for rafctl features.

## Core Features

- **[Getting Started](./getting-started.md)** - Installation and first steps
- **[Profile Management](./profiles.md)** - Creating and managing profiles
- **[Authentication](./authentication.md)** - OAuth and API Key modes

## Advanced Features

- **[Quota Monitoring](./quota-monitoring.md)** - Track Claude API usage limits
- **[TUI Dashboard](./dashboard.md)** - Interactive terminal interface
- **[Shell Completions](./completions.md)** - Tab completion for your shell

## Reference

- **[Configuration](./configuration.md)** - Config files and settings
- **[Troubleshooting](./troubleshooting.md)** - Common issues and solutions

## Quick Links

| Task | Command |
|------|---------|
| Create profile | `rafctl profile add <name> --tool claude` |
| Login | `rafctl auth login <profile>` |
| Run tool | `rafctl run <profile>` |
| Check quota | `rafctl quota` |
| Open dashboard | `rafctl dashboard` |
| View status | `rafctl status` |
