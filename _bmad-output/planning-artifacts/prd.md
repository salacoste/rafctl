---
stepsCompleted: [1, 2, 3, 4, 5-skipped, 6, 7, 8, 9, 10, 11]
inputDocuments:
  - pre-prd.md
  - _bmad-output/project-context.md
documentCounts:
  briefs: 0
  research: 0
  brainstorming: 0
  projectDocs: 0
  prePrd: 1
  projectContext: 1
workflowType: 'prd'
lastStep: 11
---

# Product Requirements Document - rafctl

**Author:** Ivan
**Date:** 2025-01-05

## Executive Summary

**rafctl** is a CLI profile manager for AI coding agents that solves the configuration chaos developers face when working with multiple accounts across multiple LLM providers.

### The Problem

When developers use multiple accounts (personal, work, client projects) across tools like Claude Code and Codex CLI, they face constant friction:

- **Config file collisions** — Tools write credentials to global directories (`~/.claude/`, `~/.codex/`). Running one account overwrites the previous config. Switching back requires manual file juggling.
- **Multi-provider confusion** — Each tool has its own config location. Multiple accounts per tool multiplies the confusion. Developers lose track of which config is currently active.
- **No parallel execution** — It's impossible to run two terminals with different accounts of the same tool. They share the same credential files.
- **Manual workarounds** — Developers resort to backing up config files, symlinking, or re-authenticating every time they switch contexts.

### The Solution

rafctl provides **complete profile isolation** through environment variable overrides:

- Each profile gets its own isolated config directory
- `CLAUDE_CONFIG_DIR` and `CODEX_HOME` are redirected per-profile
- Zero overhead — no containers, no virtualization, just ENV isolation
- Run unlimited parallel instances with different accounts
- Always know which account is active in which terminal

### What Makes This Special

**Assumption challenged:** "You need containers or VMs for true isolation"

**Reality:** A simple environment variable override is all you need. rafctl leverages documented ENV variables that Claude Code and Codex CLI already support, providing instant, zero-overhead isolation without any infrastructure complexity.

## Project Classification

| Attribute | Value |
|-----------|-------|
| **Technical Type** | CLI Tool |
| **Domain** | Developer Tooling |
| **Complexity** | Low (general software practices) |
| **Project Context** | Greenfield — new project |
| **Language** | Rust |
| **Target Platform** | macOS, Linux, Windows (WSL) |


## Success Criteria

### User Success

- **Instant understanding** — User understands how to use rafctl within 2 minutes, without reading documentation
- **Works first time, every time** — Commands execute successfully on first attempt, consistently
- **No surprises** — Does exactly what's expected, nothing more, nothing less
- **Clear mental model** — User always knows which profile is active in which terminal

### Technical Success

| Metric | Target |
|--------|--------|
| CLI response time | < 100ms |
| Auth success rate | > 95% |
| Crash rate | < 0.1% |
| Profile isolation | 100% — profiles never interfere with each other |

### Business Success

- **Organic growth** — Word-of-mouth recommendations due to simplicity and reliability
- **GitHub stars** — 500 (3 months) → 2,000 (6 months)
- **Installs** — 1,000 (3 months) → 5,000 (6 months)
- **Active weekly users** — 200 (3 months) → 1,000 (6 months)

### Core Philosophy

> **Simple. Clear. Always works.**

No magic, no surprises. A utility that does one thing and does it reliably.

## Product Scope

### MVP — Minimum Viable Product (v0.1)

| Feature | Commands |
|---------|----------|
| Profile Management | `profile add`, `profile list`, `profile remove`, `profile show` |
| Authentication | `auth login`, `auth logout`, `auth status` |
| Execution | `run <profile>` |
| Status | `status` |
| Platforms | macOS, Linux |

### Growth Features (v0.2)

- TUI dashboard (`rafctl ui`)
- Quota monitoring with caching
- Shell completions (bash, zsh, fish)
- `rafctl shell <profile>` command
- Windows native support

### Vision (v1.0+)

- Desktop app (Tauri)
- System tray integration
- Native notifications
- Team/enterprise features
- Config sync between machines


## User Journeys

### Journey 1: Alex — First Time Setup

Alex is a frontend developer at a product company. He has a personal Claude account for pet projects and a work account from his company. Every time he switches between projects, he has to log out of one account and log into another. This is frustrating, especially when he needs to quickly check something in his personal project during the workday.

One day, a colleague shares a link to rafctl. Alex installs it via `cargo install rafctl` and runs it without arguments. He sees a welcome message with a quick start guide. In 30 seconds, he creates two profiles:

```bash
rafctl profile add work --tool claude
rafctl profile add personal --tool claude
```

