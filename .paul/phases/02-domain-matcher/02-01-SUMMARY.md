---
phase: 02-domain-matcher
plan: 01
subsystem: domain-matching
tags: [rust, toml, domain-matching, dedup, session-state, cli]

requires:
  - phase: 01-hook-engine
    provides: Hook dispatch (fail-open), config layer (BaseConfig, tiered loading), session-start/post-tool-use handlers
provides:
  - Deterministic domain matching via domains.toml
  - Multi-signal matcher (keywords, paths, always-on, exclude, sticky)
  - user-prompt-submit hook handler with rule injection
  - Session-level dedup via rules-hash change detection
  - base domain add-trigger/list/get CLI commands
  - find_workspace_base shared utility
affects: [carl-absorption, signal-layer]

tech-stack:
  added: []
  patterns: [session state file (.base/.session), domains.toml format, multi-signal matching, rules-hash dedup]

key-files:
  created: [src/domain/mod.rs, src/domain/matcher.rs, src/domain/session.rs, src/hook/user_prompt_submit.rs]
  modified: [src/config.rs, src/hook/mod.rs, src/hook/session_start.rs, src/cli.rs, src/lib.rs]

key-decisions:
  - "Session dedup via .base/.session file with rules-hash — bounded, self-cleaning at session-start"
  - "domains.toml format with [[domain]] array tables — clean, operator-readable"
  - "Matcher moved to Task 1 (natural fit with data model) — plan said Task 2"

patterns-established:
  - "Session state pattern: .base/.session cleared by session-start, updated by user-prompt-submit"
  - "find_workspace_base() shared utility in config.rs for walking up to .base/ directory"
  - "Domain matching: always-on + keyword + path + exclude + sticky signals"
  - "Dedup by domain name + rules hash — changed rules trigger re-injection"

duration: ~15min
started: 2026-06-01T09:30:00-05:00
completed: 2026-06-01T09:40:00-05:00
---

# Phase 2 Plan 01: Domain Matcher + Rule Injection Summary

**Deterministic domain matching via domains.toml with multi-signal matcher, user-prompt-submit rule injection, session dedup, and CLI domain management.**

## Performance

| Metric | Value |
|--------|-------|
| Duration | ~15 min |
| Started | 2026-06-01T09:30:00-05:00 |
| Completed | 2026-06-01T09:40:00-05:00 |
| Tasks | 3 completed |
| Files created | 6 |
| Files modified | 5 |
| Tests | 44 pass (20 unit + 11 integration + 4+4+5 regression) |

## Acceptance Criteria Results

| Criterion | Status | Notes |
|-----------|--------|-------|
| AC-1: domains.toml parsed with tiered loading | Pass | Global → workspace merge by name; empty Vec on missing files |
| AC-2: Multi-signal matcher identifies domains | Pass | 8 unit tests: keyword, path, always-on, exclude, case-insensitive, dedup, re-inject on change |
| AC-3: user-prompt-submit emits matched rules | Pass | 6 integration tests: matched rules, silent without domains, dedup across calls, empty prompt, missing prompt, malformed TOML |
| AC-4: Dedup prevents re-injection | Pass | Rules-hash based; same hash = skip, different hash = re-inject; session-start clears state |
| AC-5: base domain add-trigger mutates domains.toml | Pass | 5 CLI tests: add keyword, create domain, add path, no duplicates, create .base/ if missing |

## Accomplishments

- Full domain matching pipeline: domains.toml → parse → match → inject → dedup — replaces CARL's Python matching
- Multi-signal matcher with 5 signal types: always-on, keyword, path, exclude (veto), sticky
- Session dedup via `.base/.session` with rules-hash change detection — bounded and self-cleaning
- Graph-integrated path matching: queries `ops:lastActive` entities for recently-active file paths
- CLI domain management: add-trigger, list, get — operators manage domains from the command line
- Shared `find_workspace_base()` utility factored out for cross-module workspace discovery

## Files Created/Modified

| File | Change | Purpose |
|------|--------|---------|
| `src/domain/mod.rs` | Created | DomainDef struct, load_domains (tiered), add_trigger, list/get |
| `src/domain/matcher.rs` | Created | match_domains with 5 signal types + dedup |
| `src/domain/session.rs` | Created | SessionState (load/save/clear), rules_hash |
| `src/hook/user_prompt_submit.rs` | Created | Full handler: prompt → match → inject → dedup |
| `tests/hook_user_prompt_submit_test.rs` | Created | 6 integration tests |
| `tests/domain_cli_test.rs` | Created | 5 CLI mutation tests |
| `src/config.rs` | Modified | Added find_workspace_base() utility |
| `src/hook/mod.rs` | Modified | Added user-prompt-submit route |
| `src/hook/session_start.rs` | Modified | Added session state clear at start |
| `src/cli.rs` | Modified | Implemented DomainAction (add-trigger, list, get) |
| `src/lib.rs` | Modified | Added pub mod domain |

## Decisions Made

| Decision | Rationale | Impact |
|----------|-----------|--------|
| Session dedup via .base/.session with rules-hash | Bounded file, self-cleaning at session-start, detects rule changes | Clean dedup without unbounded state growth |
| Matcher moved from Task 2 to Task 1 | Natural fit alongside data model — testing matcher without handler | Plan deviation but cleaner execution |
| find_workspace_base() in config.rs | Shared utility for session.rs, session_start.rs, post_tool_use.rs | Reduces code duplication across modules |

## Deviations from Plan

### Summary

| Type | Count | Impact |
|------|-------|--------|
| Task reorder | 1 | Matcher logic moved to Task 1 (better cohesion) |
| Scope additions | 0 | — |
| Deferred | 0 | — |

**Total impact:** Minimal — task reorder improved code organization.

## Next Phase Readiness

**Ready:**
- Hook pipeline complete: session-start (context injection), user-prompt-submit (rule injection), post-tool-use (activity tracking)
- Domain matching provides the rule-injection layer Phase 6 (CARL Absorption) builds on
- Session state pattern available for future per-session tracking needs

**Concerns:**
- Sticky domains currently rely on dedup state (once injected = "sticky") — true sticky behavior (matching on subsequent prompts regardless of keywords) needs session state to store activated-domain names separately from injected-domain hashes
- Graph-based path matching only works when entities have ops:path set (requires Phase 3/4 data)

**Blockers:**
None.

---
*Phase: 02-domain-matcher, Plan: 01*
*Completed: 2026-06-01*
