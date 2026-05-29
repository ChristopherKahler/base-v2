# BASE v2 — Ontological State Engine

> A ground-up rebuild of the BASE framework that replaces v1's JSON-file + MCP-CRUD model with an RDF knowledge graph (Oxigraph), governing operator / workspace / project state across three tiers through one queryable, self-healing store.

**Type:** Application (framework / internal tooling)
**Stack:** Oxigraph (embedded RDF store) · Python extraction pipeline · SPARQL query layer · MCP write server · Claude Code hooks · markdown/TOML authoring surface
**Skill Loadout:** `/paul:init` (managed build) · `/paul:audit` (per-milestone) · `/aegis:audit` (post-build)
**Quality Gates:** idempotent extraction (zero dup/drift) · SPARQL query correctness · hook latency budget · lossless v1 migration

> **v1 reference (do not edit in place):** `apps/base/` · v1 data: `.base/data/*.json`
> **Fork substrate:** `~/ops-sys/toolbox/mcp-servers/open-ontologies` (already extended with markdown-frontmatter parsing, TOML namespace config, Obsidian TTL export)
> **Full ideation:** `planning/base-v2/PLANNING.md`

---

## Overview

BASE v1 tracks projects, tasks, people, tools, frameworks, and decisions as JSON records mutated through an MCP CRUD server and surfaced via session-start hooks. It fails structurally, not cosmetically:

- **No referential integrity** — `workspace.json` registers the same satellite twice (`aegis` *and* `sat_30d02113`, etc.) because JSON identity is by-key, not referential.
- **No supersession** — `entities.json` still calls a dissolved partnership current; JSON can't retire a fact.
- **Firehose surfacing** — hooks dump the whole backlog/active list every session; no relevance filter, so it gets tuned out.
- **Closed query surface** — only the questions someone hardcoded as tools are answerable.

The data is already a relationship graph forced into JSON adjacency lists. v2 moves it onto RDF + SPARQL — an open, relationship-oriented model where queries compose at runtime, relationships traverse arbitrarily, inference materializes unasserted facts, and IRIs make identity global and merge-safe. For a system whose value *is* the connections, that's a categorical upgrade.

---

## Architecture

### Three tiers = named graphs

| Tier | Named graph | Store | Owns |
|------|-------------|-------|------|
| User / global | `ops:graph/global` | `.base-gbl/` | Operator profile, goals, cross-workspace entities, shared vocabulary |
| Workspace (×3) | `ops:graph/ws/{slug}` | `{workspace}/.base/` | Projects, tasks, reminders, decisions, knowledge docs |
| Project | folded into workspace graph | — | PAUL project state, indexed read-only |

Shared vocabulary (`ops:`) is global; instances are per-tier. Cross-tier edges are legal (global goal → workspace project → app). Global can query down across workspace graphs.

### Two write paths, one graph

1. **MCP server (imperative)** — BASE-owned mutable state (projects, tasks, reminders, entities, decisions, goals). Mutations are SPARQL UPDATE scoped to the entity IRI (`DELETE`/`INSERT`), idempotent, supersession-preserving. No raw SPARQL scattered across commands — one guarded write path.
2. **Extraction (declarative)** — file-owned data. MOP (markdown frontmatter → triples) for knowledge docs; a JSON plugin (`paul.json` key-values → triples) for PAUL projects. Each scan clears that node's subgraph and re-emits from the current file.

**BASE never writes foreign state.** PAUL is *indexed* read-only via extraction (`marker_file: .paul/paul.json`, `field_map` keys→predicates). Because extraction is idempotent and keyed on the project IRI (derived from path/slug, never a generated `sat_xxx`), re-scan overwrites in place — the v1 satellite rot is structurally impossible. The file is truth; the graph is a disposable projection.

### Install model — two setup commands

- **`init-global`** — first run, once per machine. Stands up `.base-gbl/`, loads the shared `ops:` ontology + TOML namespaces, embeds the open-ontologies engine. Idempotent.
- **`init-workspace`** — per workspace. Stands up `{workspace}/.base/` and registers the workspace up into the global graph.

---

## Data Model (the ontology)

| Class | Subtypes | Source of truth | Write path |
|-------|----------|-----------------|------------|
| `ops:Project` | App, Framework, TrackingProject | BASE | MCP |
| `ops:Task` | — | BASE | MCP |
| `ops:Reminder` | — | BASE | MCP |
| `ops:Entity` | Person, Organization | BASE | MCP |
| `ops:Decision` | — | BASE | MCP |
| `ops:Goal` | — | BASE | MCP |
| `ops:Workspace` | — | init-workspace | MCP (registers up) |
| `ops:Document` | — | markdown file | Extraction (MOP) |
| `ops:PaulProject` | — | `.paul/paul.json` | Extraction (read-only) |

- **IRI scheme is load-bearing** — stable identity (path + slug), never generated ids. Same entity referenced N places = one node.
- **Supersession is native** — retire via `ops:status superseded` + timestamp or `ops:supersedes` edge; never silent deletion. Free audit trail.

---

## API Surface

