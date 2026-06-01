---
phase: 03-write-commands
plan: 01
subsystem: crud
tags: [rust, sparql-update, oxigraph, cli, crud, projects, tasks]

requires:
  - phase: 01-hook-engine
    provides: Store module (load_graph, query, write_back), config layer (BaseConfig, NamespaceConfig, find_workspace_base)
provides:
  - CRUD infrastructure (build_iri, slugify, prefixes, field_update, load_and_mutate, load_and_query)
  - Project CRUD (add/list/get/update) via SPARQL INSERT/UPDATE
  - Task CRUD (add/list/done) with project linkage via hasTask
  - Workspace graph bootstrap (creates .base/graph.trig on first mutation)
affects: [extraction-layer, signal-layer, carl-absorption, v1-migration]

tech-stack:
  added: []
  patterns: [SPARQL INSERT DATA for entity creation, DELETE+INSERT WHERE for field updates, workspace graph auto-bootstrap]

key-files:
  created: [src/crud/mod.rs, src/crud/project.rs, src/crud/task.rs]
  modified: [src/cli.rs, src/lib.rs]

key-decisions:
  - "SPARQL INSERT DATA for creation, DELETE+INSERT WHERE for updates — no ORM, pure SPARQL"
  - "Workspace graph auto-creates on first mutation — empty-state bootstrap"
  - "field_update helper with OPTIONAL pattern handles both new fields and updates"

patterns-established:
  - "CRUD function signature: (cwd, ns, ...entity_args) → Result"
  - "load_and_mutate pipeline: find_or_create_base → load/create store → update → write_back"
  - "Slug-based IRI generation: {ns.uri}{type}/{slug}"
  - "field_update with OPTIONAL WHERE for idempotent upsert"

duration: ~15min
started: 2026-06-01T09:54:00-05:00
completed: 2026-06-01T10:08:00-05:00
---

# Phase 3 Plan 01: CRUD Infrastructure + Projects + Tasks Summary

**CRUD infrastructure with SPARQL mutation helpers, project add/list/get/update, and task add/list/done with project linkage — all namespace-configurable, graph auto-bootstrapping.**

## Performance

| Metric | Value |
|--------|-------|
| Duration | ~15 min |
| Started | 2026-06-01T09:54:00-05:00 |
| Completed | 2026-06-01T10:08:00-05:00 |
| Tasks | 3 completed |
| Files created | 5 |
| Files modified | 2 |
| Tests | 56 pass (23 unit + 9 CRUD integration + 24 regression) |

## Acceptance Criteria Results

| Criterion | Status | Notes |
|-----------|--------|-------|
| AC-1: SPARQL mutation helpers work correctly | Pass | INSERT DATA creates entities, DELETE+INSERT updates fields, write_back persists atomically |
| AC-2: IRI generation follows ontology scheme | Pass | build_iri + slugify produce {ns.uri}{type}/{slug}; unit tests verify |
| AC-3: Project CRUD end-to-end | Pass | 5 integration tests: add, add with path, list, get, update status + verify old removed |
| AC-4: Task CRUD end-to-end | Pass | 4 integration tests: add linked to project, list by project, done, default priority |
| AC-5: Supersession preserves audit trail | Pass | Updates set updatedAt + lastActive timestamps; old values replaced |

## Accomplishments

- CRUD infrastructure reusable for all entity types: build_iri, slugify, prefixes, field_update, load_and_mutate, load_and_query, term_display
- Project lifecycle: add → list → get → update with status/blockedBy/nextAction fields
- Task management: add with project linkage (hasTask edge), list filtered by project, mark done
- Workspace graph auto-bootstrap: first `base project add` creates .base/graph.trig automatically
- All SPARQL uses configured namespace prefix — zero hardcoded `ops:`

## Files Created/Modified

| File | Change | Purpose |
|------|--------|---------|
| `src/crud/mod.rs` | Created | Infrastructure: IRI builder, slugify, SPARQL helpers, graph pipeline, display |
| `src/crud/project.rs` | Created | Project add/list/get/update |
| `src/crud/task.rs` | Created | Task add/list/done with project linkage |
| `tests/crud_project_test.rs` | Created | 5 integration tests |
| `tests/crud_task_test.rs` | Created | 4 integration tests |
| `src/cli.rs` | Modified | ProjectAction + TaskAction replacing stubs |
| `src/lib.rs` | Modified | Added pub mod crud |

## Deviations from Plan

None — executed as planned.

## Next Phase Readiness

**Ready:**
- CRUD infrastructure established — Plan 03-02 replicates the pattern for decisions, entities, goals, reminders
- field_update helper handles both creation and update cases (OPTIONAL WHERE)
- Graph auto-bootstrap means operators can start using commands immediately

**Concerns:**
- No validation that a project exists before adding a task to it — task creation succeeds even with a non-existent project slug (the hasTask edge just points to a non-existent IRI). Plan 03-02 or a later phase could add validation.

**Blockers:**
None.

---
*Phase: 03-write-commands, Plan: 01*
*Completed: 2026-06-01*
