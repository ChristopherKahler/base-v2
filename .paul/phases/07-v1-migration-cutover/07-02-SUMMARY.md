---
phase: 07-v1-migration-cutover
plan: 02
subsystem: hooks, cli
tags: [ast, sparql, pre-tool-use, post-tool-use, grep-intercept, code-navigation]
requires:
  - phase: 07-v1-migration-cutover
    plan: 01
    provides: CRUD infrastructure, graph store, milestone entity
provides:
  - base ast query CLI command (4 modes: --contains, --file, --calls, --imports)
  - PreToolUse file map injection on Read (source files, session dedup)
  - PreToolUse grep/find/rg intercept with ast-hint redirect
  - PostToolUse section-specific AST entity context for partial reads
affects: [all-code-navigation, explore-agents, subagent-workflows]
tech-stack:
  added: []
  patterns: [SPARQL AST queries against code: namespace, grep intercept via tool_name detection, session dedup for AST injection]
key-files:
  created: [src/crud/ast_query.rs]
  modified: [src/crud/mod.rs, src/cli.rs, src/hook/pre_tool_use.rs, src/hook/post_tool_use.rs, src/domain/session.rs]
key-decisions:
  - "AST query uses CONTAINS for label matching — handles () suffix in tree-sitter labels"
  - "Import query matches by IRI stem, not just rdfs:label — import edges target module IRIs not file labels"
  - "Grep intercept is guidance only — does not block the grep command"
  - "AST file map injected once per file per session via SessionState.ast_injected set"
  - "PostToolUse section context only fires on partial reads (offset+limit present)"
patterns-established:
  - "ast_query module exposes both CLI functions and compact/section helpers for hook consumption"
  - "SessionState extended with ast_injected HashSet for cross-hook dedup"
  - "PreToolUse handles three concerns: domain rules, AST file map, grep intercept — additive, not exclusive"
duration: 12min
started: 2026-06-02T09:05:00-05:00
completed: 2026-06-02T09:16:00-05:00
---

# Phase 7 Plan 02: AST Context Injection Summary

**AST graph wired into code navigation: explicit query command replaces grep, automatic file map on Read, section context on partial reads, grep intercept redirects to graph.**

## Performance

| Metric | Value |
|--------|-------|
| Duration | ~12 min |
| Started | 2026-06-02T09:05 CDT |
| Completed | 2026-06-02T09:16 CDT |
| Tasks | 3 completed |
| Files modified | 6 |

## Acceptance Criteria Results

| Criterion | Status | Notes |
|-----------|--------|-------|
| AC-1: AST query --contains returns file/line/type | Pass | `--contains "slugify"` → `signal/mod.rs:48 fn slugify()` |
| AC-2: File map injection on Read | Pass | PreToolUse injects compact entity map for source files, session dedup via ast_injected |
| AC-3: Section context on partial Read | Pass | PostToolUse extracts offset/limit, queries entities in line range |
| AC-4: Grep/find intercept | Pass | Bash commands with grep/rg/find get ast-hint injection |
| AC-5: 117+ tests pass | Pass | 117/117, 0 failures, 0 warnings |

## Accomplishments

- **`base ast query` command** — 4 modes: `--contains` (entity search by name), `--file` (full file entity map with imports/importers), `--calls` (find callers of entity), `--imports` (find files importing from a module). All backed by SPARQL against the AST graph.
- **PreToolUse file map injection** — When reading a source file (.rs, .py, .js, .ts, .go, etc.), hook queries AST graph and injects compact entity summary: count, key entities with line numbers, imports, imported-by. Deduped per file per session via `SessionState.ast_injected`.
- **PreToolUse grep intercept** — Detects grep/rg/find patterns in Bash commands, extracts search term, injects `<ast-hint>` suggesting `base ast query --contains` instead. Guidance only — does not block.
- **PostToolUse section context** — On partial reads (offset + limit present), queries entities within the read line range and injects their call/caller relationships. Full reads skip (PreToolUse already covered).

## Files Created/Modified

| File | Change | Purpose |
|------|--------|---------|
| `src/crud/ast_query.rs` | Created | AST graph query module — contains, file, calls, imports + compact/section helpers for hooks |
| `src/crud/mod.rs` | Modified | Added `pub mod ast_query` |
| `src/cli.rs` | Modified | Added `Ast` command with `AstAction::Query` subcommand and dispatch |
| `src/hook/pre_tool_use.rs` | Modified | Added AST file map injection + grep/find intercept alongside existing domain rule injection |
| `src/hook/post_tool_use.rs` | Modified | Added section-specific AST entity context for partial reads |
| `src/domain/session.rs` | Modified | Added `ast_injected: HashSet<String>` + `has_ast_injected`/`mark_ast_injected` methods |

## Decisions Made

| Decision | Rationale | Impact |
|----------|-----------|--------|
| CONTAINS match for entity labels | Tree-sitter labels include `()` suffix; exact match misses them | --calls and --contains work with bare function names |
| Import query matches IRI stem | Import edges target module IRIs (code:src_store), not file labels (store.rs) | --imports strips extension and matches against IRI string |
| Grep intercept = guidance only | Blocking grep would break legitimate non-code-search uses | Model can still grep; the hint nudges toward graph query |
| AST dedup via SessionState | Prevents repeated injection on re-reads of the same file | First read gets the map; subsequent reads are clean |
| ast_query exposes both CLI + hook helpers | file_map_compact() and section_entities() avoid reimplementing query logic in hooks | Single source of truth for AST query patterns |

## Deviations from Plan

### Summary

| Type | Count | Impact |
|------|-------|--------|
| Auto-fixed | 2 | Label matching and import IRI resolution adjusted |
| Deferred | 1 | AST data is stale (pre-Plan 01 code changes) |

### Auto-fixed Issues

**1. Label suffix mismatch**
- Found during: Task 1 verification
- Issue: `--calls "load_and_mutate"` didn't match label `"load_and_mutate()"`
- Fix: Changed exact LCASE match to CONTAINS for calls query
- Verification: `--calls "dispatch"` returns signal/mod.rs:12

**2. Import IRI mismatch**
- Found during: Task 1 verification
- Issue: `--imports "store.rs"` returned empty — import edges use module IRIs not file labels
- Fix: Strip file extension and match against IRI string with CONTAINS
- Verification: `--imports "config.rs"` returns 24 dependent files

### Deferred Items

- AST data needs re-extraction (`base sync --ast`) to reflect Plan 01 code changes (milestone, resolve_slug, etc.) — not blocking, but --contains won't find new entities until re-synced

## Next Phase Readiness

**Ready:**
- Graph-backed code navigation fully operational
- Hooks fire for all Claude Code instances (main + subagents)
- 4 query modes cover discovery, orientation, caller analysis, and dependency mapping

**Next steps:**
- Re-run `base sync --ast` to update graph with latest code
- Phase 7 Plan 03: v1 JSON → triples migration + cutover (the original Phase 7 scope)

---
*Phase: 07-v1-migration-cutover, Plan: 02*
*Completed: 2026-06-02*
