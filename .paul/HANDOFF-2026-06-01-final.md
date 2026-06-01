# PAUL Handoff

**Date:** 2026-06-01 17:00
**Status:** paused — Phase 6 complete + system live + extensive post-plan productization

---

## READ THIS FIRST

You have no prior context. This document tells you everything.

**Project:** base-v2 — Proactive context-injection engine for Claude Code
**Core value:** Inject only the salient slice of context anchored on what Claude is touching. Stay silent until something changes.

---

## Current State

**Version:** 0.1.0
**Phase:** 6 of 8 — CARL Absorption — COMPLETE
**Status:** Binary LIVE at ~/.local/bin/base. All hooks wired. CARL retired. Tested with real sessions.

**Loop Position:**
```
PLAN ──▶ APPLY ──▶ UNIFY
  ✓        ✓        ✓     [Phase 6 complete]
```

**Milestone:** [█████████░] ~90%

---

## What Was Done (June 1, 2026 — Two Sessions)

### Session 1: Phase 6 PAUL Plans (3 loops)

| Plan | What | Commit |
|------|------|--------|
| 06-01 | Graph-backed domain rules, dual keywords, SPARQL injection, neighborhood context | `0452ba2` |
| 06-02 | DEVMODE dashboard, context brackets (FRESH/MODERATE/DEPLETED/CRITICAL) | `fb1cd48` |
| 06-03 | base learn — graph-backed structured memory with relational edges | `dbc81ff` |

### Session 2: Productization (25 commits post-plan)

| Feature | Commit |
|---------|--------|
| base install (global installer) | `0f180aa` |
| base rule CRUD (graph-native, stripped from TOML) | `ac29662` |
| Pre-tool-use hook (file path → rules BEFORE edit) | `ebd4acf` |
| paul.toml template + session-start auto-ingestion | `c938437` |
| Workspace registry in base.toml | `e59a3eb` |
| base scaffold (workspace setup) | `3d79463` |
| CLAUDE.md auto-append for Claude awareness | `bfbfb11` |
| Auto-domain on project add (path-first) | `52054ac` |
| Mandatory edges (no orphan entities) | `6018e17`, `5a68997` |
| Operator profile (operator.toml + session-start emission) | `fc77dc9` |
| Sync fix (additive-only, never deletes graph data) | `bbf08f1` |
| Star commands (*COMMAND parsing + commands.toml) | `7240177` |
| DEVMODE format spec (exact template) | `dbc85e6` |
| AST codebase extraction bundled (35+ languages) | `20d6776` |
| MOP doc + system rule seeded on install | `07c8cfc` |
| README rewritten as product documentation | `a873766` |
| v1 hooks removed from settings.json | (settings edits) |
| v1 MCP servers removed from .mcp.json | (mcp.json edit) |

### System Inventory

- **Binary:** `~/.local/bin/base` (0.1.0, Rust, ~16MB)
- **Global config:** `~/.base-gbl/base.toml` (devmode, brackets, signals, 4 workspaces)
- **Global domains:** `~/.base-gbl/domains.toml` (9 trigger-only domains)
- **Global graph:** `~/.base-gbl/graph.trig` (13 GLOBAL rules, 8 DEV rules, 4 projects, 3 tasks, 2 reminders, 2 decisions)
- **Operator:** `~/.base-gbl/operator.toml` (Chris's profile, emits on session-start)
- **Commands:** `~/.base-gbl/commands.toml` (DISCUSS, META, BRIEF, DEV)
- **MOP spec:** `~/.base-gbl/docs/markdown-ontology-protocol.md`
- **AST scripts:** `scripts/ast/` (onto_ast.py, extractor.py, ttl_serializer.py, cache.py)
- **CLAUDE.md:** BASE CLI section appended to `~/.claude/CLAUDE.md`
- **Hooks wired:** session-start, pre-tool-use, user-prompt-submit, post-tool-use

### Hooks in settings.json (FINAL STATE)

**Global (~/.claude/settings.json):**
- UserPromptSubmit: time, machine, `base hook user-prompt-submit`
- SessionStart: `base hook session-start`, clauditor, context-mode-cache-heal
- PreToolUse: `base hook pre-tool-use`
- PostToolUse: `base hook post-tool-use`, open-ontologies

**Workspace (chris-ai-systems/.claude/settings.json):**
- UserPromptSubmit: calendar
- SessionStart: social-engine-quota, firm-session-pulse

---

## Key Decisions Made Across Both Sessions

- CARL merges into BASE as a feature (not separate tool)
- domains.toml = trigger config ONLY — rules are graph-native
- Two keyword types: prompt_keywords + file_keywords
- PreToolUse for file-path matching (rules BEFORE tool executes)
- Mandatory edges on all entity-creating commands
- Workspace registry in base.toml (BASE owns its own config)
- paul.toml replaces paul.json (TOML consistency)
- Sync is additive-only (never deletes graph data)
- Operator profile is a core feature (operator.toml at global tier)
- Star commands via commands.toml (separate from domain matching)
- MOP spec bundled and auto-seeded on install
- IP attribution in binary output + generated configs

---

## What's Next

**Immediate:**
1. Port PAUL to emit paul.toml instead of paul.json
2. Port orientation skill to write operator.toml (global, not workspace)
3. Fix base sync frontmatter extraction bug (erroring on SUMMARY files)
4. Consider closing Phase 6 formally + transitioning to Phase 7

**Phase 7 (v1 Migration + Cutover):**
- Migrate .base/data/*.json → graph triples
- Remove v1 .base/base-mcp/ and .base/carl-mcp/ directories
- Verify all v1 data is in the graph
- Retire v1 JSON files

**Phase 8 (Dashboard, optional):**
- SPARQL-backed operator views

**Known gaps:**
- base sync --ast needs testing with real codebases
- Session-start signals need more graph data to be useful (projects seeded, but tasks/reminders sparse)
- No tests for post-plan features (install, scaffold, rule CRUD, pre-tool-use, star commands — all manually verified)
- Context bracket thresholds may need tuning
- base install can't copy over itself when running

---

## Resume Instructions

1. Read `.paul/STATE.md`
2. Run `/paul:resume`
3. Phase 6 is functionally complete — decide: close formally or move to Phase 7
4. Priority: fix the sync frontmatter bug so `base sync` works clean

---

*Handoff created: 2026-06-01 17:00*
