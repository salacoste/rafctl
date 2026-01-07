---
stepsCompleted: [1, 2, 3, 4]
inputDocuments:
  - _bmad-output/planning-artifacts/prd.md
  - _bmad-output/planning-artifacts/architecture.md
  - _bmad-output/project-context.md
workflowType: 'epics-and-stories'
project_name: 'rafctl'
user_name: 'Ivan'
date: '2026-01-06'
status: 'complete'
---

# rafctl - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for rafctl, decomposing the requirements from the PRD and Architecture into implementable stories.

## Requirements Inventory

### Functional Requirements

**Profile Management:**
- FR1: User can create a new profile with a name and tool type
- FR2: User can list all existing profiles
- FR3: User can view details of a specific profile
- FR4: User can delete an existing profile
- FR5: User can see when a profile was created and last used
- FR6: System generates unique profile name if not provided

**Authentication:**
- FR7: User can authenticate a profile through browser-based OAuth flow
- FR8: User can view authentication status of a profile
- FR9: User can view authentication status of all profiles at once
- FR10: User can log out of a profile (revoke stored credentials)
- FR11: System stores credentials in profile-isolated directory

**Execution:**
- FR12: User can run a tool (Claude/Codex) with a specific profile
- FR13: User can run a tool with the default (last used) profile
- FR14: System isolates tool execution via environment variable override
- FR15: System passes through additional arguments to the underlying tool
- FR16: System updates "last used" timestamp after successful run

**Status & Monitoring:**
- FR17: User can view status summary of all profiles
- FR18: User can view detailed status of a specific profile
- FR19: Status displays profile name, tool type, auth state
- FR20: Status indicates which profile is the default

**Configuration:**
- FR21: User can override default profile via environment variable
- FR22: User can override default profile via CLI flag
- FR23: System reads global config from ~/.rafctl/config.yaml
- FR24: System follows config priority: CLI flags > ENV > config file

**Output & Formatting:**
- FR25: User can request JSON output for any command
- FR26: User can request plain text output (no colors) for any command
- FR27: System displays colored, human-readable output by default
- FR28: System uses appropriate exit codes for scripting

**Shell Integration:**
- FR29: User can generate shell completion scripts (bash, zsh, fish)
- FR30: Shell completion includes dynamic profile name suggestions
- FR31: Shell completion includes command and flag suggestions

**Multi-Provider Support:**
- FR32: System supports Claude Code via CLAUDE_CONFIG_DIR
- FR33: System supports Codex CLI via CODEX_HOME
- FR34: System architecture allows adding new providers

### Non-Functional Requirements

**Performance:**
- NFR1: CLI commands complete within 100ms
- NFR2: Profile listing returns instantly for up to 100 profiles (<50ms)
- NFR3: Tool launch adds <50ms overhead
- NFR4: Status command completes within 200ms

**Security:**
- NFR5: Credentials stored with 600 permissions
- NFR6: No credentials logged or displayed
- NFR7: Config directory created with 700 permissions
- NFR8: No credentials passed via command line arguments

**Reliability:**
- NFR9: Crash rate <0.1%
- NFR10: Graceful degradation on missing config
- NFR11: Profile isolation guaranteed
- NFR12: Atomic config file writes

**Compatibility:**
- NFR13: Works on macOS 12+ (Intel and ARM)
- NFR14: Works on Linux (glibc 2.17+)
- NFR15: Works on Windows WSL2
- NFR16: Single static binary
- NFR17: UTF-8 support in profile names

**Usability:**
- NFR18: Helpful error messages
- NFR19: Consistent command structure
- NFR20: Works without internet

### Additional Requirements (from Architecture)

- Starter template: Manual setup per AGENTS.md (no external template)
- Atomic file writes using temp + rename pattern
- Error handling: thiserror (library) + anyhow (application)
- Path handling: PathBuf everywhere, never String
- Output formatting: colored + comfy-table, respect NO_COLOR
- Testing: tempfile::TempDir for isolation
- CI/CD: GitHub Actions (fmt, clippy, test)
- Release targets: Linux x64, macOS Intel/ARM, Windows x64

