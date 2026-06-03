# Roadmap: base-v2

## Overview

Build the BASE v2 proactive context engine from the Rust scaffold up: define the ontology, wire hooks, implement domain matching, add CRUD commands, prove the signal layer against the hardest existing signal, absorb CARL, then migrate off v1. Phase numbering reflects the CLI-first architecture locked in the 2026-05-31 design session.

## Current Milestone

**v1.0 Agentic OS Distribution** (v1.0.0)
Status: Active
Phases: 1 of 4 complete

## Previous Milestones

### v0.1 Proactive Context Engine (v0.1.0) — Complete ✓
Phases: 9 of 9 complete

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
| 8 | Command Center Dashboard | 07 | In Progress (polish) | 2026-06-02 |
| 9 | PAUL Integration Layer | 02 | Complete ✓ | 2026-06-03 |

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

### Phase 9: PAUL Integration Layer

**Goal:** BASE v2 consumes PAUL's session ledger (`ledger.toml`) to provide per-project, per-phase, per-milestone cost attribution. The missing link between "which sessions did PAUL work" and "how much did those sessions cost." PAUL produces clean timestamped ledger entries; BASE extracts them into the graph and joins against session JSONL for token/cost data.
**Depends on:** Phase 4 (Extraction Layer), Phase 8 (Usage Analytics)
**Coordinates with:** PAUL Framework v1.4 (Agentic OS Integration) — PAUL produces ledger.toml, BASE consumes it.

**Scope:**

**Plan 1: Ledger Extractor + Schema**
- `ledger.toml` file spec (append-only, `[[entry]]` tables with action/phase/plan/timestamp/note)
- New extractor in `src/extract/` producing `ops:LedgerEntry` triples
- Timestamp-match join: ledger entry `at` → session JSONL time range → token/cost attribution
- API endpoint: `/api/ops/ledger` returning entries with cost data joined
- Integration with existing `collect_all_events()` provider pipeline

**Plan 2: Cost Attribution Dashboard**
- Per-phase cost breakdown (plan/apply/iterate/unify costs)
- Per-milestone cost rollup
- Per-project lifetime cost
- Operations panel or Usage Analytics integration (TBD based on Plan 1 output)

---

## v1.0 Agentic OS Distribution — Phase Details

### Phase 10: Manifest & Version Infrastructure

**Goal:** Create the local manifest system that tracks installed Agentic OS components, versions, and update check state.
**Depends on:** Nothing (first phase of new milestone)

**Scope:**
- Define `~/.base-gbl/manifest.toml` format:
  ```toml
  [agentic-os]
  installed_at = "2026-06-03T14:00:00-05:00"
  source = "https://chrisai.cv/skool"
  token = ""  # HMAC(installed_at, compiled_secret). Valid token = direct install (no promos). Invalid/empty = community (full pipeline).

  [components.base]
  version = "0.1.0"
  path = "~/.local/bin/base"
  installed_at = "2026-06-03T14:00:00-05:00"

  [components.paul]
  version = "1.4.0"
  path = "~/.claude/commands/paul"
  installed_at = "2026-06-03T14:00:00-05:00"

  [components.seed]
  version = "1.0.0"
  path = "~/.claude/commands/seed"
  installed_at = "2026-06-03T14:00:00-05:00"

  [components.skillsmith]
  version = "1.0.0"
  path = "~/.claude/commands/skillsmith"
  installed_at = "2026-06-03T14:00:00-05:00"

  [update_check]
  last_checked = "2026-06-03T14:00:00-05:00"
  ttl_seconds = 604800  # 7 days
  endpoint = "https://chrisai.cv/agentic-os/versions.json"
  pending_update = ""          # empty = no pending, else "PAUL 1.4.0→1.5.0, ..."
  dismissed_until = ""         # empty = not dismissed, else ISO timestamp
  ```
- `base install` writes/updates manifest.toml on every install
- `base install --full` creates the full manifest with all 4 components
- Manifest is the single source of truth for what's installed

### Phase 11: Update Check & Persistent Banner

**Goal:** Weekly version check against CDN. Once an update is detected, banner persists on EVERY session start until the user either updates or snoozes (24h). Two exit paths: `base update` or `base update --snooze`.
**Depends on:** Phase 10 (manifest must exist)