He authorizes each via `rafctl auth login work` — browser opens, he logs in, done. Same for personal. He runs `rafctl status` — sees both profiles with green checkmarks.

Now he simply opens two terminals: `rafctl run work` in one, `rafctl run personal` in the other. Both Claude instances work simultaneously, each with its own account. Alex realizes he'll never have to manually re-login again.

**Requirements revealed:**
- Clear first-run experience with quick start guide
- Fast profile creation (single command)
- Simple browser-based authorization
- Visual status of all profiles

---

### Journey 2: Alex — Quota Exhausted, Switch Context

Two weeks later. Alex is actively working on a work project via `rafctl run work`. Mid-day, Claude shows that the quota is exhausted — he needs to wait 5 hours.

Alex opens a new terminal and runs `rafctl status`. He sees:

```
work      claude  ✓ Active   ██░░░░░░░░ 0% (resets in 4h 32m)
personal  claude  ✓ Active   ████████░░ 80%
```

He realizes he can continue working with his personal account. He runs `rafctl run personal` — and continues coding without downtime. When the work quota resets, he'll simply switch back.

No config juggling, no re-authentication. Just another terminal with a different profile.

**Requirements revealed:**
- Quick status overview of all profiles
- Quota display with reset time (when available)
- Instant context switching between profiles

---

### Journey Requirements Summary

| Capability | Source Journey |
|------------|----------------|
| Profile creation (`profile add`) | First Use |
| Profile listing (`profile list`) | First Use |
| Browser-based auth (`auth login`) | First Use |
| Status overview (`status`) | First Use, Daily Use |
| Isolated execution (`run`) | First Use, Daily Use |
| Quota display | Daily Use |
| Multi-profile parallel work | Both |


## Innovation & Novel Patterns

### New Product Category: AI Coding Agent Manager

As AI coding assistants proliferate (Claude Code, Codex CLI, Cursor, Aider, Continue, Cody, and more), developers face a new category of tooling pain:

- **Account sprawl** — Multiple accounts per provider (personal, work, client projects)
- **Quota chaos** — Different limits, different reset times, impossible to track mentally
- **Subscription fatigue** — Multiple subscriptions across providers
- **Config conflicts** — Each tool writes to global directories, causing collisions

rafctl creates a new product category: **AI Coding Agent Manager** — a meta-layer for managing the growing ecosystem of AI coding assistants.

### Unified Multi-Provider Architecture

Unlike single-provider solutions, rafctl provides:

- **One interface for all AI coding agents** — Claude, Codex today; extensible to others
- **Consistent profile management** — Same commands regardless of underlying tool
- **Aggregated visibility** — See all accounts, all quotas, all statuses in one view
- **Plugin-ready architecture** — New providers can be added without changing core UX

### Validation Approach

- Start with Claude Code + Codex (most popular AI coding CLIs)
- Validate core isolation mechanism works reliably
- Gather feedback from early adopters on multi-provider needs
- Prioritize next providers based on user demand

### Future Vision

rafctl evolves from profile manager to comprehensive AI coding workflow layer:

- Session history across providers
- Token usage analytics
- Cost tracking across subscriptions
- Performance comparison between models


## CLI Tool Specific Requirements

### Interaction Modes

rafctl supports both interactive and scriptable usage:

| Mode | Use Case | Features |
|------|----------|----------|
| **Interactive** | Human at terminal | Colored output, tables, progress indicators, helpful messages |
| **Scriptable** | Automation, CI/CD | Machine-readable output, exit codes, no interactive prompts |

### Output Formats

| Format | Flag | Description |
|--------|------|-------------|
| Human-readable | (default) | Colored text, tables via `comfy-table`, emoji indicators |
| JSON | `--json` | Structured output for scripting and parsing |
| Plain text | `--plain` | No colors/formatting, suitable for pipes and logs |
| YAML | `--yaml` | Alternative structured format |

### Configuration Hierarchy

Priority order (highest to lowest):

1. **CLI flags** — `--profile work`, `--json`
2. **Environment vars** — `RAFCTL_DEFAULT_PROFILE`, `RAFCTL_OUTPUT_FORMAT`
3. **Global config** — `~/.rafctl/config.yaml`
4. **[v0.2] Local config** — `./.rafctl.yaml` (project-specific)

#### Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `RAFCTL_DEFAULT_PROFILE` | Override default profile | `work` |
| `RAFCTL_OUTPUT_FORMAT` | Force output format | `json`, `plain`, `yaml` |
| `RAFCTL_CONFIG_DIR` | Custom config directory | `~/.rafctl-test` |

