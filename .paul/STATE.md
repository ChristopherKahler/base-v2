# Project State

## Project Reference

See: .paul/PROJECT.md (updated 2026-06-01)

**Core value:** Proactive, deterministic context-injection engine — suppression, not detection. The gate that stays silent until the one thing that matters changes.
**Current focus:** Phase 8 in progress. Dashboard server + Graph Explorer shipped. Rich markdown extraction + MOP hook + edge repair live.

## Current Position

Milestone: v0.1 Proactive Context Engine
Phase: 8 of 8 (Command Center Dashboard) — Planning
Plan: 08-03 created, awaiting approval
Status: PLAN created, ready for APPLY
Last activity: 2026-06-02 14:15 — Created Plan 08-03 (WebSocket session activity + drag-drop kanban)

Progress:
- Milestone: [█████████░] 98%
- Phase 0: [██████████] 100% ✓
- Phase 1: [██████████] 100% ✓
- Phase 2: [██████████] 100% ✓
- Phase 3: [██████████] 100% ✓
- Phase 4: [██████████] 100% ✓
- Phase 5: [██████████] 100% ✓
- Phase 6: [██████████] 100% ✓
- Phase 7: [██████████] 100% ✓
- Phase 8: [██████░░░░] 66%

## Loop Position

Current loop state:
```
PLAN ──▶ APPLY ──▶ UNIFY
  ✓        ○        ○     [Plan 08-03 created, awaiting approval]
```

## Accumulated Context

### Decisions
47 decisions (6 new this session):
- (prior 30 decisions carried forward)
- Milestone slugs use project.milestone dot notation matching task convention
- resolve_slug skips IRI construction for inputs with spaces (invalid IRI)
- Tests call sync_domain_list directly to bypass global config leakage
- Tasks dual-linked to project AND milestone for flexible querying
- Test isolation via function extraction, not env var hacks
- Hunter Exotics marked completed, CaseGate blocked by Haider, Chris AI Website pending refactor
- AST query uses CONTAINS for label matching — handles () suffix from tree-sitter
- Import query matches by IRI stem, not just rdfs:label
- Grep intercept is guidance only — does not block grep
- AST file map deduped per file per session via SessionState.ast_injected
- PostToolUse section context only fires on partial reads (offset+limit present)
- Svelte over vanilla JS for dashboard — component model from day 1
- Huly design system — dark canvas #090A0C, gradient depth over shadows
- Decision CRUD creates hasDecision edge to domain — was silently orphaning
- 0.0.0.0 bind for WSL2 — localhost forwarding unreliable
- Custom left-edge resize on detail panel — CSS resize only goes bottom-right
- OperatorNotes planned for Plan 02 — first dashboard write mutation

### Deferred Issues
- AST data needs re-extraction to reflect Plans 01-02 code changes
- base sync frontmatter extraction bug (errors on SUMMARY.md files) — pre-existing
- base install can't copy over itself when running binary IS target — pre-existing
- signal/mod.rs has 43 entities — complexity hotspot

### Git State
Last commit: 9330f67 (uncommitted: Plans 01+02 — milestone, slug resolution, test fixes, AST query, hook injection)
Branch: main

### Blockers/Concerns
- None blocking

## Session Continuity

Last session: 2026-06-02 14:15
Stopped at: Plan 08-03 created (WebSocket session activity + drag-drop kanban)
Next action: Review and approve plan, then run /paul:apply .paul/phases/08-command-center-dashboard/08-03-PLAN.md
Resume file: .paul/phases/08-command-center-dashboard/08-03-PLAN.md

---
*STATE.md — Updated after every significant action*
