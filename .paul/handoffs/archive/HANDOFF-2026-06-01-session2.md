# PAUL Handoff

**Date:** 2026-06-01 15:48
**Status:** paused — massive session, Phase 6 + install + live testing + post-plan work

---

## READ THIS FIRST

You have no prior context. This document tells you everything.

**Project:** base-v2 — Proactive context-injection engine for Claude Code
**Core value:** Injecting only the salient slice of context anchored on what Claude is touching, staying silent the rest of the time.

---

## Current State

**Version:** 0.1.0
**Phase:** 6 of 8 — CARL Absorption — 3 of 4 PAUL plans complete + significant post-plan work
**Status:** Binary is LIVE and INSTALLED globally. CARL Python hooks retired.

**Loop Position:**
```
PLAN ──▶ APPLY ──▶ UNIFY
  ✓        ✓        ✓     [Plan 06-03 loop closed]
```

**Milestone:** [████████░░] ~80% — Phases 0-5 complete + Phase 6 at 75% via PAUL + extensive post-plan features

---

## What Was Done (This Session — June 1, 2026, Session 2)

### PAUL Plans Completed (3 loops)

| Plan | What | Tests | Commit |
|------|------|-------|--------|
| 06-01 | Graph-backed domain rules — dual keywords (prompt/file), domain sync, SPARQL injection with 1-hop neighborhood | 89 | `0452ba2` |
| 06-02 | DEVMODE dashboard + context brackets — FRESH/MODERATE/DEPLETED/CRITICAL, lean mode, force-refresh | 101 | `fb1cd48` |
| 06-03 | base learn — graph-backed structured memory with relational edges, auto-surface in injection | 106 | `dbc81ff` |

### Post-Plan Features (outside PAUL loop, built during testing)

| Feature | What | Commit |
|---------|------|--------|
| CARL retirement | Removed Python CARL hook from settings.json | (settings.json edit) |
| base install | Global install command: binary copy, ~/.base-gbl/, hook wiring, CARL migration, CLAUDE.md integration | `0f180aa` |
| base rule CRUD | Graph-native rules — base rule add/list/remove, stripped rules from domains.toml | `ac29662` |
| DEVMODE count fix | Rule count from graph, not TOML | `666181f` |
| Pre-tool-use hook | File path matching → inject domain rules BEFORE tool executes | `ebd4acf` |
| paul.toml template | Session-start scans all registered workspaces for paul.toml, auto-ingests projects | `c938437` |
| Workspace registry | [[workspace]] in base.toml, not settings.json — BASE owns its own registry | `e59a3eb` |
| base scaffold | Workspace setup: .base/, configs, auto-register globally | `3d79463` |
| CLAUDE.md integration | base install auto-appends BASE CLI section for Claude proactive usage | `bfbfb11` |
| Auto-domain on project add | base project add creates [[domain]] trigger with path matching | `52054ac` |
| Mandatory edges | --domain required on learn/entity, --path required on project add | `5a68997` |
| IP attribution | Chris AI Systems branding in --help, install output, generated configs | built into install |

### System State

- **Binary:** `/home/chriskahler/.local/bin/base` (0.1.0, 16MB release)
- **Global config:** `~/.base-gbl/base.toml` (devmode on, brackets on, 4 workspaces registered)
- **Global domains:** `~/.base-gbl/domains.toml` (9 domains, trigger-only, rules in graph)
- **Global graph:** `~/.base-gbl/graph.trig` (9 domains, 28+ rules, decisions synced)
- **CLAUDE.md:** BASE CLI section appended to `~/.claude/CLAUDE.md`
- **Hooks wired:** session-start, pre-tool-use, user-prompt-submit, post-tool-use

### Registered Workspaces
1. `/home/chriskahler/chris-ai-systems`
2. `/home/chriskahler/chris-ai-socials`
3. `/home/chriskahler/ops-sys/extendly`
4. `/home/chriskahler/ops-sys/outpost-v2`

---

## Key Decisions Made This Session

- CARL merges into BASE as a feature, not a peer tool
- domains.toml = trigger config ONLY — rules are graph-native (base rule add)
- Two keyword types: prompt_keywords (user-configured) and file_keywords (code-oriented)
- PreToolUse for file-path matching (rules land BEFORE tool executes)
- `base learn` replaces PSMM — graph-backed structured memory
- Mandatory edges on all entity-creating commands (no orphans)
- Workspace registry in base.toml, not settings.json
- paul.toml replaces paul.json (TOML consistency, richer schema)
- IP attribution baked into binary output + generated configs
- base install auto-appends CLAUDE.md section for Claude awareness

---

## What's Next

**Immediate priorities:**
1. Update PAUL framework to emit paul.toml instead of paul.json
2. Test session-start with real paul.toml data (need at least one paul.toml in a project)
3. Consider Phase 7 (v1 Migration) — migrate .base/data/*.json to graph triples

**Known gaps:**
- base install can't copy over itself when running (need to detect and skip)
- Session-start signals need graph data to be useful (paul.toml ingestion provides projects)
- Context brackets thresholds may need tuning based on real-world sessions
- PAUL Plan 06-04 (CARL retirement) can be closed formally or dropped since retirement happened manually
- No tests for post-plan features (install, scaffold, rule CRUD, pre-tool-use — all manually verified)

**Deferred:**
- File-keyword content scanning (reading file content for keyword match on pre-tool-use)
- Directory-scoped .base.toml config files
- Cross-workspace note recall
- base goal add --project linkage

---

## Key Files

| File | Purpose |
|------|---------|
| `.paul/STATE.md` | Live project state (NEEDS UPDATE) |
| `.paul/PROJECT.md` | Requirements and decisions |
| `.paul/ROADMAP.md` | Phase overview |
| `~/.base-gbl/base.toml` | Global config + workspace registry |
| `~/.base-gbl/domains.toml` | Global domain triggers |
| `~/.base-gbl/graph.trig` | Global knowledge graph |
| `~/.claude/CLAUDE.md` | User-level Claude instructions (includes BASE CLI section) |
| `~/.claude/settings.json` | Hook wiring for all 4 hooks |
| `src/install.rs` | Global install command |
| `src/scaffold.rs` | Workspace scaffold command |
| `src/hook/pre_tool_use.rs` | File-path domain matching |
| `src/crud/rule.rs` | Graph-native rule CRUD |
| `src/crud/note.rs` | base learn/recall |
| `src/extract/paul_toml.rs` | paul.toml scanner + graph ingestion |

---

## Resume Instructions

1. Read `.paul/STATE.md` for position
2. Run `/paul:resume`
3. STATE.md needs updating to reflect post-plan work
4. Phase 6 can be closed (CARL retirement happened, all features shipped)
5. Consider: update PAUL to emit paul.toml, then Phase 7 (v1 migration)

---

*Handoff created: 2026-06-01 15:48*