### FR Coverage Map

| Epic | Stories | FRs Covered |
|------|---------|-------------|
| Epic 1: Foundation | 1.1-1.2 | (Infrastructure) |
| Epic 2: Profile Management | 2.1-2.4 | FR1-FR6 |
| Epic 3: Tool Abstraction | 3.1-3.2 | FR32-FR34 |
| Epic 4: Authentication | 4.1-4.3 | FR7-FR11 |
| Epic 5: Execution | 5.1-5.2 | FR12-FR16 |
| Epic 6: Status & Config | 6.1-6.3 | FR17-FR24 |
| Epic 7: Output & Shell | 7.1-7.2 | FR25-FR31 |
| Epic 8: Quality & Release | 8.1-8.3 | NFR1-NFR20 |

## Epic List

1. **Epic 1: Project Foundation** — Scaffolding and CLI skeleton
2. **Epic 2: Profile Management** — Profile CRUD operations
3. **Epic 3: Tool Abstraction** — Claude/Codex tool support layer
4. **Epic 4: Authentication** — Auth login/logout/status flows
5. **Epic 5: Tool Execution** — Run command with ENV isolation
6. **Epic 6: Status & Configuration** — Status display and config hierarchy
7. **Epic 7: Output & Shell Integration** — Formatting and completions
8. **Epic 8: Quality Assurance & Release** — Testing, CI/CD, release

---

## Epic 1: Project Foundation

**Goal:** Establish the project scaffolding and CLI skeleton with all subcommands declared but not implemented.

### Story 1.1: Project Scaffolding

As a **developer**,
I want **the project structure created with all dependencies configured**,
So that **I can start implementing features with a working build**.

**Acceptance Criteria:**

**Given** no project exists
**When** scaffolding is complete
**Then** the following structure exists:
- `Cargo.toml` with all dependencies from Architecture
- `src/main.rs` (5-line entry point)
- `src/lib.rs` (library exports)
- `src/cli/mod.rs` (CLI module)
- `src/core/mod.rs` (core module)
- `src/tools/mod.rs` (tools module)
- `src/error.rs` (error types)
- `tests/integration.rs` (test structure)

**And** `cargo build` passes with no errors
**And** `cargo test` passes (0 tests, no errors)
**And** `cargo fmt --check` passes
**And** `cargo clippy -- -D warnings` passes

**Technical Notes:**
- Use Cargo.toml from Architecture document exactly
- Create empty mod.rs files with module declarations
- Follow project structure from Architecture

---

### Story 1.2: CLI Skeleton with Clap

As a **user**,
I want **all CLI commands declared and showing help text**,
So that **I can see the full command structure before implementation**.

**Acceptance Criteria:**

**Given** the project is scaffolded
**When** I run `rafctl --help`
**Then** I see all top-level commands: profile, auth, run, status

**Given** the CLI is implemented
**When** I run `rafctl profile --help`
**Then** I see subcommands: add, list, remove, show

**Given** the CLI is implemented
**When** I run `rafctl auth --help`
**Then** I see subcommands: login, logout, status

**Given** the CLI is implemented
**When** I run `rafctl status`
**Then** I see "Not implemented yet" message

**And** `rafctl --version` shows version from Cargo.toml
**And** all commands have doc comments for help text

**Technical Notes:**
- Use lib.rs template from Architecture
- Use clap derive macros
- Each command prints stub message

---

## Epic 2: Profile Management

**Goal:** Implement full profile CRUD operations with persistent storage.

### Story 2.1: Core Profile Data Structures

As a **developer**,
I want **Profile and ToolType data structures defined**,
So that **I can store and manipulate profile data**.

**Acceptance Criteria:**

**Given** the core module exists
**When** Profile struct is implemented
**Then** it has fields: name (String), tool (ToolType), created_at (DateTime), last_used (Option<DateTime>)