#### Global Config (~/.rafctl/config.yaml)

```yaml
default_profile: work          # Last used or explicitly set
output_format: human           # Default output format
color: auto                    # auto | always | never
```

### Command Structure

```
rafctl [global-flags] <command> [command-flags] [args]

Global flags:
  --profile <name>    Override profile for this command
  --json              Output as JSON
  --plain             No colors/formatting
  --yaml              Output as YAML
  --config <path>     Custom config file
  --verbose, -v       Verbose output
  --quiet, -q         Suppress non-essential output
  --help, -h          Show help
  --version, -V       Show version

Commands:
  profile             Manage profiles (add, list, remove, show)
  auth                Authentication (login, logout, status)
  run                 Run tool with profile
  status              Show status of all profiles
  shell               Open shell with profile environment [v0.2]
  completion          Generate shell completions
```

### Shell Completion (MVP)

Built-in completion generator supporting:

| Shell | Command |
|-------|---------|
| Bash | `rafctl completion bash > ~/.bash_completion.d/rafctl` |
| Zsh | `rafctl completion zsh > ~/.zfunc/_rafctl` |
| Fish | `rafctl completion fish > ~/.config/fish/completions/rafctl.fish` |

#### Completion Features

- Command and subcommand completion
- Flag completion with descriptions
- **Dynamic profile name completion** — `rafctl run <TAB>` shows available profiles
- Tool type completion (`--tool <TAB>` shows `claude`, `codex`)

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid arguments / usage error |
| 3 | Profile not found |
| 4 | Authentication error |
| 5 | Tool execution error |

### Scripting Support

```bash
# Check if profile exists
if rafctl profile show work --json | jq -e '.name' > /dev/null; then
  echo "Profile exists"
fi

# Get all profile names as array
profiles=$(rafctl profile list --json | jq -r '.[].name')

# Run with specific profile in CI
RAFCTL_DEFAULT_PROFILE=ci-account rafctl run
```


## Project Scoping & Phased Development

### MVP Strategy & Philosophy

**MVP Approach:** Problem-Solving MVP
> Solve the core problem with minimal features. No unnecessary embellishments.

**Resource Requirements:** 1-2 developers, 2-4 weeks to MVP

**Core Principle:** Simple. Clear. Always works.

### MVP Feature Set (Phase 1 — v0.1)

**Core User Journeys Supported:**
- First Time Setup (profile creation, auth, first run)
- Quota Exhausted Context Switch (status check, profile switch)

**Must-Have Capabilities:**

| Feature | Priority | Rationale |
|---------|----------|-----------|
| `profile add <name> --tool <type>` | P0 | Cannot use tool without profiles |
| `profile list` | P0 | Must see available profiles |
| `profile remove <name>` | P1 | Cleanup capability |
| `profile show <name>` | P1 | View profile details |
| `auth login <profile>` | P0 | Cannot use without auth |
| `auth logout <profile>` | P1 | Security requirement |
| `auth status [profile]` | P0 | Must verify auth state |
| `run <profile>` | P0 | Core functionality |
| `run` (default profile) | P0 | UX convenience |
| `status` | P0 | Overview of all profiles |
| Shell completion | P1 | Essential UX for CLI |
| ENV-based isolation | P0 | Core mechanism |

**Platforms:** macOS, Linux

**Output Formats:** Human-readable (default), JSON (`--json`), Plain (`--plain`)

### Post-MVP Features

**Phase 2 — Growth (v0.2)**

| Feature | Description |
|---------|-------------|
| Quota monitoring | Parse `/status` output, cache results, display remaining quota |
| TUI dashboard | `rafctl ui` — interactive terminal interface (ratatui) |
| `rafctl shell` | Open shell with profile environment pre-configured |
| Windows native | PowerShell support, native paths |
| Local project config | `.rafctl.yaml` for project-specific defaults |
| YAML output | `--yaml` format option |

**Phase 3 — Expansion (v1.0+)**

| Feature | Description |
|---------|-------------|
| Desktop app | Tauri-based GUI application |
| System tray | Quick profile switching from menu bar |
| Notifications | Quota alerts, auth expiry warnings |
| New providers | Cursor, Aider, Continue, Cody support |
| Plugin architecture | Community-contributed provider plugins |
| Team features | Shared profiles, usage analytics |
| Config sync | Sync profiles across machines |

### Risk Mitigation Strategy

**Technical Risks:**
| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| ENV variables change | High | Low | Abstract tool configs, version compatibility layer |
| Auth flow changes | High | Low | Wrapper approach, fallback to manual token |
| Quota API unavailable | Medium | Medium | Graceful degradation, show "Unknown" |

