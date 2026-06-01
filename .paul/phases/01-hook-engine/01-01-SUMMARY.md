---
phase: 01-hook-engine
plan: 01
subsystem: hooks
tags: [rust, oxigraph, sparql, hooks, config, toml, claude-code]

requires:
  - phase: 00-rust-scaffold-ontology
    provides: CLI skeleton, store module (load_graph, load_graphs, query, write_back), ontology module (ops.ttl, load_vocabulary)
provides:
  - Hook dispatch engine with fail-open guarantee
  - Session-start handler (TriG discovery → SPARQL → filtered context injection)
  - Post-tool-use handler (lastActive SPARQL UPDATE → atomic write-back)
  - Runtime-configurable namespace via base.toml
  - Operator-configurable SPARQL queries via queries.toml with tiered override
affects: [domain-matcher, extraction-layer, signal-layer, carl-absorption]

tech-stack:
  added: [chrono 0.4, dirs 5]
  patterns: [fail-open hooks, tiered config override, namespace-aware vocabulary, SPARQL UPDATE for mutations]

key-files:
  created: [src/config.rs, src/hook/mod.rs, src/hook/session_start.rs, src/hook/post_tool_use.rs, src/queries.default.toml, docs/settings-hook-config.md]
  modified: [src/cli.rs, src/lib.rs, src/ontology/mod.rs, Cargo.toml, tests/ontology_test.rs]

key-decisions:
  - "Namespace URI + prefix runtime-configurable via base.toml"
  - "SPARQL queries operator-configurable via queries.toml with tiered override (embedded → global → workspace)"
  - "Session-end nudge dropped — unreliable hook, deferred to signal layer"
  - "No dirty-file tracking — graph self-manages via mtime vs ops:lastExtracted comparison"

patterns-established:
  - "Fail-open hook pattern: errors → stderr, exit 0, empty stdout"
  - "Tiered config: compiled defaults → ~/.base-gbl/ → {workspace}/.base/"
  - "{{prefix}} placeholder in SPARQL templates for namespace portability"
  - "Sub-prefix TriG authoring (gws:, proj:, etc.) — slash-free prefixed names"

duration: ~25min
started: 2026-06-01T08:46:00-05:00
completed: 2026-06-01T09:08:00-05:00
---

# Phase 1 Plan 01: Hook Engine Summary

**Hook dispatch engine with configurable namespace and SPARQL queries — session-start injects filtered context, post-tool-use updates lastActive timestamps, all fail-open.**

## Performance

| Metric | Value |
|--------|-------|
| Duration | ~25 min |
| Started | 2026-06-01T08:46:00-05:00 |
| Completed | 2026-06-01T09:08:00-05:00 |
| Tasks | 3 completed |
| Files created | 8 |
| Files modified | 5 |
| Tests | 21 pass (8 unit + 8 integration + 5 regression) |

## Acceptance Criteria Results

| Criterion | Status | Notes |
|-----------|--------|-------|
| AC-1: Hook dispatch routes events and fails open | Pass | Dispatch routes by event name; unknown events silent; all errors caught to stderr |
| AC-2: Namespace configurable via base.toml | Pass | BaseConfig::load with tiered fallback; new unit test confirms custom namespace |
| AC-3: Session-start emits filtered context from configurable queries | Pass | 4 integration tests: active projects, no TriG silent, malformed fail-open, custom namespace |
| AC-4: Post-tool-use updates lastActive atomically | Pass | SPARQL UPDATE + write_back; test confirms timestamp changes on path match, unchanged on no match |
| AC-5: Queries configurable with tiered override | Pass | Embedded defaults → global → workspace; merge by name; {{prefix}} replacement |

## Accomplishments

- Hook engine end-to-end: stdin JSON → parse → route → load graph → SPARQL → stdout/mutation → fail-open
- Full namespace portability: nothing hardcoded to `ops:` or Chris's setup — any operator can configure their own namespace
- Query extensibility: operators add/override SPARQL queries via `queries.toml` at global or workspace tier
- 3 default queries ship embedded: active-projects (table), blocked-items (list), recent-activity (list)
- Post-tool-use updates `ops:lastActive` via SPARQL UPDATE with path matching — graph self-tracks activity