**Scope:**
- In session_start handler, after existing signals:
  0. Read `~/.base-gbl/manifest.toml`
  1. **If `install_channel == "direct"` → skip entire banner/promo pipeline.** Still run version check silently (write `pending_update` to manifest for `base update --check`), but never inject session output. Creator-installed instances get zero noise.
  2. **If `install_channel == "community"` (or absent/unknown) → full pipeline:**
  3. **If `pending_update` is set AND `dismissed_until` is empty or expired:**
     - Inject banner EVERY session:
       ```
       ═══════════════════════════════════════
       🔄 Agentic OS update available
          {pending_update contents}

          Run: base update
          Snooze 24h: base update --snooze
          Chris AI Systems · https://chrisai.cv/skool
       ═══════════════════════════════════════
       ```
     - Do NOT make an HTTP call — already know updates exist
     - Skip to end
  3. **If `pending_update` is set AND `dismissed_until` is in the future:**
     - Skip silently (user snoozed, respect it)
  4. **If no `pending_update`:**
     - Check `last_checked` timestamp
     - If now - last_checked < 604800 seconds (7 days) → skip silently
     - If >= 7 days → HTTP GET to `update_check.endpoint`
       - Timeout: 3 seconds (never block session start)
       - On failure (timeout, offline, 404, parse error): skip silently, do NOT update last_checked (retry next session)
     - On success: compare `versions.json` versions vs `components.*.version`
       - All current → update `last_checked`, clear `pending_update`, no output
       - Updates found → write `pending_update` string (e.g., "PAUL 1.4.0→1.5.0, SEED 1.0.0→1.1.0"), update `last_checked`, inject banner
  5. If `versions.json` contains a `message` field → append to banner (promotional channel)
- `base update` → installs updates, clears `pending_update` and `dismissed_until`
- `base update --snooze` → sets `dismissed_until` = now + 24h, does NOT clear `pending_update`
- Traffic math: 1000 users × 1 check/week = ~143 checks/day → CDN-cached static file, origin sees ~24 refreshes/day max
- Banner frequency: every session until resolved — no passive decay
- `versions.json` format:
  ```json
  {
    "base": "0.2.0",
    "paul": "1.5.0",
    "seed": "1.0.0",
    "skillsmith": "1.0.0",
    "message": ""
  }
  ```

### Phase 12: `base install --full` and `base update`

**Goal:** Single command installs the full Agentic OS. `base update` pulls latest versions of outdated components.
**Depends on:** Phase 10 + 11 (manifest + version check)

**Scope:**
- `base install --full`:
  1. Install BASE binary (existing logic)
  2. Wire hooks (existing logic)
  3. Download PAUL/SEED/SKILLSMITH from GitHub releases (tar.gz of skill files)
  4. Extract to `~/.claude/commands/{paul,seed,skillsmith}/`
  5. Write full manifest.toml
  6. Print Agentic OS welcome banner with chrisai.cv/skool link
- `base update`:
  1. Fetch versions.json (force, ignore TTL)
  2. Compare against manifest
  3. Download + install only outdated components
  4. Update manifest versions + last_checked
  5. Report what was updated
- `base update --check`:
  1. Fetch versions.json, report status, don't install anything
- Cross-platform: GitHub release assets include pre-built BASE binaries for linux-x86_64, darwin-x86_64, darwin-aarch64, windows-x86_64

### Phase 13: Unofficial Install Detection & Attribution

**Goal:** If someone installs frameworks outside the official channel, BASE detects it and shows a gentle redirect. Provenance verification.
**Depends on:** Phase 10 (manifest is what "official" means)

**Scope:**
- **Skip entirely if `install_channel == "direct"`.** Creator installs never see unofficial detection messaging.
- For `community` channel (or absent/unknown manifest):
- On session start, if BASE detects PAUL/SEED/SKILLSMITH commands exist but NO manifest.toml:
  ```
  ℹ️ Agentic OS frameworks detected without official manifest.
  For updates, support, and training from the creator:
  https://chrisai.cv/skool

  Run: base install --full (to register and enable auto-updates)
  ```
- If manifest exists but `source` field is not `https://chrisai.cv/skool`:
  Same message — gentle, non-blocking redirect
- If frameworks exist with `skillsmith_source` / `seed_source` / `paul.source` fields in their configs pointing to chrisai.cv/skool but no manifest: suggest `base install --full` to complete the setup
- Never block. Never hostile. Just persistent, polite awareness.

---
*Roadmap created: 2026-05-29*
*Last updated: 2026-06-03 — Phase 10 complete*
