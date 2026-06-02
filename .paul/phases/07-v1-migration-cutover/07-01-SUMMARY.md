---
phase: 07-v1-migration-cutover
plan: 01
subsystem: crud
tags: [milestone, slug-resolution, test-isolation, oxigraph, sparql]
requires:
  - phase: 06-carl-absorption
    provides: CRUD layer, graph store, CLI dispatch
provides:
  - 3-tier project hierarchy (Project → Milestone → Task)
  - Slug/name resolution for all CRUD commands
  - 117/117 test suite (0 failures)
affects: [v1-migration, project-management, AST-injection]
tech-stack:
  added: []
  patterns: [resolve_slug 3-step lookup, sync_domain_list test isolation]
key-files:
  created: [src/crud/milestone.rs]
  modified: [src/crud/mod.rs, src/crud/task.rs, src/cli.rs, src/domain/sync.rs]
key-decisions:
  - "Milestone slugs use project.milestone dot notation matching task convention"
  - "resolve_slug skips IRI construction for inputs with spaces (invalid IRI)"
  - "Tests call sync_domain_list directly to bypass global config leakage"
  - "Tasks link to both project AND milestone (dual edges for flexible querying)"
patterns-established:
  - "resolve_slug 3-step: exact slug → slugify → SPARQL name lookup"
  - "Test isolation via extracted core function + parsed test data (no env var hacks)"
duration: 25min
started: 2026-06-02T08:14:00-05:00
completed: 2026-06-02T08:37:00-05:00
---

# Phase 7 Plan 01: CRUD Infrastructure + Test Isolation Summary

**3-tier project hierarchy shipped (Project → Milestone → Task), slug/name resolution fixed across all CRUD commands, test suite cleaned to 117/117.**

## Performance

| Metric | Value |
|--------|-------|
| Duration | ~25 min |
| Started | 2026-06-02T08:14 CDT |
| Completed | 2026-06-02T08:37 CDT |
| Tasks | 8 completed |
| Files modified | 8 |

## Acceptance Criteria Results

| Criterion | Status | Notes |
|-----------|--------|-------|
| AC-1: Display name resolves to slug | Pass | `"Hunter Exotics"` → `hunter-exotics` via slugify step |
| AC-2: Milestone CRUD works | Pass | add/list/get/update all verified |
| AC-3: Task links to milestone | Pass | `--milestone` flag adds `hasMilestone` edge |
| AC-4: Task list filters by milestone | Pass | `--milestone X.Y` returns only milestone tasks |
| AC-5: 117/117 tests pass | Pass | 0 failures (was 4 failures before) |
| AC-6: Binary installed | Pass | `~/.local/bin/base` updated |

## Accomplishments

- **resolve_slug()** — 3-step resolution (exact slug → slugify → SPARQL name lookup) wired into project, entity, goal, milestone, and reminder commands. Loads graph once, runs all checks against it.
- **Milestone entity type** — Full CRUD with `belongsTo` project edge, `hasMilestone` reverse edge, `hasTask` grouping. Slug format: `{project}.{milestone}`.
- **Test isolation fixed** — Extracted `sync_domain_list()` so tests call it with parsed TOML directly, bypassing `load_domains()` which merges `~/.base-gbl/` global config. Domain CLI test fixed to assert on specific domain rather than total count.

## Files Created/Modified

| File | Change | Purpose |
|------|--------|---------|
| `src/crud/milestone.rs` | Created | Full CRUD: add, list, get, update for Milestone entity |
| `src/crud/mod.rs` | Modified | Added `resolve_slug()`, `capitalize_first()`, `pub mod milestone` |
| `src/crud/task.rs` | Modified | Added optional `milestone_slug` param to `add()` and `list()` |
| `src/cli.rs` | Modified | Added `Milestone` command, `MilestoneAction` enum, `resolve()` helper, wired resolution into all dispatch paths |
| `src/domain/sync.rs` | Modified | Extracted `sync_domain_list()` for test isolation, updated all 6 tests |
| `tests/crud_task_test.rs` | Modified | Updated 4 `task::add` call sites for new signature |
| `tests/signal_test.rs` | Modified | Updated 1 `task::add` call site |
| `tests/domain_cli_test.rs` | Modified | Fixed assertion to check specific domain, not total count |

## Decisions Made

| Decision | Rationale | Impact |
|----------|-----------|--------|
| Milestone slug = `project.milestone` | Matches existing task dot-notation convention | Consistent hierarchical naming |
| Tasks dual-linked to project AND milestone | `task list --project` shows all; `task list --milestone` shows grouped | Flexible querying without breaking existing project-level views |
| resolve_slug skips IRI for inputs with spaces | Spaces create invalid IRIs that crash SPARQL parser | Falls through cleanly to slugify step |
| Test isolation via function extraction, not env vars | Env var approach is racy in parallel tests | Tests are deterministic regardless of user's global config |

## Deviations from Plan

### Summary

| Type | Count | Impact |
|------|-------|--------|
| Auto-fixed | 1 | Milestone list showed `milestone/` prefix in slug column |
| Scope additions | 1 | domain_cli_test fix (4th pre-existing failure discovered) |

### Auto-fixed Issues

**1. Milestone list slug display**
- **Found during:** verification after milestone list
- **Issue:** `term_display` extracts after `#`, returning `milestone/slug` instead of `slug`
- **Fix:** `strip_prefix("milestone/")` in milestone list display
- **Verification:** `base milestone list` shows clean slug

## Next Phase Readiness

**Ready:**
- 3-tier hierarchy operational — milestones can structure Phase 7 work itself
- Slug resolution eliminates the slug-vs-name UX paper cut
- Test suite clean — no pre-existing failures to carry forward

**Next:**
- Phase 7 Plan 02: AST Context Injection (pre-tool-use hook queries graph on file touch)

---
*Phase: 07-v1-migration-cutover, Plan: 01*
*Completed: 2026-06-02*