## Files Created/Modified

| File | Change | Purpose |
|------|--------|---------|
| `src/config.rs` | Created | BaseConfig, NamespaceConfig, QueryDef, tiered config + query loading |
| `src/queries.default.toml` | Created | 3 embedded default SPARQL queries with {{prefix}} placeholders |
| `src/hook/mod.rs` | Created | Hook dispatch, fail-open wrapper, stdin JSON parsing |
| `src/hook/session_start.rs` | Created | TriG discovery, graph loading, query execution, result formatting |
| `src/hook/post_tool_use.rs` | Created | Event file path extraction, SPARQL UPDATE lastActive, atomic write-back |
| `tests/hook_session_start_test.rs` | Created | 4 integration tests for session-start handler |
| `tests/hook_post_tool_use_test.rs` | Created | 4 integration tests for post-tool-use handler |
| `docs/settings-hook-config.md` | Created | settings.json wiring documentation for operators |
| `src/cli.rs` | Modified | Wired hook dispatch replacing stub |
| `src/lib.rs` | Modified | Added pub mod config, pub mod hook |
| `src/ontology/mod.rs` | Modified | Namespace-aware load_vocabulary(store, &NamespaceConfig) |
| `Cargo.toml` | Modified | Added chrono 0.4, dirs 5 |
| `tests/ontology_test.rs` | Modified | Updated load_vocabulary calls for new signature |

## Decisions Made

| Decision | Rationale | Impact |
|----------|-----------|--------|
| Namespace URI + prefix runtime-configurable via base.toml | Product is for other operators, not just Chris — nothing hardcoded | Phase 0 ontology module refactored; all future phases use NamespaceConfig |
| SPARQL queries operator-configurable via queries.toml | Operators need control over what context gets injected | Session-start is a query runner, not hardcoded logic |
| Session-end nudge dropped entirely | Users close VS Code without graceful shutdown — hook unreliable | Reduces Phase 1 scope; nudge behavior deferred to signal layer (session-start of next session) |
| No dirty-file tracking (no dirty.txt) | Graph self-manages via mtime vs ops:lastExtracted — zero external state | Phase 4 extraction uses filesystem mtime, not sidecar files |
| {{prefix}} placeholder in SPARQL templates | Queries need to reference the configured namespace without knowing it at authoring time | All queries portable across namespace configurations |

## Deviations from Plan

### Summary

| Type | Count | Impact |
|------|-------|--------|
| Auto-fixed | 2 | Minimal — type conversion and TriG syntax |
| Scope additions | 0 | — |
| Deferred | 0 | — |

**Total impact:** Essential fixes only, no scope creep.

### Auto-fixed Issues

**1. TermRef type conversion in session_start.rs**
- **Found during:** Task 2 (Session-start handler)
- **Issue:** Oxigraph's `QuerySolution::get()` returns `&Term`, but `term_display` expected `TermRef<'_>`
- **Fix:** Added `.into()` conversion at call site
- **Verification:** `cargo test` pass

**2. TriG sub-prefix syntax in test fixtures**
- **Found during:** Task 2 (Session-start handler)
- **Issue:** Test TriG data used `ops:graph/ws/test` — slash invalid in TriG prefix local names (Phase 0 decision)
- **Fix:** Used sub-prefixes (`gws:`, `proj:`) matching Phase 0's established pattern
- **Verification:** `cargo test` pass

## Issues Encountered

None beyond the auto-fixed deviations above.

## Next Phase Readiness

**Ready:**
- Hook plumbing is complete — any new hook event type is a new match arm + handler
- Config layer supports any future config sections (just add fields to BaseConfig)
- Query system is extensible — operators or future phases add queries without code changes
- Post-tool-use's lastActive tracking provides the activity data Phase 5 (Signal Layer) needs

**Concerns:**
- Post-tool-use only matches entities that have `ops:path` set — entities without path won't get lastActive updates
- No data exists in the graph yet (no CRUD commands, no extraction) — hooks work but produce empty output until Phase 3/4 populate the graph

**Blockers:**
None.

---
*Phase: 01-hook-engine, Plan: 01*
*Completed: 2026-06-01*
