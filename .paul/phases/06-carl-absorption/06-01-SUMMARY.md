---
phase: 06-carl-absorption
plan: 01
subsystem: domain
tags: [sparql, oxigraph, domain-matching, graph-injection, carl-absorption]

requires:
  - phase: 02-domain-matcher
    provides: DomainDef struct, match_domains(), session dedup
  - phase: 04-extraction-layer
    provides: File-to-graph sync pattern, store.rs load/query/write_back
provides:
  - Graph-backed domain rule injection (SPARQL replaces struct read)
  - Dual keyword schema (prompt_keywords + file_keywords)
  - Domain/Rule/Decision entities synced to graph from domains.toml + carl.json
  - Auto-sync on user-prompt-submit (mtime-gated)
  - 1-hop neighborhood context in injection output
affects: [06-02-devmode, 06-03-base-learn, 06-04-carl-retirement]

tech-stack:
  added: []
  patterns: [graph-backed-injection, auto-sync-on-hook, toml-as-authoring-graph-as-query]

key-files:
  created:
    - src/domain/sync.rs
    - tests/graph_injection_test.rs
  modified:
    - src/domain/mod.rs
    - src/domain/matcher.rs
    - src/hook/user_prompt_submit.rs
    - src/ontology/ops.ttl
    - src/cli.rs
    - tests/domain_cli_test.rs

key-decisions:
  - "TOML stays as authoring surface, graph is the query layer — two-layer architecture"
  - "Auto-sync on hook invocation (mtime-gated) rather than requiring manual `base domain sync`"
  - "Fallback to TOML rules when graph is empty — graceful degradation"
  - "Combined hash (rules + neighborhood) for session dedup"

patterns-established:
  - "Graph-backed injection: match via TOML triggers, fetch via SPARQL"
  - "Auto-sync: check source mtime vs marker file, sync if stale"
  - "Neighborhood context: 1-hop traversal from matched domain IRI"

duration: ~25min
started: 2026-06-01T12:44:00-05:00
completed: 2026-06-01T12:55:00-05:00
---

# Phase 6 Plan 01: Graph-backed Domain Rules Summary

**Rules moved from static TOML text to graph entities with relational neighborhood injection — dual keyword schema (prompt/file), auto-sync on hook, SPARQL-backed retrieval with TOML fallback.**

## Performance

| Metric | Value |
|--------|-------|
| Duration | ~25 min |
| Started | 2026-06-01T12:44 |
| Completed | 2026-06-01T12:55 |
| Tasks | 3 completed |
| Files modified | 8 |
| Tests | 89 passing (+5 new) |

## Acceptance Criteria Results

| Criterion | Status | Notes |
|-----------|--------|-------|
| AC-1: Dual keyword parsing with backward compat | Pass | `prompt_keywords` + `file_keywords`, legacy `keywords` alias works |
| AC-2: Domain and Rule entities in graph | Pass | `base domain sync` creates ops:Domain + ops:Rule with edges |
| AC-3: Decision entities from carl.json | Pass | `--carl` flag migrates decisions with domain edges |
| AC-4: Graph-backed injection with neighborhood | Pass | SPARQL fetches rules + 1-hop decisions/projects, fallback works |
| AC-5: All tests pass | Pass | 89 tests, 0 failures, clippy clean, release build clean |

## Accomplishments

- **Dual keyword schema**: `DomainDef` now has `prompt_keywords` (user-configured, natural language) and `file_keywords` (code-oriented, system-suggestable) with backward compat via serde alias
- **Domain-to-graph sync**: New `src/domain/sync.rs` creates ops:Domain + ops:Rule entities from domains.toml, migrates decisions from carl.json — idempotent, atomic write-back
- **Graph-backed injection**: `user_prompt_submit` queries SPARQL for matched domain's rules + 1-hop neighborhood (decisions, projects), with auto-sync and TOML fallback
- **New ontology predicates**: `ops:promptKeyword`, `ops:fileKeyword`, `ops:hasDecision` added to ops.ttl

## Files Created/Modified

| File | Change | Purpose |
|------|--------|---------|
| `src/domain/sync.rs` | Created | Domain/Rule/Decision → graph sync engine |
| `tests/graph_injection_test.rs` | Created | 4 integration tests for graph injection path |
| `src/domain/mod.rs` | Modified | DomainDef: prompt_keywords + file_keywords, updated display/mutation |
| `src/domain/matcher.rs` | Modified | Uses prompt_keywords for matching |
| `src/hook/user_prompt_submit.rs` | Modified | Graph-backed injection, auto-sync, fallback, neighborhood context |
| `src/ontology/ops.ttl` | Modified | Added promptKeyword, fileKeyword, hasDecision predicates |
| `src/cli.rs` | Modified | Added `base domain sync --carl` command |
| `tests/domain_cli_test.rs` | Modified | Updated for prompt_keywords field name |

## Decisions Made

| Decision | Rationale | Impact |
|----------|-----------|--------|
| TOML as authoring surface, graph as query layer | Operator edits TOML (familiar), graph handles relational queries | Two-layer architecture — no new config file needed |
| Auto-sync on hook invocation | Avoids requiring manual `base domain sync` before rules work | Adds ~1-5ms on first prompt if domains.toml changed |
| Combined hash for dedup | Rules + neighborhood must be considered together for change detection | Session dedup works correctly with graph-sourced content |
| Fallback to TOML when graph empty | Never break injection if sync hasn't run or graph is missing | Graceful degradation, fail-open preserved |

## Deviations from Plan

None — plan executed as specified.

## Issues Encountered

None.

## Next Phase Readiness

**Ready:**
- Graph-backed rules are the foundation for all remaining Phase 6 plans
- DEVMODE (06-02) can query the graph for what was matched/injected
- Context brackets (06-02) can vary injection depth based on graph content
- `base learn` (06-03) can create entities that link to domains/rules
- CARL retirement (06-04) has decision data already in the graph

**Concerns:**
- None

**Blockers:**
- None

---
*Phase: 06-carl-absorption, Plan: 01*
*Completed: 2026-06-01*