**Given** ToolType enum is implemented
**When** serialized to YAML
**Then** it produces lowercase string ("claude" or "codex")

**And** Profile implements Serialize and Deserialize
**And** ToolType implements Clone, Copy, Debug, PartialEq
**And** unit tests verify serialization round-trip

**Technical Notes:**
- Place in `src/core/profile.rs`
- Use chrono for DateTime<Utc>
- Use serde rename for YAML keys

---

### Story 2.2: Profile Storage with Atomic Writes

As a **developer**,
I want **profile data persisted to YAML files atomically**,
So that **data is never corrupted on interrupted writes**.

**Acceptance Criteria:**

**Given** a Profile needs to be saved
**When** save_profile() is called
**Then** file is written to `~/.rafctl/profiles/<name>/meta.yaml`

**Given** atomic writes are required
**When** writing to disk
**Then** content is written to `.yaml.tmp` first, then renamed

**Given** directory doesn't exist
**When** saving a profile
**Then** directory is created with 700 permissions (Unix)

**Given** file is created
**When** on Unix systems
**Then** file has 600 permissions

**And** load_profile() reads and deserializes correctly
**And** list_profiles() returns all profile names
**And** delete_profile() removes directory recursively
**And** integration tests use tempfile::TempDir

**Technical Notes:**
- Place in `src/core/profile.rs`
- Use dirs::home_dir() for home directory
- Use #[cfg(unix)] for permissions

---

### Story 2.3: Profile Add Command

As a **user**,
I want **to create a new profile with a name and tool type**,
So that **I can manage multiple accounts**.

**Acceptance Criteria:**

**Given** I run `rafctl profile add work --tool claude`
**When** profile doesn't exist
**Then** profile is created with name "work" and tool "claude"
**And** success message shows "✓ Profile 'work' created"

**Given** profile name "work" already exists
**When** I run `rafctl profile add work --tool claude`
**Then** error shows "✗ Profile 'work' already exists"
**And** exit code is 1

**Given** invalid tool type "invalid"
**When** I run `rafctl profile add test --tool invalid`
**Then** error shows valid options: claude, codex
**And** exit code is 2

**Given** profile name contains invalid characters
**When** I run `rafctl profile add "work@home" --tool claude`
**Then** error shows valid pattern: [a-zA-Z0-9_-]+

**And** profile names are case-insensitive (Work == work)
**And** max profile name length is 64 characters

**Technical Notes:**
- Validate name before creating
- Use colored crate for output
- Covers FR1, FR6

---

### Story 2.4: Profile List, Show, Remove Commands

As a **user**,
I want **to view and manage my existing profiles**,
So that **I can see what's available and clean up unused profiles**.

**Acceptance Criteria:**

**Given** profiles exist
**When** I run `rafctl profile list`
**Then** I see a table with columns: Name, Tool, Created, Last Used
**And** profiles are sorted alphabetically

**Given** profile "work" exists
**When** I run `rafctl profile show work`
**Then** I see detailed info: name, tool, created_at, last_used, config directory path

**Given** profile "work" exists
**When** I run `rafctl profile remove work`
**Then** profile directory is deleted
**And** success message shows "✓ Profile 'work' removed"

**Given** profile "nonexistent" doesn't exist
**When** I run `rafctl profile show nonexistent`
**Then** error shows "✗ Profile 'nonexistent' not found"
**And** if similar name exists, show "Did you mean 'work'?"

**And** comfy-table is used for table output
**And** empty list shows helpful message

**Technical Notes:**
- Covers FR2, FR3, FR4, FR5
- Use prefix matching for "did you mean"

---

## Epic 3: Tool Abstraction

**Goal:** Create abstraction layer for Claude Code and Codex CLI tools.

### Story 3.1: ToolType Implementation

As a **developer**,
I want **tool-specific logic encapsulated in the tools module**,
So that **adding new providers is straightforward**.

**Acceptance Criteria:**

