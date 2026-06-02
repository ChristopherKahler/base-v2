# Project State

## Project Reference

See: .paul/PROJECT.md (updated 2026-06-01)

**Core value:** Proactive, deterministic context-injection engine — suppression, not detection. The gate that stays silent until the one thing that matters changes.
**Current focus:** Phase 8 in progress. Dashboard: Graph Explorer + Operations (drag-drop kanban) + Session Activity (WebSocket) live. Usage Analytics next.

## Current Position

Milestone: v0.1 Proactive Context Engine
Phase: 8 of 8 (Command Center Dashboard) — In Progress
Plan: 08-05 created, awaiting approval
Status: PLAN created, ready for APPLY
Last activity: 2026-06-02 16:42 — Created Plan 08-05 (dashboard enhancements — 8 features)

Progress:
- Milestone: [█████████░] 99%
- Phase 0: [██████████] 100% ✓
- Phase 1: [██████████] 100% ✓
- Phase 2: [██████████] 100% ✓
- Phase 3: [██████████] 100% ✓
- Phase 4: [██████████] 100% ✓
- Phase 5: [██████████] 100% ✓
- Phase 6: [██████████] 100% ✓
- Phase 7: [██████████] 100% ✓
- Phase 8: [█████████░] 90%

## Loop Position

Current loop state:
```
PLAN ──▶ APPLY ──▶ UNIFY
  ✓        ○        ○     [Plan 08-05 created, awaiting approval]
```

## Accumulated Context

### Decisions
51 decisions (4 new this session):
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
- Session-grouped cards over flat event log — raw telemetry is noise
- File-tailing via line-count over byte-offset seeking — simpler, no edge cases
- HookEventData return type on user_prompt_submit — domain match data without refactoring all handlers
- Cache-Control: no-cache on index.html — prevents stale JS bundles

### Deferred Issues
- base install can't copy over itself when running binary IS target — pre-existing
- signal/mod.rs has 43 entities — complexity hotspot
- hook-events.jsonl grows unbounded — needs rotation/truncation
- Debug eprintln! in WS handler — remove in polish pass

### Git State
Last commit: dbd259a (Plan 08-03 + README update)
Branch: main

### Blockers/Concerns
- None blocking

## Session Continuity

Last session: 2026-06-02 16:41
Stopped at: Plan 08-04 complete (usage analytics, all 4 panels live)
Next action: /paul:apply .paul/phases/08-command-center-dashboard/08-05-PLAN.md
Resume file: .paul/phases/08-command-center-dashboard/08-05-PLAN.md

---
*STATE.md — Updated after every significant action*
