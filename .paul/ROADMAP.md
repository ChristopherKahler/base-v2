# Roadmap: base-v2

## Overview

Build the BASE v2 proactive context engine from the Rust scaffold up: define the ontology, wire hooks, implement domain matching, add CRUD commands, prove the signal layer against the hardest existing signal, absorb CARL, then migrate off v1. Phase numbering reflects the CLI-first architecture locked in the 2026-05-31 design session.

## Current Milestone

**v0.1 Proactive Context Engine** (v0.1.0)
Status: In progress
Phases: 6 of 9 complete (Phase 7 in progress)

## Phases

| Phase | Name | Plans | Status | Completed |
|-------|------|-------|--------|-----------|
| 0 | Rust Scaffold + Ontology | 01 | Complete ✓ | 2026-05-31 |
| 1 | Hook Engine | 01 | Complete ✓ | 2026-06-01 |
| 2 | Domain Matcher + Rule Injection | 01 | Complete ✓ | 2026-06-01 |
| 3 | Write Commands (CRUD) | 02 | Complete ✓ | 2026-06-01 |
| 4 | Extraction Layer | 01 | Complete ✓ | 2026-06-01 |
| 5 | Signal Layer (proof) | 01 | Complete ✓ | 2026-06-01 |
| 6 | CARL Absorption | 03+ | Complete ✓ | 2026-06-01 |
| 7 | v1 Migration + Cutover | 02 | Complete ✓ | 2026-06-02 |
| 8 | Command Center Dashboard | 01+ | Planning | - |

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

### Phase 8: Command Center Dashboard

**Goal:** A local web dashboard served by the base binary itself (`base dashboard`). Four panels covering graph exploration, business operations, live session activity, and usage analytics. The visual surface that makes BASE tangible and marketable.
**Depends on:** Phase 7
**Research:** axum/warp for embedded HTTP, D3.js for graph viz, WebSocket for live panel, ccusage JSONL format for usage data.

**Scope:**

**Panel 1: Graph Explorer**
- D3.js force-directed graph visualization
- Nodes color-coded by entity type (code=blue, projects=green, people=orange, decisions=yellow)
- Click node → expand neighborhood, show relationships, line numbers for code entities
- Filter by entity type, domain, project
- Search bar (runs ast query --contains visually)

**Panel 2: Operations View**
- Kanban board: columns by status (active/blocked/completed/pending), cards are tasks
- Table view: sortable/filterable table of projects → milestones → tasks
- Toggle between kanban and table (same data, two views)
- People and their project connections
- Recent decisions with rationale
- Overdue reminders highlighted

**Panel 3: Session Activity (live)**
- WebSocket-fed log of hook events in real-time
- Which hooks fired, what domains matched, what got injected vs suppressed
- Context bracket state progression through the session
- AST injection log: which files got maps, which sections got detail
- Hooks append to a log file, server tails and pushes via WebSocket

**Panel 4: Usage Analytics**
- Fork/fold ccusage (apps/ccusage) JSONL parsing for Claude Code token usage
- Daily/weekly/monthly token usage charts
- Cost tracking and model distribution (Opus/Sonnet/Haiku)
- Session-level breakdown (which sessions burned the most tokens)
- Per-project usage attribution (via session → workspace → project mapping)

**Architecture:**
- `base dashboard` starts embedded axum HTTP server on localhost, opens browser
- Frontend: SPA (React or Svelte) compiled to static assets, embedded in binary via include_dir!
- Backend: SPARQL queries against graph.trig + ccusage JSONL parsing → JSON API endpoints
- Live data: WebSocket for session activity panel, hooks append to structured log file
- No external dependencies for the user — one command, one binary

---
*Roadmap created: 2026-05-29*
*Last updated: 2026-06-02 — Phase 8 scoped as Command Center Dashboard*