**Given** ToolType::Claude
**When** env_var_name() is called
**Then** returns "CLAUDE_CONFIG_DIR"

**Given** ToolType::Codex
**When** env_var_name() is called
**Then** returns "CODEX_HOME"

**Given** ToolType::Claude
**When** command_name() is called
**Then** returns "claude"

**Given** ToolType::Codex
**When** command_name() is called
**Then** returns "codex"

**Given** any ToolType
**When** install_url() is called
**Then** returns appropriate installation URL

**And** credential_path() returns expected credential file location
**And** all methods are documented with doc comments

**Technical Notes:**
- Place in `src/tools/mod.rs`
- Create `src/tools/claude.rs` and `src/tools/codex.rs` for tool-specific constants
- Covers FR32, FR33, FR34

---

### Story 3.2: Tool Availability Check

As a **user**,
I want **helpful error messages when a tool isn't installed**,
So that **I know how to fix the problem**.

**Acceptance Criteria:**

**Given** claude command is not in PATH
**When** I try to run with Claude profile
**Then** error shows:
```
✗ 'claude' command not found

Claude Code is not installed or not in PATH.

Install: https://claude.ai/download
```

**Given** codex command is not in PATH
**When** I try to run with Codex profile
**Then** similar helpful error with install URL

**And** check happens before attempting to spawn process
**And** exit code is 5 (tool execution error)

**Technical Notes:**
- Use std::process::Command::new().spawn() to detect NotFound
- Covers NFR18

---

## Epic 4: Authentication

**Goal:** Implement authentication flow using tool's native OAuth.

### Story 4.1: Auth Login Command

As a **user**,
I want **to authenticate a profile through the tool's OAuth flow**,
So that **I can use my account with that profile**.

**Acceptance Criteria:**

**Given** profile "work" exists and is not authenticated
**When** I run `rafctl auth login work`
**Then** message shows "Opening browser for authorization..."
**And** tool is spawned with ENV isolation
**And** message shows "→ Waiting for authentication..."

**Given** authentication succeeds
**When** tool exits
**Then** credential file exists in profile directory
**And** message shows "✓ Authenticated successfully!"

**Given** profile doesn't exist
**When** I run `rafctl auth login nonexistent`
**Then** error shows profile not found
**And** exit code is 3

**And** timeout is 10 minutes by default
**And** user can cancel with Ctrl+C

**Technical Notes:**
- Spawn tool with isolated ENV
- Check credential file existence after exit
- Covers FR7, FR11

---

### Story 4.2: Auth Status Command

As a **user**,
I want **to check authentication status of profiles**,
So that **I know which profiles are ready to use**.

**Acceptance Criteria:**

**Given** profile "work" is authenticated
**When** I run `rafctl auth status work`
**Then** shows "✓ Authenticated" with last_used date if available

**Given** profile "work" is not authenticated
**When** I run `rafctl auth status work`
**Then** shows "✗ Not authenticated"

**Given** profile was last used > 7 days ago
**When** I run `rafctl auth status work`
**Then** shows "⚠ Auth may need refresh"

**Given** no profile specified
**When** I run `rafctl auth status`
**Then** shows auth status for all profiles in table format

**And** status is determined by credential file existence
**And** never reads credential contents

**Technical Notes:**
- Use StatusIndicator enum
- Covers FR8, FR9

---

### Story 4.3: Auth Logout Command

As a **user**,
I want **to log out of a profile**,
So that **credentials are removed for security**.

**Acceptance Criteria:**

**Given** profile "work" is authenticated
**When** I run `rafctl auth logout work`
**Then** credential files are deleted
**And** message shows "✓ Logged out of 'work'"

**Given** profile "work" is not authenticated
**When** I run `rafctl auth logout work`
**Then** message shows "Profile 'work' is not authenticated"
**And** exit code is 0 (not an error)

**And** profile metadata is preserved (only credentials removed)
**And** confirmation not required (quick operation)

