# Phase 0 Context: Ontology + Fork Hardening

*Created via /paul:discuss — 2026-05-29. Source: planning/base-v2/PLANNING.md + apps/base-v2/README.md.*

## Goal

Define the schema everything downstream targets: the `ops:` vocabulary, the IRI scheme, named-graph tiering, and TOML namespace config — plus harden the open-ontologies fork to BASE's needs. This is the foundation gate; Phases 1–8 all target what's locked here. Get the IRI scheme + tiering right and re-scans stay idempotent (v1 rot structurally impossible); get them wrong and v1's rot is rebuilt on RDF.

## Architecture Decisions (locked this phase)

These were the three Phase-0-gating Open Questions (PLANNING.md §Open Questions 1–3). Resolved:

| # | Decision | Rationale |
|---|----------|-----------|
| 1 | **Store topology: single shared Oxigraph store, one named graph per tier** | Federation only buys portability/decoupling — irrelevant for a single-operator local tool. Cross-tier query (global goal → workspace project) is a core feature; trivial in a shared store, painful federated. (Confirmed operator's stated lean.) |
| 2 | **Hook query execution: pyoxigraph in-process** | Session-start reads must not depend on MCP server liveness, and a roundtrip adds latency every session. Clean split established: **reads = in-process SPARQL, writes = MCP.** |
| 3 | **MCP server language: Python** | Only pull toward Node was v1 parity, but Design Decision #1 ("port nothing wholesale") means no v1 code is reused. Fork + extraction + hooks are all Python — one language across the stack. |

## Scope

1. `ops:` vocabulary authored as TTL — classes, subtypes, predicates from the Data Model table (Project[App/Framework/TrackingProject], Task, Reminder, Entity[Person/Organization], Decision, Goal, Workspace, Document, PaulProject).
2. IRI scheme — derived from stable identity (path + slug), never generated `sat_xxx` ids. Document with one concrete example IRI per class.
3. Named-graph tiering model — `ops:graph/global`, `ops:graph/ws/{slug}`, project folded into its workspace graph. Shared-store decision baked in.
4. TOML namespace config.
5. Fork hardening — trim the onto tool surface to the ~5 BASE needs (ingest, query, load, save, search); suppress the enterprise surface (reason/shacl/align/crosswalk/dl_check). Confirm pyoxigraph in-process read path works. Confirm the fork's existing extensions (markdown-frontmatter parsing, TOML namespace config, Obsidian TTL export) still load.
6. Workspace vocabulary extension — minimal pattern for a workspace to add its own entity types via `ws:`-namespace subclasses of `ops:` types, without forking the global `ops:` contract (Open Q#5 — the only other open question touching Phase 0).

## Approach / Constraints

- **Substrate:** open-ontologies fork at `~/ops-sys/toolbox/mcp-servers/open-ontologies` — already extended with markdown-frontmatter parsing, TOML namespace config, Obsidian TTL export. Build on it, don't reinvent.
- **Supersession native** — facts retire via `ops:status superseded` + timestamp or `ops:supersedes` edge, never silent deletion.
- **Graph is a disposable projection** for file-owned data — the file is truth, the graph rebuilds from scratch.
- **Clean rebuild** — lift concepts from v1 (`apps/base/`), port no code wholesale.

## Done When

Hand-authored TTL loads + validates against the `ops:` ontology; a sample SPARQL query returns the expected rows. (Operator's stated Phase 0 bar, unchanged.)

## Deferred (not Phase 0)

- Q#4 — CARL relationship (absorb vs. index as foreign source). Revisit at extraction layer (Phase 3).
- Q#7 — enumerate the third workspace (chris-ai-systems + chris-ai-socials known). Data, not architecture.
- Q#8 — dashboard rebuild against SPARQL. Phase 8, optional.

## Ready for

`/paul:plan` — turn this scope into the Phase 0 plan structure.
