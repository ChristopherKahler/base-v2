---
phase: 05-signal-layer
plan: 01
subsystem: signals
tags: [rust, sparql, signals, suppression, active-awareness, pulse, staleness, budget-cap]

requires:
  - phase: 03-write-commands
    provides: CRUD entity data (projects, tasks, decisions, reminders)
  - phase: 04-extraction-layer
    provides: Extracted file data (Documents, PaulProjects)
provides:
  - Signal layer with 3 signal types (active-awareness, pulse, staleness)
  - Suppression layer (cross-session hash dedup, budget cap)
  - Session-start signals-first injection (ad-hoc queries as fallback)
  - SignalConfig in base.toml (max_chars, stale_days, active_days, enabled toggle)
affects: [carl-absorption]

key-decisions:
  - "Signal state persists across sessions (NOT cleared by session-start) — novelty is cross-session"
  - "Signals-first, queries-fallback in session-start — signals are primary injection source"
  - "Priority 1 signal (active-awareness) never dropped by budget cap"

duration: ~15min
started: 2026-06-01T11:23:00-05:00
completed: 2026-06-01T11:34:00-05:00
---

# Phase 5 Plan 01: Signal Layer Summary

**3 signal types (active-awareness, pulse, staleness) with cross-session suppression and budget cap — session-start now injects structured, prioritized, suppressed context instead of raw query dumps.**

## Performance

| Metric | Value |
|--------|-------|
| Duration | ~15 min |
| Tasks | 3 completed |
| Files created | 6 |
| Files modified | 3 |
| Tests | 84 pass (28 unit + 7 signal + 44 regression + 5 Phase 0) |

## Acceptance Criteria Results

| Criterion | Status | Notes |
|-----------|--------|-------|
| AC-1: Active-awareness surfaces recent entities | Pass | Groups by blocked/projects/tasks; excludes stale |
| AC-2: Pulse shows workspace health | Pass | Counts active/blocked/completed projects, open tasks, overdue reminders, recent decisions |
| AC-3: Staleness detects neglected entities | Pass | Configurable threshold, sorted by staleness |
| AC-4: Suppression prevents redundant injection | Pass | Hash-based dedup across sessions; re-emits on data change |
| AC-5: Budget cap limits output | Pass | Priority 1 always emits; lower priorities dropped with note |

## Deviations

- Used `cargo clippy --fix` for collapsible-if patterns (4 auto-fixes) — stylistic, no logic change

## Next Phase Readiness

**Ready:**
- The product thesis is now provable: structured signals with suppression replace v1's firehose
- Signal infrastructure extensible — add new signals as modules with a priority number
- Phase 6 (CARL Absorption) can port CARL's remaining mechanisms (DEVMODE, PSMM, dedup signatures) on top of this

**Blockers:** None.

---
*Phase: 05-signal-layer, Plan: 01*
*Completed: 2026-06-01*
