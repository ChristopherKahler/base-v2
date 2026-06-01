---
phase: 06-carl-absorption
plan: 02
subsystem: hook-output
tags: [devmode, context-brackets, session-state, observability]

requires:
  - phase: 06-carl-absorption/01
    provides: Graph-backed rule injection, user_prompt_submit hook
provides:
  - Context brackets (FRESH/MODERATE/DEPLETED/CRITICAL) with configurable thresholds
  - DEVMODE telemetry block with loaded/available domain listing
  - Bracket-aware injection (lean mode, force-refresh on dedup)
  - Prompt count tracking in session state
affects: [06-03-base-learn, 06-04-carl-retirement]

tech-stack:
  added: []
  patterns: [bracket-state-machine, devmode-telemetry, lean-injection-mode]

key-files:
  created:
    - tests/devmode_test.rs
  modified:
    - src/config.rs
    - src/domain/session.rs
    - src/hook/user_prompt_submit.rs

key-decisions:
  - "Prompt count as bracket proxy — no direct token counting (Claude Code doesn't expose it in events)"
  - "DEVMODE instruction-only — tells Claude what to report, doesn't verify compliance"
  - "Lean mode on FRESH (first 2 prompts): rules only, skip neighborhood"
  - "Force-refresh dedup every Nth prompt in DEPLETED/CRITICAL"

patterns-established:
  - "Bracket state machine: derive from prompt_count + configurable thresholds"
  - "DEVMODE as output appendix: always-emitted when enabled, not subject to dedup"

duration: ~15min
started: 2026-06-01T13:07:00-05:00
completed: 2026-06-01T13:17:00-05:00
---

# Phase 6 Plan 02: DEVMODE + Context Brackets Summary

**Context brackets track session depth via prompt count (FRESH→MODERATE→DEPLETED→CRITICAL), DEVMODE emits structured telemetry showing loaded domains, bracket state, and dedup status.**

## Performance

| Metric | Value |
|--------|-------|
| Duration | ~15 min |
| Started | 2026-06-01T13:07 |
| Completed | 2026-06-01T13:17 |
| Tasks | 2 completed |
| Files modified | 4 |
| Tests | 101 passing (+12 new) |

## Acceptance Criteria Results

| Criterion | Status | Notes |
|-----------|--------|-------|
| AC-1: Brackets from prompt count | Pass | FRESH≤3, MODERATE≤10, DEPLETED≤20, CRITICAL>20 |
| AC-2: Bracket affects injection | Pass | Lean mode on FRESH, force-refresh on DEPLETED interval |
| AC-3: DEVMODE block when enabled | Pass | Lists loaded/available domains, bracket, dedup count |
| AC-4: All tests pass | Pass | 101 tests, clippy clean, release clean |

## Accomplishments

- **BracketConfig + DevmodeConfig** added to BaseConfig (base.toml sections)
- **Bracket enum** with Display, state derivation from prompt_count, force-refresh logic
- **Lean mode**: FRESH sessions (prompts 1-2) inject rules only, skip neighborhood context
- **Force-refresh**: DEPLETED/CRITICAL sessions clear dedup every Nth prompt
- **DEVMODE block**: lists loaded domains (with match reason), available unmatched domains, bracket state, dedup count
- **prompt_count** persists in session JSON across prompts

## Files Created/Modified

| File | Change | Purpose |
|------|--------|---------|
| `tests/devmode_test.rs` | Created | 6 integration tests for DEVMODE + bracket behavior |
| `src/config.rs` | Modified | Added BracketConfig + DevmodeConfig structs |
| `src/domain/session.rs` | Modified | prompt_count, Bracket enum, force-refresh, clear_dedup |
| `src/hook/user_prompt_submit.rs` | Modified | Bracket-aware injection + DEVMODE output |

## Deviations from Plan

None.

## Next Phase Readiness

**Ready:**
- `base learn` (Plan 06-03) can build on session state and graph infrastructure
- CARL retirement (Plan 06-04) now has DEVMODE + brackets ported — two of CARL's major features

**Concerns:**
- None

**Blockers:**
- None

---
*Phase: 06-carl-absorption, Plan: 02*
*Completed: 2026-06-01*