**Technical Notes:**
- Delete only credential files, not meta.yaml
- Covers FR10

---

## Epic 5: Tool Execution

**Goal:** Implement the run command with full ENV isolation.

### Story 5.1: Run Command with Profile

As a **user**,
I want **to run a tool with a specific profile's credentials**,
So that **I can use the correct account**.

**Acceptance Criteria:**

**Given** profile "work" exists and is authenticated
**When** I run `rafctl run work`
**Then** tool is spawned with CLAUDE_CONFIG_DIR or CODEX_HOME set
**And** tool receives the isolated config directory
**And** rafctl waits for tool to exit

**Given** profile "work" is not authenticated
**When** I run `rafctl run work`
**Then** error shows "Profile 'work' is not authenticated"
**And** suggestion: "Run: rafctl auth login work"

**Given** additional arguments provided
**When** I run `rafctl run work -- --some-flag`
**Then** arguments after `--` are passed to the tool

**And** last_used timestamp is updated after successful run
**And** exit code matches tool's exit code

**Technical Notes:**
- Use std::process::Command with env()
- Covers FR12, FR14, FR15, FR16

---

### Story 5.2: Run with Default Profile

As a **user**,
I want **to run without specifying a profile**,
So that **I can quickly use my most recent profile**.

**Acceptance Criteria:**

**Given** I previously ran `rafctl run work`
**When** I run `rafctl run` (no profile)
**Then** "work" profile is used (last used)

**Given** no profile has ever been used
**When** I run `rafctl run`
**Then** error shows "No default profile. Specify a profile or run one first."
**And** lists available profiles

**Given** RAFCTL_DEFAULT_PROFILE=personal is set
**When** I run `rafctl run`
**Then** "personal" profile is used (ENV override)

**Given** --profile work flag is provided
**When** I run `rafctl run --profile work`
**Then** "work" profile is used (CLI flag override)

**And** priority: CLI flag > ENV > config > last_used

**Technical Notes:**
- Store last_used in global config
- Covers FR13, FR21, FR22

---

## Epic 6: Status & Configuration

**Goal:** Implement status command and configuration management.

### Story 6.1: Status Command

As a **user**,
I want **to see status of all my profiles at a glance**,
So that **I know what's available and their states**.

**Acceptance Criteria:**

**Given** multiple profiles exist
**When** I run `rafctl status`
**Then** I see a table with: Name, Tool, Auth Status, Last Used
**And** default profile is marked with (default)

**Given** profile "work" exists
**When** I run `rafctl status work`
**Then** I see detailed status for that profile only

**And** auth status shows colored indicators (✓ green, ⚠ yellow, ✗ red)
**And** table is formatted with comfy-table
**And** empty state shows helpful message

**Technical Notes:**
- Covers FR17, FR18, FR19, FR20

---

### Story 6.2: Global Configuration

As a **user**,
I want **my preferences stored in a config file**,
So that **I don't have to specify them every time**.

**Acceptance Criteria:**

**Given** ~/.rafctl/config.yaml doesn't exist
**When** first profile is created
**Then** config.yaml is created with defaults

**Given** config.yaml exists with default_profile: work
**When** I run `rafctl run`
**Then** "work" profile is used

**Given** config.yaml has output_format: json
**When** I run `rafctl status`
**Then** output is JSON (unless --plain overrides)

**And** config file uses atomic writes
**And** missing config file doesn't cause error

**Technical Notes:**
- Covers FR23, FR24

---

### Story 6.3: Config Priority Chain

As a **developer**,
I want **configuration priority properly implemented**,
So that **CLI flags always win over other sources**.

**Acceptance Criteria:**

**Given** CLI flag --profile work
**When** ENV has RAFCTL_DEFAULT_PROFILE=personal
**And** config has default_profile: other
**Then** "work" is used (CLI wins)

**Given** no CLI flag
**When** ENV has RAFCTL_DEFAULT_PROFILE=personal
**Then** "personal" is used (ENV wins over config)