**Market Risks:**
| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Competitor launches | Medium | Medium | Speed to market, community focus, UX excellence |
| Tools become obsolete | Low | Low | Plugin architecture enables pivoting |

**Resource Risks:**
| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Solo developer | Medium | High | Lean MVP, ruthless prioritization, community help |
| Time constraints | Medium | Medium | Phase approach, ship early, iterate |


## Functional Requirements

### Profile Management

- **FR1:** User can create a new profile with a name and tool type
- **FR2:** User can list all existing profiles
- **FR3:** User can view details of a specific profile
- **FR4:** User can delete an existing profile
- **FR5:** User can see when a profile was created and last used
- **FR6:** System generates unique profile name if not provided (profile-YYYYMMDD-HHMMSS)

### Authentication

- **FR7:** User can authenticate a profile through browser-based OAuth flow
- **FR8:** User can view authentication status of a profile
- **FR9:** User can view authentication status of all profiles at once
- **FR10:** User can log out of a profile (revoke stored credentials)
- **FR11:** System stores credentials in profile-isolated directory

### Execution

- **FR12:** User can run a tool (Claude/Codex) with a specific profile
- **FR13:** User can run a tool with the default (last used) profile
- **FR14:** System isolates tool execution via environment variable override
- **FR15:** System passes through additional arguments to the underlying tool
- **FR16:** System updates "last used" timestamp after successful run

### Status & Monitoring

- **FR17:** User can view status summary of all profiles
- **FR18:** User can view detailed status of a specific profile
- **FR19:** Status displays profile name, tool type, auth state
- **FR20:** Status indicates which profile is the default

### Configuration

- **FR21:** User can override default profile via environment variable
- **FR22:** User can override default profile via CLI flag
- **FR23:** System reads global config from ~/.rafctl/config.yaml
- **FR24:** System follows config priority: CLI flags > ENV > config file

### Output & Formatting

- **FR25:** User can request JSON output for any command
- **FR26:** User can request plain text output (no colors) for any command
- **FR27:** System displays colored, human-readable output by default
- **FR28:** System uses appropriate exit codes for scripting

### Shell Integration

- **FR29:** User can generate shell completion scripts (bash, zsh, fish)
- **FR30:** Shell completion includes dynamic profile name suggestions
- **FR31:** Shell completion includes command and flag suggestions

### Multi-Provider Support

- **FR32:** System supports Claude Code via CLAUDE_CONFIG_DIR
- **FR33:** System supports Codex CLI via CODEX_HOME
- **FR34:** System architecture allows adding new providers


## Non-Functional Requirements

### Performance

| NFR | Requirement | Measurement |
|-----|-------------|-------------|
| **NFR1** | CLI commands complete within 100ms | Excluding network operations (auth, tool launch) |
| **NFR2** | Profile listing returns instantly for up to 100 profiles | < 50ms response time |
| **NFR3** | Tool launch adds < 50ms overhead | Compared to direct tool invocation |
| **NFR4** | Status command completes within 200ms | Including file system reads |

### Security

| NFR | Requirement | Measurement |
|-----|-------------|-------------|
| **NFR5** | Credentials stored with 600 permissions | Owner read/write only |
| **NFR6** | No credentials logged or displayed | Even in verbose mode |
| **NFR7** | Config directory created with 700 permissions | Owner access only |
| **NFR8** | No credentials passed via command line arguments | Prevents ps/history exposure |

### Reliability

| NFR | Requirement | Measurement |
|-----|-------------|-------------|
| **NFR9** | Crash rate < 0.1% | Across all command invocations |
| **NFR10** | Graceful degradation on missing config | Clear error messages, no panic |
| **NFR11** | Profile isolation guaranteed | No cross-profile data leakage |
| **NFR12** | Atomic config file writes | No corruption on interrupted writes |

### Compatibility

| NFR | Requirement | Measurement |
|-----|-------------|-------------|
| **NFR13** | Works on macOS 12+ (Intel and ARM) | Tested on both architectures |
| **NFR14** | Works on Linux (glibc 2.17+) | Major distributions: Ubuntu, Debian, Fedora |
| **NFR15** | Works on Windows WSL2 | Ubuntu and Debian on WSL |
| **NFR16** | Single static binary | No runtime dependencies |
| **NFR17** | UTF-8 support in profile names | International characters allowed |

### Usability

| NFR | Requirement | Measurement |
|-----|-------------|-------------|
| **NFR18** | Helpful error messages | Include suggested fixes |
| **NFR19** | Consistent command structure | All commands follow same patterns |
| **NFR20** | Works without internet | Except for auth and tool execution |

