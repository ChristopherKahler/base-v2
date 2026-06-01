# BASE v2 — Proactive Context Engine

> A ground-up rebuild of BASE that replaces v1's JSON-file + MCP-CRUD model with a proactive, deterministic context-injection engine backed by an RDF knowledge graph. The operator's context becomes one queryable relationship graph, surfaced through a Rust CLI that hooks call directly.

**Type:** Application (framework / internal tooling)
**Stack:** Rust single binary · embedded Oxigraph (in-memory, TTL-persisted) · SPARQL query layer · Claude Code hooks (stdin/stdout) · TOML config · markdown/TOML authoring surface
**Quality Gates:** idempotent extraction (zero dup/drift) · SPARQL query correctness · hook latency budget (<5ms) · lossless v1 migration

> **v1 reference (do not edit in place):** `apps/base/` · v1 data: `.base/data/*.json`
> **Architecture decisions:** `ARCHITECTURE.md` (source of truth for all resolved design questions)
> **Full ideation:** `planning/base-v2/PLANNING.md`

---

## Headline Thesis

BASE v2 is **not** project management with a graph. It is a **proactive, deterministic context-injection engine** for Claude — an "all-seeing eye" that anchors on what Claude is touching (file, path, prompt, project) and injects only the salient slice of the operator's graph: rules, decisions, relations, history, gotchas. PM/CRUD is a supporting capability, not the headline.

The system lives or dies on **suppression, not detection.** Detection (fire a hook, match a path) is trivial. The product is the gate that stays silent until the one thing that matters changes.

---

## Architecture

**Interface = CLI, not MCP server.** One surface callable by Claude (Bash), hooks (settings.json), and the user (terminal). Near-zero standing context (no tool schemas loaded).

**Language = Rust, single binary.** `base` = CRUD CLI + hook handler + embedded Oxigraph (Rust-native). Cleanest distribution, fastest hooks (~1-5ms vs Python's ~50-150ms cold-start). Builder is Claude, so operator's non-fluency in Rust is irrelevant.

**Hooks call the CLI directly.** `settings.json` command = `base hook <event>`. No wrapper scripts. Binary has a `hook` mode: reads event JSON on stdin, writes injection/decision to stdout.

See `ARCHITECTURE.md` for the full design rationale.

### Three tiers = named graphs

| Tier | Named graph | Store | Owns |
|------|-------------|-------|------|
| User / global | `ops:graph/global` | `.base-gbl/graph.trig` | Operator profile, goals, cross-workspace entities, shared vocabulary |
| Workspace | `ops:graph/ws/{slug}` | `{workspace}/.base/graph.trig` | Projects, tasks, reminders, decisions, knowledge docs |
| Project | folded into workspace graph | — | PAUL project state, indexed read-only |

### Persistence: TTL is the store

Each tier is a TriG text file. Every `base` write = short-lived process: load relevant TTL → apply SPARQL UPDATE → atomic write-back (temp + rename). Every read (incl. hooks) loads file(s) into in-memory Oxigraph, queries, exits. No long-running server, no in-memory-lost-on-crash. Git-sync is native — TTL is text.

### Two write paths, one graph

1. **CLI commands (imperative)** — BASE-owned mutable state (projects, tasks, decisions, entities, goals). Mutations are SPARQL UPDATE scoped to the entity IRI, idempotent, supersession-preserving.
2. **Extraction (declarative)** — file-owned data. Markdown frontmatter → triples (MOP); `paul.json` → triples. Each scan clears that node's subgraph and re-emits from the current file.

### Domain matching

Deterministic matching via user-edited `domains.toml` — keywords, path globs, excludes, sticky sessions. Multi-signal: prompt keywords + edited file paths + active-project edges. No semantic/fuzzy matching in the core. See `ARCHITECTURE.md` §6.

---

## Data Model

| Class | Source of truth | Write path |
|-------|-----------------|------------|
| `ops:Project` | BASE | CLI |
| `ops:Task` | BASE | CLI |
| `ops:Decision` | BASE | CLI |
| `ops:Entity` (Person, Org) | BASE | CLI |
| `ops:Goal` | BASE | CLI |
| `ops:Reminder` | BASE | CLI |
| `ops:Workspace` | init-workspace | CLI |
| `ops:Domain` | `domains.toml` | Extraction |
| `ops:Rule` | `domains.toml` | Extraction |
| `ops:Document` | markdown file | Extraction |
| `ops:PaulProject` | `.paul/paul.json` | Extraction (read-only) |

**IRI scheme is load-bearing** — stable identity (path + slug), never generated ids. Same entity referenced N places = one node.
**Supersession is native** — retire via `ops:status superseded` + timestamp; never silent deletion.

---

## Implementation Phases

| # | Phase | "It works" |
|---|-------|-----------|
| 0 | Rust Scaffold + Ontology | Binary compiles; hand-authored TTL loads; sample SPARQL returns expected rows |
| 1 | Hook Engine | Session starts produce filtered injection; file edits update lastActive |
| 2 | Domain Matcher + Rule Injection | Replaces CARL's Python matching on UserPromptSubmit |
| 3 | Write Commands (CRUD) | CLI manages entities; TTL updates atomically |
| 4 | Extraction Layer | Re-scan = identical graph; edit file → stale fact gone |
| 5 | Signal Layer (proof) | Filtered injection beats v1 firehose on `active-awareness` |
| 6 | CARL Absorption | Python CARL retires; all injection through Rust binary |
| 7 | v1 Migration + Cutover | All v1 records in graph; dead JSON unreferenced |
| 8 | Dashboard (optional) | Operator dashboard reads from SPARQL |

---

## Design Decisions

1. **Clean rebuild, not refactor** — lift concepts from v1, port nothing wholesale.
2. **Injection-engine headline** — PM/CRUD supports; proactive context injection is the product.
3. **Rust single binary** — CLI + hook handler + embedded Oxigraph. No wrapper scripts, no runtime deps.
4. **CLI, not MCP server** — one surface for Claude, hooks, and user. Zero standing context cost.
5. **TTL-is-the-store** — TriG text files, no separate database. Git-native, trivial at operator scale.
6. **Hooks call CLI directly** — `base hook <event>` reads stdin, writes stdout. Fail-open on error.
7. **Deterministic matching** — `domains.toml` keyword/path/exclude, no fuzzy/semantic matching in core.
8. **CARL absorbed fully** — rules, decisions, domains, matching become graph entities; Python CARL retires.
9. **Suppression layer is the product** — dedup, novelty gate, salience threshold, budget cap.
10. **Three tiers as named graphs** — shared vocabulary global, instances per-tier, cross-tier edges.
11. **Two write paths** — CLI (imperative, BASE-owned) + extraction (declarative, file-owned).
12. **Pre-filtered SPARQL in hooks** — relevance is a query, not a firehose.

---

## References

- Architecture decisions: `ARCHITECTURE.md`
- v1: `apps/base/` (src/, data model, hooks)
- v1 data shape: `.base/data/*.json`, `.base/workspace.json`
- Ideation: `planning/base-v2/PLANNING.md`

---
*Last updated: 2026-05-31*