**Given** no CLI flag, no ENV
**When** config has default_profile: other
**Then** "other" is used

**And** priority is: CLI > ENV > config > last_used
**And** each level is tested independently

**Technical Notes:**
- Implement in core/config.rs
- Covers FR24

---

## Epic 7: Output & Shell Integration

**Goal:** Implement output formatting options and shell completions.

### Story 7.1: Output Format Options

As a **user**,
I want **to get output in different formats**,
So that **I can script with rafctl or pipe to other tools**.

**Acceptance Criteria:**

**Given** I run `rafctl profile list --json`
**Then** output is valid JSON array of profiles

**Given** I run `rafctl status --plain`
**Then** output has no colors or emoji
**And** suitable for piping to other commands

**Given** NO_COLOR env var is set
**When** I run any command
**Then** colors are disabled automatically

**And** --json and --plain are mutually exclusive
**And** JSON output includes all relevant fields

**Technical Notes:**
- Covers FR25, FR26, FR27, FR28

---

### Story 7.2: Shell Completions

As a **user**,
I want **tab completion in my shell**,
So that **I can type commands faster**.

**Acceptance Criteria:**

**Given** I run `rafctl completion bash`
**Then** output is valid bash completion script

**Given** I run `rafctl completion zsh`
**Then** output is valid zsh completion script

**Given** I run `rafctl completion fish`
**Then** output is valid fish completion script

**Given** completions are installed
**When** I type `rafctl run <TAB>`
**Then** available profile names are suggested

**And** flag completions include descriptions
**And** clap_complete is used for generation

**Technical Notes:**
- Covers FR29, FR30, FR31

---

## Epic 8: Quality Assurance & Release

**Goal:** Implement comprehensive testing, CI/CD, and release pipeline.

### Story 8.1: Integration Tests

As a **developer**,
I want **comprehensive integration tests**,
So that **I can refactor with confidence**.

**Acceptance Criteria:**

**Given** test suite exists
**When** running `cargo test`
**Then** all tests pass

**Given** isolation test
**When** two profiles are created and run in parallel
**Then** they don't share any config files (CRITICAL)

**And** tests use tempfile::TempDir for isolation
**And** tests cover all CLI commands
**And** tests verify exit codes

**Technical Notes:**
- Place in tests/integration.rs
- Covers NFR11

---

### Story 8.2: CI/CD Pipeline

As a **developer**,
I want **automated CI/CD on GitHub Actions**,
So that **every PR is validated and releases are automated**.

**Acceptance Criteria:**

