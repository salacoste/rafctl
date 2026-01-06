# rafctl — Backlog: Post-Review Improvements

**Source:** Party Mode Review (Epics 1-5)
**Date:** 2026-01-06
**Status:** Triaged

---

## Summary

После ревью реализованных эпиков 1-5 выявлены улучшения, которые необходимо запланировать. Ниже — структурированный backlog с распределением по командам и эпикам.

---

## Triage Matrix

| ID | Priority | Team | Target Epic | Effort | Description |
|----|----------|------|-------------|--------|-------------|
| IMP-1 | 🔴 High | Dev | New: Epic 9 | M | Verify tool CLI interfaces (claude/codex) |
| IMP-2 | 🟡 Medium | QA | Epic 8 | L | Integration tests for profile/auth/run |
| IMP-3 | 🟡 Medium | Dev | Epic 4 | S | Add 10-min timeout for auth login |
| IMP-4 | 🟢 Low | Dev | Epic 2 | S | Replace regex with simple validation |
| IMP-5 | 🟢 Low | Dev | Epic 8 | XS | Fix deprecated cargo_bin warning |
| IMP-6 | 🟢 Low | Dev | Epic 5 | XS | Log save_profile errors properly |

**Effort Scale:** XS (<1h), S (1-2h), M (2-4h), L (4-8h), XL (>8h)

---

## Decision: Epic Assignment

### Option A: Add to Existing Epics
- IMP-2, IMP-5 → Epic 8 (Quality Assurance)
- IMP-3 → Epic 4 (Authentication) — но Epic 4 уже "complete"
- IMP-4 → Epic 2 (Profile Management) — но Epic 2 уже "complete"
- IMP-6 → Epic 5 (Execution) — но Epic 5 уже "complete"

### Option B: Create New Epic for Polish (Recommended ✅)
Создать **Epic 9: Hardening & Polish** для всех post-review improvements.
Это позволяет:
- Не трогать "completed" эпики
- Группировать tech debt в одном месте
- Чётко отделить MVP от polish

**Decision:** Option B — Create Epic 9

---

## Epic 9: Hardening & Polish (NEW)

**Goal:** Address post-review findings, improve robustness and reduce tech debt.

**Owner:** Dev Team + QA
**Priority:** Should complete before v1.0 release

---

### Story 9.1: Verify Tool CLI Interfaces [DEV] 🔴

As a **developer**,
I want **to verify the actual CLI interfaces of Claude Code and Codex CLI**,
So that **auth login and run commands work correctly with real tools**.

**Acceptance Criteria:**

**Given** Claude Code is installed
**When** I check its CLI interface
**Then** I document the correct command for authentication
**And** update `handle_login` to use the correct command

**Given** Codex CLI is installed (or documented)
**When** I check its CLI interface
**Then** I document the correct command for authentication
**And** update `handle_login` to use the correct command

**Research Tasks:**
- [ ] Check Claude Code CLI: `claude --help`, `claude auth --help`
- [ ] Check if Claude uses `claude auth login` or another pattern
- [ ] Check Codex CLI documentation (may not be installed locally)
- [ ] Update `src/cli/auth.rs` if needed
- [ ] Update `src/tools/claude.rs` and `src/tools/codex.rs` constants if needed

**Technical Notes:**
- Claude Code may use browser-based auth without explicit `auth login` subcommand
- Codex CLI interface needs verification from OpenAI docs
- May need to spawn tool differently for each provider

**Effort:** M (2-4h)
**Risk:** 🔴 High — if wrong, auth won't work

---

### Story 9.2: Add Auth Login Timeout [DEV] 🟡

As a **user**,
I want **authentication to timeout after 10 minutes**,
So that **I don't have hanging processes if I forget to complete auth**.

**Acceptance Criteria:**

**Given** I run `rafctl auth login <profile>`
**When** 10 minutes pass without completing authentication
**Then** the process is terminated with timeout error
**And** message shows "Authentication timed out after 10 minutes"

**Given** I complete authentication before timeout
**When** credentials are saved
**Then** success message is shown as usual

**Technical Notes:**
```rust
// In src/cli/auth.rs, add timeout to Command spawn
use std::time::Duration;

// Option 1: Use wait_timeout (requires nix crate or manual impl)
// Option 2: Use tokio with timeout (adds async dependency)
// Option 3: Use std::thread::spawn with channel timeout

// Recommended: Simple thread-based timeout
let handle = std::thread::spawn(move || cmd.status());
match handle.join() {
    Ok(result) => result,
    Err(_) => // timeout handling
}
```

**Effort:** S (1-2h)
**Dependencies:** May need additional crate for clean timeout impl

---

### Story 9.3: Replace Regex with Simple Validation [DEV] 🟢

As a **developer**,
I want **to remove regex dependency for profile name validation**,
So that **binary size is reduced by ~200KB**.

**Acceptance Criteria:**

**Given** profile name validation is needed
**When** using simple char-based validation
**Then** same validation rules apply: `[a-zA-Z0-9_-]+`, max 64 chars
**And** regex crate is removed from Cargo.toml
**And** binary size is reduced

