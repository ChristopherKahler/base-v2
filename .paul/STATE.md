# Project State

## Project Reference

See: .paul/PROJECT.md (updated 2026-06-01)

**Core value:** Proactive, deterministic context-injection engine — suppression, not detection. The gate that stays silent until the one thing that matters changes.
**Current focus:** Phase 5 complete. Ready for Phase 6 planning (CARL Absorption).

## Current Position

Milestone: v0.1 Proactive Context Engine
Phase: 5 of 8 (Signal Layer) — Complete ✓
Plan: 05-01 complete (PLAN → APPLY → UNIFY closed)
Status: Phase 5 done. Ready for Phase 6 PLAN.
Last activity: 2026-06-01 11:34 — UNIFY closed loop on Plan 05-01, phase transition complete

Progress:
- Milestone: [██████░░░░] 66%
- Phase 0: [██████████] 100% ✓
- Phase 1: [██████████] 100% ✓
- Phase 2: [██████████] 100% ✓
- Phase 3: [██████████] 100% ✓
- Phase 4: [██████████] 100% ✓
- Phase 5: [██████████] 100% ✓

## Loop Position

Current loop state:
```
PLAN ──▶ APPLY ──▶ UNIFY
  ○        ○        ○     [Loop closed. Ready for next PLAN]
```

## Accumulated Context

### Decisions
20 decisions:
- Rust single binary with embedded Oxigraph — CLI + hook handler, no MCP server.
- TTL-is-the-store — TriG text files, no separate database. Git-native.
- IRI-keyed idempotent extraction is the structural fix for v1 rot.
- Deterministic matching via `domains.toml` (keyword + path + sticky), no fuzzy.
- Hooks call CLI directly — `base hook <event>` reads stdin, writes stdout, fail-open.
- Namespace URI + prefix are runtime-configurable via `base.toml`.
- SPARQL queries are operator-configurable via `queries.toml` with tiered override.
- Session-end nudge dropped — hook unreliable.
- No dirty-file tracking — mtime self-healing.
- Session dedup via `.base/.session` with rules-hash.
- Signal state persists across sessions (NOT cleared by session-start) — novelty is cross-session.
- Signals-first, queries-fallback in session-start.
- Priority 1 signal (active-awareness) never dropped by budget cap.

### Deferred Issues
- AST/code extraction (open-ontologies integration) — deferred to future plan.

### Git State
Last commit: 6368a6b
Branch: main
Feature branches merged: none

### Blockers/Concerns
None.

## Session Continuity

Last session: 2026-06-01 11:34
Stopped at: Phase 5 UNIFY complete, transition done
Next action: Run /paul:plan for Phase 6 (CARL Absorption)
Resume file: .paul/phases/05-signal-layer/05-01-SUMMARY.md

---
*STATE.md — Updated after every significant action*
