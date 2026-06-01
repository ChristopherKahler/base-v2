# Project State

## Project Reference

See: .paul/PROJECT.md (updated 2026-06-01)

**Core value:** Proactive, deterministic context-injection engine — suppression, not detection. The gate that stays silent until the one thing that matters changes.
**Current focus:** Phase 6 in progress. Plan 06-01 complete (graph-backed rules). Ready for Plan 06-02.

## Current Position

Milestone: v0.1 Proactive Context Engine
Phase: 6 of 8 (CARL Absorption) — In Progress
Plan: 06-01 complete (PLAN → APPLY → UNIFY closed)
Status: Ready for next PLAN (06-02: DEVMODE + Context Brackets)
Last activity: 2026-06-01 12:55 — UNIFY closed loop on Plan 06-01

Progress:
- Milestone: [██████░░░░] 66%
- Phase 0: [██████████] 100% ✓
- Phase 1: [██████████] 100% ✓
- Phase 2: [██████████] 100% ✓
- Phase 3: [██████████] 100% ✓
- Phase 4: [██████████] 100% ✓
- Phase 5: [██████████] 100% ✓
- Phase 6: [███░░░░░░░] 25%

## Loop Position

Current loop state:
```
PLAN ──▶ APPLY ──▶ UNIFY
  ✓        ✓        ✓     [Loop complete — ready for next PLAN]
```

## Accumulated Context

### Decisions
24 decisions:
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
- CARL merges into BASE — one binary, not separate tools. CARL becomes a feature.
- `domains.toml` is trigger config only — two keyword types: prompt_keywords (user, natural language) and file_keywords (code-oriented).
- Rule content lives in the graph as entities — TOML is authoring surface, graph is query layer.
- Auto-sync domains to graph on hook invocation — mtime-gated, no manual sync required.
- Combined hash (rules + neighborhood) for session dedup of graph-backed injection.
- PSMM renamed to `base learn` — graph-backed structured memory system.
- PSMM deferred until after core rule injection + DEVMODE + context brackets.
- Directory-scoped `.base.toml` config files deferred — path globs in `domains.toml` suffice.
- Fallback to TOML rules when graph is empty — graceful degradation, never break injection.
- `ops:hasDecision` predicate added — domain → decision linkage in ontology.

### Deferred Issues
- AST/code extraction (open-ontologies integration) — deferred to future plan.

### Git State
Last commit: 6368a6b
Branch: main
Feature branches merged: none

### Blockers/Concerns
None.

## Session Continuity

Last session: 2026-06-01 12:55
Stopped at: Plan 06-01 loop closed — graph-backed rules shipped
Next action: /paul:plan for Plan 06-02 (DEVMODE + Context Brackets)
Resume file: .paul/phases/06-carl-absorption/06-01-SUMMARY.md

---
*STATE.md — Updated after every significant action*
