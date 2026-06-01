# Project State

## Project Reference

See: .paul/PROJECT.md (updated 2026-06-01)

**Core value:** Proactive, deterministic context-injection engine — suppression, not detection. The gate that stays silent until the one thing that matters changes.
**Current focus:** Phase 6 COMPLETE. Binary live, CARL retired, v1 hooks/MCP removed, full productization done. Ready for Phase 7 (v1 Migration).

## Current Position

Milestone: v0.1 Proactive Context Engine
Phase: 6 of 8 (CARL Absorption) — Complete ✓
Plan: 3 PAUL plans + 25 post-plan commits. Phase 6 fully shipped.
Status: Product live. All hooks wired. Operator profile, star commands, AST extraction, MOP, mandatory edges all shipped.
Last activity: 2026-06-01 17:00 — Final handoff after full productization

Progress:
- Milestone: [█████████░] 90%
- Phase 0: [██████████] 100% ✓
- Phase 1: [██████████] 100% ✓
- Phase 2: [██████████] 100% ✓
- Phase 3: [██████████] 100% ✓
- Phase 4: [██████████] 100% ✓
- Phase 5: [██████████] 100% ✓
- Phase 6: [██████████] 100% ✓

## Loop Position

Current loop state:
```
PLAN ──▶ APPLY ──▶ UNIFY
  ✓        ✓        ✓     [Phase 6 complete]
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
Last commit: 07c8cfc
Branch: main
Feature branches merged: none

### Blockers/Concerns
- base sync frontmatter extraction has a bug (errors on SUMMARY.md files)
- base install can't copy over itself when the running binary IS the target

## Session Continuity

Last session: 2026-06-01 17:00
Stopped at: Phase 6 complete. Full productization done. System live and tested.
Next action: Close Phase 6 transition → Phase 7 (v1 Migration), OR port PAUL to emit paul.toml, OR fix sync bug
Resume file: .paul/HANDOFF-2026-06-01-final.md

---
*STATE.md — Updated after every significant action*