**Implementation:**
```rust
// Replace in src/core/profile.rs:
pub fn validate_profile_name(name: &str) -> Result<(), RafctlError> {
    if name.is_empty() || name.len() > MAX_PROFILE_NAME_LENGTH {
        return Err(RafctlError::InvalidProfileName(name.to_string()));
    }
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return Err(RafctlError::InvalidProfileName(name.to_string()));
    }
    Ok(())
}

// Remove from Cargo.toml:
// regex = "1"
```

**Effort:** S (1-2h)
**Verification:** Existing tests should still pass

---

### Story 9.4: Log Errors Instead of Ignoring [DEV] 🟢

As a **developer**,
I want **to log warnings when non-critical operations fail**,
So that **users are aware of issues without crashing the app**.

**Acceptance Criteria:**

**Given** `save_profile` or `set_last_used_profile` fails after successful run
**When** the error is non-critical
**Then** a warning is printed to stderr
**And** the main operation still succeeds

**Implementation:**
```rust
// In src/cli/run.rs, replace:
let _ = save_profile(&profile);
let _ = set_last_used_profile(&name_lower);

// With:
if let Err(e) = save_profile(&profile) {
    eprintln!("{} Failed to update profile: {}", "⚠".yellow(), e);
}
if let Err(e) = set_last_used_profile(&name_lower) {
    eprintln!("{} Failed to save last used profile: {}", "⚠".yellow(), e);
}
```

**Effort:** XS (<1h)

---

### Story 9.5: Fix Deprecated cargo_bin Warning [DEV] 🟢

As a **developer**,
I want **to fix deprecated API warnings in tests**,
So that **codebase is clean and future-proof**.

**Acceptance Criteria:**

**Given** integration tests use `Command::cargo_bin`
**When** running `cargo test`
**Then** no deprecation warnings are shown

**Implementation:**
```rust
// In tests/integration.rs, replace:
Command::cargo_bin("rafctl")

// With:
use assert_cmd::cargo::CommandCargoExt;
Command::cargo_bin!("rafctl")
// Or use the new API as per assert_cmd docs
```

**Effort:** XS (<1h)

---

### Story 9.6: Integration Tests for CLI Commands [QA] 🟡

As a **QA engineer**,
I want **integration tests for all implemented CLI commands**,
So that **regressions are caught automatically**.

**Acceptance Criteria:**

**Given** profile commands are implemented
**When** running integration tests
**Then** these scenarios are covered:
- `profile add` creates profile
- `profile add` fails on duplicate
- `profile add` fails on invalid name
- `profile list` shows profiles
- `profile show` displays details
- `profile show` fails on non-existent
- `profile remove` deletes profile

**Given** auth commands are implemented
**When** running integration tests
**Then** these scenarios are covered:
- `auth status` shows unauthenticated profiles
- `auth logout` on unauthenticated profile is not error

**Given** run command is implemented
**When** running integration tests
**Then** these scenarios are covered:
- `run` with no profiles shows helpful error
- `run` with unauthenticated profile shows error
- `run` with non-existent profile shows error

**Technical Notes:**
- Use `tempfile::TempDir` with `RAFCTL_CONFIG_DIR` env override
- Add config dir override support if not exists
- Tests should be isolated and not affect real `~/.rafctl`

**Effort:** L (4-8h)
**Blocked by:** Need to add `RAFCTL_CONFIG_DIR` env support for test isolation

---

## Team Assignment

### Dev Team Scope
| Story | Priority | Effort | Status |
|-------|----------|--------|--------|
| 9.1: Verify Tool CLI | 🔴 High | M | TODO |
| 9.2: Auth Timeout | 🟡 Medium | S | TODO |
| 9.3: Remove Regex | 🟢 Low | S | TODO |
| 9.4: Log Errors | 🟢 Low | XS | TODO |
| 9.5: Fix Deprecation | 🟢 Low | XS | TODO |

**Total Dev Effort:** ~6-10h

### QA Team Scope
| Story | Priority | Effort | Status |
|-------|----------|--------|--------|
| 9.6: Integration Tests | 🟡 Medium | L | TODO |

**Total QA Effort:** ~4-8h

**Blocker for QA:** Story 9.6 requires `RAFCTL_CONFIG_DIR` env override support. This should be added by Dev as Story 9.0 (prerequisite).

---

## Recommended Execution Order

1. **Story 9.1** — Verify Tool CLI (🔴 blocking for real usage)
2. **Story 9.0** (new) — Add `RAFCTL_CONFIG_DIR` env override (blocker for tests)
3. **Story 9.2** — Auth Timeout
4. **Story 9.6** — Integration Tests (QA, parallel with dev)
5. **Story 9.3** — Remove Regex
6. **Story 9.4** — Log Errors
7. **Story 9.5** — Fix Deprecation

---

## Definition of Done (Epic 9)

- [ ] All stories completed
- [ ] `cargo test` passes with no warnings
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Binary size verified (after regex removal)
- [ ] Manual testing of auth with real Claude Code
- [ ] All acceptance criteria verified

---

## Notes

- Epic 9 should be completed before v1.0 release
- Story 9.1 is critical — without it, auth may not work
- Story 9.6 (QA) can run in parallel with dev stories after 9.0
- Consider adding Story 9.0 for test isolation infrastructure