- **Write (MCP):** tools for BASE-owned entities only — project / task / reminder / entity / decision / goal / workspace-registration.
- **Read (SPARQL):** open query surface over the merged/federated store.
- **Signal (hooks):** session-start hooks run pre-filtered parameterized SPARQL (active, not blocked, stale > threshold, ordered, limited) and inject only surviving rows — replaces the v1 firehose. PostToolUse hook captures entity edges on Write/Edit.

---

## Deploy

Local, single-operator. Web threat model (OWASP/authz/PII) is N/A. The integrity guarantee is idempotent IRI-keyed extraction + guarded MCP mutation + no silent deletion. v2 ships the open-ontologies fork inside the package as the embedded engine.

---

## UI/UX

"UI" = the surface layers Claude reads: session-start injection (SPARQL-filtered), pulse/health (integrity + staleness), optional Obsidian export (TTL → notes + Leiden communities), and a possible later dashboard rebuilt against SPARQL. No web frontend.

---

## Implementation Phases

Maps directly to PAUL milestones. Phase 5 (signal layer) is the proof — validate on the single hardest signal (`active-awareness`) before migrating the rest.

| # | Phase | Build | "It works" |
|---|-------|-------|-----------|
| 0 | Ontology + Fork Hardening | `ops:` vocabulary, IRI scheme, named-graph tiering, TOML namespaces | Hand-authored TTL loads + validates; sample SPARQL returns expected rows |
| 1 | Global Install | `init-global` → `.base-gbl` + ontology + engine | Fresh machine → one command → queryable empty global graph |
| 2 | Workspace Install | `init-workspace` → `.base/` + register up | Workspace node appears in global graph; cross-tier query works |
| 3 | Extraction Layer | MOP + paul.json plugin → idempotent IRI-keyed triples | Re-scan twice → identical graph; edit file → old fact gone |
| 4 | MCP Write Layer | SPARQL-UPDATE tools for BASE-owned entities | Complete a task → status mutates in place, prior fact retained |
| 5 | Signal Layer (proof) | Pre-filtered SPARQL hooks; port `active-awareness` first | Filtered injection beats v1 firehose on noise |
| 6 | Legacy-Conversion Pipeline | scan → approve → propose frontmatter → approve → apply → graph; `conversionStatus` in graph | Point at docs folder → gated flow → approved docs graphed |
| 7 | v1 Migration + Cutover | one-time `.base/data/*.json` → triples; retire v1 MCP/JSON | Every v1 record in graph; dead JSON unreferenced |
| 8 | Dashboard (optional) | Rebuild against SPARQL | — |

---

## Design Decisions

1. **Clean rebuild, not refactor** — lift concepts from v1, port nothing wholesale.
2. **Ontological replacement, not augmentation** — the graph IS the model.
3. **Three tiers as named graphs** — global vocabulary shared, instances per-tier, cross-tier edges allowed; BASE governs all three.
4. **Two setup commands** — `init-global` + `init-workspace`.
5. **open-ontologies ships inside the package** as embedded store + query engine.
6. **Two write paths, one graph** — MCP (imperative, BASE-owned) + extraction (declarative, file-owned + foreign).
7. **PAUL indexed into the graph, read-only** — JSON-extraction plugin, idempotent, IRI-keyed. Write-ownership (PAUL owns writes) is independent of graph-presence (PAUL is still a queryable node); conflating them was the v1 mistake.
8. **Pre-filtered SPARQL in hooks** — relevance is a query, not a firehose.
9. **Legacy-conversion is gated + self-tracking** — pipeline progress lives in the graph.
10. **v1 rot is a design target, structurally eliminated** — IRI-keyed idempotent extraction + supersession.

---

## Open Questions

These gate Phase 0/1 architecture — resolve early:

1. **Store topology** — one shared store with named graphs (easy cross-tier query) vs. per-workspace stores federated on read (portable, decoupled).
2. **Hook query execution** — pyoxigraph in-process (fast, no server dependency) vs. MCP roundtrip.
3. **MCP server language** — Node (v1 parity) vs. Python (fork cohesion).
4. **CARL relationship** — absorb CARL decisions into the graph vs. index it as a foreign source.
5. **Workspace vocabulary extension** — how a workspace adds entity types without forking the global `ops:` contract.
6. **Onto tool surface** — confirm the ~5 tools to expose vs. suppress the other ~46.
7. **The three workspaces** — enumerate (chris-ai-systems + chris-ai-socials known; third TBD).
8. **Capture-before-store** — the PostToolUse edge-capture hook fires ahead of any persistent store; Phase 0/1 must stand up the store before capture is meaningful.

---

## References

- v1: `apps/base/` (BASE-V1-SPEC.md, BASE-V2-SPEC.md, src/)
- Fork: `~/ops-sys/toolbox/mcp-servers/open-ontologies` (README, `docs/markdown-ontology-protocol.md`, `scripts/extractor.py`, `scripts/post_tool_hook.py`, `scripts/obsidian_sync.py`)
- v1 data shape: `.base/data/*.json`, `.base/workspace.json`
- PAUL groundwork: paul-framework "Frontmatter Emission" phase (shipped 2026-05-26)
- Ideation: `planning/base-v2/PLANNING.md`
