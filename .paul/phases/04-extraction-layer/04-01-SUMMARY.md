---
phase: 04-extraction-layer
plan: 01
subsystem: extraction
tags: [rust, sync, frontmatter, paul-json, mtime, incremental, glob, idempotent]

requires:
  - phase: 03-write-commands/01
    provides: CRUD infrastructure (load_workspace_store, build_iri, workspace_graph_iri, prefixes)
provides:
  - base sync command (full and incremental)
  - Markdown frontmatter extractor (YAML → Document triples)
  - paul.json extractor (PAUL metadata → PaulProject triples)
  - mtime-based incremental extraction (ops:lastExtracted)
  - Configurable include/exclude patterns via SyncConfig in base.toml
  - Idempotent extraction (re-scan = identical graph)
affects: [signal-layer, v1-migration]

key-files:
  created: [src/extract/mod.rs, src/extract/frontmatter.rs, src/extract/paul_json.rs, tests/extract_test.rs]
  modified: [Cargo.toml, src/config.rs, src/cli.rs, src/lib.rs, tests/hook_session_start_test.rs]

key-decisions:
  - "Basic YAML parsing (string split, no heavy dependency) — frontmatter is simple key-value"
  - "File IRI from path slug (replace /.\\ with hyphens) — stable, human-readable"
  - "DELETE+INSERT pattern for re-extraction — ensures idempotency"

duration: ~15min
started: 2026-06-01T10:31:00-05:00
completed: 2026-06-01T10:56:00-05:00
---

# Phase 4 Plan 01: Extraction Layer Summary

**`base sync` with idempotent file-to-graph extraction — frontmatter and paul.json extractors, mtime incremental, configurable include/exclude.**

## Performance

| Metric | Value |
|--------|-------|
| Duration | ~15 min |
| Tasks | 3 completed |
| Files created | 4 |
| Files modified | 5 |
| Tests | 77 pass (28 unit + 6 extract + 38 regression + 5 Phase 0) |

## Acceptance Criteria Results

| Criterion | Status | Notes |
|-----------|--------|-------|
| AC-1: Sync extracts markdown frontmatter | Pass | YAML frontmatter → Document triples with name, status, path |
| AC-2: Sync extracts paul.json | Pass | PaulProject triples with name, phase, milestone, lastActive |
| AC-3: Extraction is idempotent | Pass | Re-sync produces identical triple count (test verified) |
| AC-4: Incremental uses mtime comparison | Pass | Unchanged files skipped; mtime > lastExtracted triggers re-extract |
| AC-5: Include/exclude configurable | Pass | SyncConfig in BaseConfig; defaults exclude node_modules, target, .git, .base |

## Deviations

- BaseConfig struct change required fixing one Phase 1 test (`hook_session_start_test.rs`) that constructed BaseConfig manually — added `..BaseConfig::default()` for forward compatibility.

## Next Phase Readiness

**Ready:**
- Graph now auto-populates from workspace files — session-start queries will surface real extracted data
- Extraction infrastructure supports adding new extractors (plug in a function, add to the router)
- AST/code extraction can be added as a new extractor module in a future plan

**Blockers:** None.

---
*Phase: 04-extraction-layer, Plan: 01*
*Completed: 2026-06-01*
