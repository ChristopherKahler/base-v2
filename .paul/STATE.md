# Project State

## Project Reference

See: .paul/PROJECT.md (updated 2026-06-01)

**Core value:** Proactive, deterministic context-injection engine — suppression, not detection. The gate that stays silent until the one thing that matters changes.
**Current focus:** Binary is LIVE. Phase 6 substantially complete (3 PAUL plans + extensive post-plan work). CARL retired. Ready for Phase 7 or Phase 6 closure.

## Current Position

Milestone: v0.1 Proactive Context Engine
Phase: 6 of 8 (CARL Absorption) — Substantially Complete
Plan: 06-03 closed via PAUL + 12 post-plan commits (install, scaffold, rule CRUD, pre-tool-use, paul.toml, mandatory edges)
Status: Binary installed and live at ~/.local/bin/base. All 4 hooks wired. CARL retired.
Last activity: 2026-06-01 15:48 — Handoff after full system live testing

Progress:
- Milestone: [████████░░] 80%
- Phase 0: [██████████] 100% ✓
- Phase 1: [██████████] 100% ✓
- Phase 2: [██████████] 100% ✓
- Phase 3: [██████████] 100% ✓
- Phase 4: [██████████] 100% ✓
- Phase 5: [██████████] 100% ✓
- Phase 6: [█████████░] 95% (CARL retired, all features shipped, formal plan 06-04 skipped)

## Loop Position

Current loop state:
```
PLAN ──▶ APPLY ──▶ UNIFY
  ✓        ✓        ✓     [Plan 06-03 closed. Post-plan work committed directly.]
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
Last commit: 5a68997
Branch: main
Feature branches merged: none

### Blockers/Concerns
None.

## Session Continuity

Last session: 2026-06-01 15:48
Stopped at: System live — binary installed, all hooks wired, CARL retired, tested with real prompts
Next action: Close Phase 6 formally, OR update PAUL to emit paul.toml, OR start Phase 7 (v1 migration)
Resume file: .paul/HANDOFF-2026-06-01-session2.md

---
*STATE.md — Updated after every significant action*