**Given** PR is opened
**When** CI runs
**Then** these checks pass:
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test`

**Given** matrix testing
**When** CI runs
**Then** tests run on: ubuntu-latest, macos-latest

**Given** tag is pushed (v*.*.*)
**When** release workflow runs
**Then** binaries are built for 4 targets
**And** GitHub Release is created with binaries

**And** workflow files are in .github/workflows/

**Technical Notes:**
- Covers NFR13, NFR14, NFR15, NFR16

---

### Story 8.3: Error Handling & UX Polish

As a **user**,
I want **helpful error messages and consistent UX**,
So that **I can fix problems without searching documentation**.

**Acceptance Criteria:**

**Given** any error occurs
**When** displayed to user
**Then** includes:
- Clear description of what went wrong
- Suggestion for how to fix it (when applicable)
- Appropriate exit code

**Given** profile "wrk" not found
**When** "work" exists
**Then** error includes "Did you mean 'work'?"

**Given** tool not installed
**When** trying to run
**Then** error includes installation URL

**And** all errors use consistent format
**And** no panics in production code
**And** unwrap() never used in production

**Technical Notes:**
- Covers NFR9, NFR10, NFR18, NFR19

---

## Epic 9: Hardening & Polish (POST-REVIEW)

**Goal:** Address findings from party mode review, improve robustness and reduce tech debt.

**Source:** Party Mode Review (Epics 1-5), 2026-01-06
**Priority:** Should complete before v1.0 release
**Full Details:** See `_bmad-output/planning-artifacts/backlog-improvements.md`

### Story 9.0: Test Isolation Infrastructure [DEV]

As a **developer**,
I want **`RAFCTL_CONFIG_DIR` environment variable to override config location**,
So that **integration tests can run in isolation without affecting real config**.

**Acceptance Criteria:**
- `RAFCTL_CONFIG_DIR` env var overrides `~/.rafctl` location
- All path functions respect this override
- Tests can use `tempfile::TempDir` for full isolation

**Effort:** S (1-2h)

---

### Story 9.1: Verify Tool CLI Interfaces [DEV] 🔴

As a **developer**,
I want **to verify the actual CLI interfaces of Claude Code and Codex CLI**,
So that **auth login and run commands work correctly with real tools**.

**Research Tasks:**
- Check Claude Code CLI: `claude --help`, `claude auth --help`
- Check Codex CLI documentation
- Update auth commands if needed

**Effort:** M (2-4h)
**Risk:** 🔴 High — if wrong, auth won't work

---

### Story 9.2: Add Auth Login Timeout [DEV] 🟡

As a **user**,
I want **authentication to timeout after 10 minutes**,
So that **I don't have hanging processes if I forget to complete auth**.

**Effort:** S (1-2h)

---

### Story 9.3: Replace Regex with Simple Validation [DEV] 🟢

As a **developer**,
I want **to remove regex dependency for profile name validation**,
So that **binary size is reduced by ~200KB**.

**Effort:** S (1-2h)

---

### Story 9.4: Log Errors Instead of Ignoring [DEV] 🟢

Replace `let _ = save_profile(...)` with proper warning logging.

**Effort:** XS (<1h)

---

### Story 9.5: Fix Deprecated cargo_bin Warning [DEV] 🟢

Update integration tests to use non-deprecated assert_cmd API.

**Effort:** XS (<1h)

---

### Story 9.6: Integration Tests for CLI Commands [QA] 🟡

Comprehensive integration tests for profile/auth/run commands.

**Effort:** L (4-8h)
**Blocked by:** Story 9.0

---

## Implementation Order

**Recommended sequence for MVP:**

1. Epic 1 (Foundation) — Stories 1.1, 1.2 ✅
2. Epic 2 (Profiles) — Stories 2.1, 2.2, 2.3, 2.4 ✅
3. Epic 3 (Tools) — Stories 3.1, 3.2 ✅
4. Epic 4 (Auth) — Stories 4.1, 4.2, 4.3 ✅
5. Epic 5 (Execution) — Stories 5.1, 5.2 ✅
6. Epic 6 (Status/Config) — Stories 6.1, 6.2, 6.3 ✅
7. Epic 7 (Output/Shell) — Stories 7.1, 7.2 ✅
8. Epic 8 (Quality/Release) — Stories 8.1, 8.2, 8.3 ✅
9. Epic 9 (Hardening/Polish) — Stories 9.0-9.6 ✅ (v0.2.1)
10. Epic 10 (Quota Monitoring) — ✅ (v0.2.0)
11. Epic 11 (TUI Dashboard) — ✅ (v0.2.0)

**Critical path:** Epic 1 → Epic 2 → Epic 3 → Epic 5 (Story 5.1) → Epic 8 (Story 8.1 isolation test)

**Post-MVP (v0.3.0+):** See `epics-v03-analytics.md` for:
- Epic 12: Local Usage Analytics (stats-cache.json parsing)
- Epic 13: Profile-Aware HUD Integration
- Epic 14: Advanced Session Monitoring (transcripts)
- Epic 15: Native Rust HUD Plugin

---

## Definition of Done (All Stories)

- [ ] All acceptance criteria met
- [ ] Unit tests written and passing
- [ ] Integration test coverage where applicable
- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Documentation updated if needed
- [ ] No unwrap() or expect() in production code
- [ ] Error messages include suggestions
