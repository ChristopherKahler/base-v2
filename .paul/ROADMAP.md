# Roadmap: base-v2

## Overview

Build the BASE v2 proactive context engine from the Rust scaffold up: define the ontology, wire hooks, implement domain matching, add CRUD commands, prove the signal layer against the hardest existing signal, absorb CARL, then migrate off v1. Phase numbering reflects the CLI-first architecture locked in the 2026-05-31 design session.

## Current Milestone

**v0.1 Proactive Context Engine** (v0.1.0)
Status: In progress
Phases: 5 of 9 complete

## Phases

| Phase | Name | Plans | Status | Completed |
|-------|------|-------|--------|-----------|
| 0 | Rust Scaffold + Ontology | 01 | Complete ✓ | 2026-05-31 |
| 1 | Hook Engine | 01 | Complete ✓ | 2026-06-01 |
| 2 | Domain Matcher + Rule Injection | 01 | Complete ✓ | 2026-06-01 |
| 3 | Write Commands (CRUD) | 02 | Complete ✓ | 2026-06-01 |
| 4 | Extraction Layer | 01 | Complete ✓ | 2026-06-01 |
| 5 | Signal Layer (proof) | TBD | Not started | - |
| 6 | CARL Absorption | TBD | Not started | - |
| 7 | v1 Migration + Cutover | TBD | Not started | - |
| 8 | Dashboard (optional) | TBD | Not started | - |

## Phase Details

### Phase 0: Rust Scaffold + Ontology

**Goal:** A compiling Rust binary with embedded Oxigraph that can load a hand-authored `ops:` vocabulary, parse TriG files, and answer SPARQL queries. The foundation everything targets.
**Depends on:** Nothing (first phase)
**Research:** Minimal — oxigraph, clap, serde crates are well-documented.

**Scope:**
- `cargo init` with deps: oxigraph, clap, serde_json, toml
- `ops:` class + predicate vocabulary as TTL
- IRI scheme (path/slug-based, stable identity)
- Named-graph tiering model (global + workspace TriG files)
- Basic CLI skeleton (`base --version`, `base --help`)
- Unit tests: load TTL → SPARQL → expected rows

### Phase 1: Hook Engine

**Goal:** `base hook session-start` and `base hook post-tool-use` work end-to-end — reading event JSON on stdin, loading TTL, running SPARQL, emitting filtered injection to stdout. Wired into `settings.json`.
**Depends on:** Phase 0
**Research:** Unlikely — hook contract is stdin/stdout JSON.

**Scope:**
- `base hook session-start` — load workspace + global TTL → pre-filtered SPARQL → stdout
- `base hook post-tool-use` — parse event → update lastActive → atomic write-back
- SessionEnd nudge (threshold-gated "anything to log?")
- Wire into `~/.claude/settings.json`

### Phase 2: Domain Matcher + Rule Injection

**Goal:** Deterministic domain matching via `domains.toml` replaces CARL's Python matching. Multi-signal: keyword + path + active-project + sticky session.
**Depends on:** Phase 1
**Research:** Unlikely — CARL's matching logic is the reference implementation.

**Scope:**
- `domains.toml` parser (triggers, paths, exclude, match mode, sticky, rules)
- Multi-signal matcher (prompt keywords + edited file paths + active-project edges)
- `base hook user-prompt-submit` — match → inject rules to stdout
- Dedup (never re-inject what's in context)
- `base domain add-trigger` (structured CLI mutation of domains.toml)

### Phase 3: Write Commands (CRUD)

**Goal:** CLI commands for BASE-owned entity management — projects, tasks, decisions, entities, goals, reminders. All mutations are SPARQL UPDATE, IRI-scoped, with supersession-preserving semantics.
**Depends on:** Phase 0
**Research:** Unlikely

**Scope:**
- `base project add/list/get/update`
- `base task add/done/list`
- `base decision log/search`
- `base entity add/list/update`
- `base goal add/update`
- `base reminder add/list/remove`
- Atomic TTL write-back (temp + rename) for all mutations
- Superseded-fact retention (audit trail)

### Phase 4: Extraction Layer

**Goal:** `base sync` scans workspace files and extracts to graph — idempotent, IRI-keyed. Markdown frontmatter (MOP) and paul.json plugin.
**Depends on:** Phase 0
**Research:** Unlikely — extraction patterns exist from v1 fork work.

**Scope:**
- `base sync` command
- Markdown frontmatter → triples (MOP port to Rust)
- paul.json key-value → triples plugin (field_map)
- Re-scan = overwrite-in-place (no dup/drift)

### Phase 5: Signal Layer (the proof)

**Goal:** Validate that SPARQL-filtered injection beats v1's firehose. Port `active-awareness` (the hardest signal) first, then remaining signals.
**Depends on:** Phase 3, Phase 4
**Research:** Unlikely

**Scope:**
- Parameterized SPARQL per signal type
- Port `active-awareness` first
- Side-by-side comparison vs v1 firehose output
- Port remaining signals (backlog, pulse, staleness)
- Suppression layer: dedup, novelty gate, salience threshold, budget cap

### Phase 6: CARL Absorption

**Goal:** Faithful port of CARL's remaining mechanisms into the Rust binary. Python CARL retires.
**Depends on:** Phase 2, Phase 5
**Research:** Likely — reconciling CARL domains vs BASE projects taxonomy.

**Scope:**
- Port dedup signatures, rule priority
- DEVMODE dashboard output
- PSMM coupling
- Reconcile CARL domains ↔ BASE workspaces/projects into one model
- Retire Python CARL hooks

### Phase 7: v1 Migration + Cutover

**Goal:** One-time transform of v1 `.base/data/*.json` → triples; retire v1 MCP/JSON.
**Depends on:** Phase 3, Phase 5
**Research:** Unlikely

**Scope:**
- JSON → triples transform (projects, entities, backlog, reminders, decisions)
- Cutover: remove v1 MCP server, hooks, JSON files
- Fidelity check (100% records)

### Phase 8: Dashboard (optional)

**Goal:** Rebuild the operator dashboard against SPARQL.
**Depends on:** Phase 7
**Research:** Unlikely

**Scope:**
- SPARQL-backed views (deferred / optional)

---
*Roadmap created: 2026-05-29*
*Last updated: 2026-05-31 — Realigned to Rust CLI architecture (ARCHITECTURE.md)*
