# Project State

## Project Reference

See: .paul/PROJECT.md (updated 2026-06-01)

**Core value:** Proactive, deterministic context-injection engine — suppression, not detection. The gate that stays silent until the one thing that matters changes.
**Current focus:** Phase 4 complete. Ready for Phase 5 planning (Signal Layer).

## Current Position

Milestone: v0.1 Proactive Context Engine
Phase: 4 of 8 (Extraction Layer) — Complete ✓
Plan: 04-01 complete (PLAN → APPLY → UNIFY closed)
Status: Phase 4 done. Ready for Phase 5 PLAN.
Last activity: 2026-06-01 10:56 — UNIFY closed loop on Plan 04-01, phase transition complete

Progress:
- Milestone: [█████░░░░░] 55%
- Phase 0: [██████████] 100% ✓
- Phase 1: [██████████] 100% ✓
- Phase 2: [██████████] 100% ✓
- Phase 3: [██████████] 100% ✓
- Phase 4: [██████████] 100% ✓

## Loop Position

Current loop state:
```
PLAN ──▶ APPLY ──▶ UNIFY
  ○        ○        ○     [Loop closed. Ready for next PLAN]
```

## Accumulated Context

### Decisions
19 decisions (12 design + 1 Phase 0 + 4 Phase 1 + 2 Phase 2):
- Rust single binary with embedded Oxigraph — CLI + hook handler, no MCP server.
- TTL-is-the-store — TriG text files, no separate database. Git-native.
- IRI-keyed idempotent extraction is the structural fix for v1 rot.
- Deterministic matching via `domains.toml` (keyword + path + sticky), no fuzzy.
- Hooks call CLI directly — `base hook <event>` reads stdin, writes stdout, fail-open.
- Sub-prefixes for TriG authoring (`proj:`, `ws:`, `g:`) — `/` invalid in prefix local names.
- Library + binary crate split (lib.rs for public modules, main.rs for CLI).
- anyhow for error handling at this stage.
- Namespace URI + prefix are runtime-configurable via `base.toml` — nothing hardcoded to Chris's setup.
- SPARQL queries are operator-configurable via `queries.toml` with tiered override (embedded defaults → global → workspace).
- Session-end nudge dropped — hook unreliable (users close VS Code). Nudge behavior deferred to signal layer at session-start.
- No dirty-file tracking — graph self-manages via `mtime` vs `ops:lastExtracted` comparison. The graph's own metadata IS the state tracker.
- Session dedup via `.base/.session` with rules-hash — bounded, self-cleaning at session-start, detects rule changes.
- domains.toml TOML format with `[[domain]]` tables — clean, operator-readable, same tiered-override pattern.

### Deferred Issues
- AST/code extraction (open-ontologies integration) — discussed, deferred to future plan. Extraction infrastructure supports adding new extractor modules.

### Git State
Last commit: 80b2c43
Branch: main
Feature branches merged: none

### Blockers/Concerns
None.

## Session Continuity

Last session: 2026-06-01 10:56
Stopped at: Phase 4 UNIFY complete, transition done
Next action: Run /paul:plan for Phase 5 (Signal Layer)
Resume file: .paul/phases/04-extraction-layer/04-01-SUMMARY.md

---
*STATE.md — Updated after every significant action*
