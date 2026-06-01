# base-v2

## What This Is

A ground-up rebuild of the BASE workspace-orchestration framework. v2 replaces v1's JSON-file + MCP-CRUD data model with a proactive, deterministic context-injection engine backed by an RDF knowledge graph (Oxigraph). Implemented as a Rust single binary (`base`) that Claude Code hooks call directly via stdin/stdout — no MCP server, no wrapper scripts, near-zero standing context cost.

## Core Value

The operator's entire context — projects, tasks, people, tools, frameworks, decisions — becomes one queryable relationship graph. The product is not the graph itself but the **suppression layer**: injecting only the salient slice of context anchored on what Claude is touching, and staying silent the rest of the time.

## Current State

| Attribute | Value |
|-----------|-------|
| Type | Application (framework / internal tooling) |
| Version | 0.0.0 |
| Status | Building |
| Last Updated | 2026-06-01 |

## Requirements

### Core Features

- **Rust single binary** — CLI + hook handler + embedded Oxigraph. `base hook <event>` reads stdin, writes stdout. Fail-open on error.
- **TTL-is-the-store** — TriG text files per tier (`.base-gbl/graph.trig`, `{workspace}/.base/graph.trig`). No separate database. Git-native.
- **Three-tier governance** — user/global, workspace, project — each a named graph under shared `ops:` vocabulary, with cross-tier edges.
- **Two write paths, one graph** — CLI commands (imperative, BASE-owned) + extraction (declarative, file-owned + foreign).
- **Deterministic domain matching** — `domains.toml` with keyword/path/exclude triggers. Multi-signal matcher (prompt + path + active-project + sticky). No fuzzy/semantic matching.
- **PAUL indexed read-only** — JSON-extraction plugin maps `paul.json` → triples; idempotent, IRI-keyed; BASE never writes PAUL.
- **Pre-filtered signal hooks** — session-start injection runs parameterized SPARQL, surfaces only relevant rows (kills v1 firehose).
- **Suppression layer** — dedup, novelty gate, salience threshold, budget cap. The actual product.
- **CARL absorption** — rules, decisions, domains become graph entities; Python CARL retires.

### Validated (Shipped)
- ✓ **Rust single binary** — CLI + hook handler + embedded Oxigraph — Phase 0
- ✓ **TTL-is-the-store** — TriG text files per tier, atomic write-back — Phase 0
- ✓ **Pre-filtered signal hooks** — session-start runs parameterized SPARQL, configurable queries — Phase 1
- ✓ **Hooks fail-open** — errors → stderr, exit 0, empty stdout — Phase 1
- ✓ **Namespace portability** — prefix + URI runtime-configurable via base.toml — Phase 1
- ✓ **Deterministic domain matching** — domains.toml with keyword/path/exclude/sticky triggers, multi-signal matcher — Phase 2
- ✓ **Two write paths** — CLI commands (CRUD) for 6 entity types via SPARQL UPDATE, IRI-scoped — Phase 3
- ✓ **Idempotent extraction** — `base sync` with frontmatter + paul.json extractors, mtime incremental, configurable patterns — Phase 4
- ✓ **Suppression layer** — 3 signal types with cross-session hash dedup, budget cap, priorities — Phase 5

### Active (In Progress)
Phase 6 (CARL Absorption) — port remaining CARL mechanisms, retire Python hooks.

### Planned (Next)
Phase 7 (v1 Migration + Cutover) — JSON→triples transform, retire v1.

### Out of Scope
- BASE writing PAUL or other foreign framework state — read-only indexing only.
- Web frontend — surfaces are injected context + optional Obsidian export.
- Semantic/fuzzy matching in the core domain matcher — deterministic only.
- MCP server — interface is CLI, not MCP.

## Constraints

### Technical Constraints
- Extraction MUST be idempotent and keyed on stable IRIs (path/slug), never generated ids — structural guarantee against v1's duplicate-node rot.
- Hook latency budget: <5ms for the hot path (Rust justification).
- Local, single-operator; no web threat model. Integrity (not auth) is the security concern.
- Hooks fail-open — on any error emit nothing, never block the prompt.

### Business Constraints
- Dogfood-first (operator's own workspaces), then package for other Claude Code operators.
- Replaces v1 (`apps/base/`) — not a parallel system; v1 retires at cutover.
- CARL is live and load-bearing → absorb last (after graph/signal layers are proven).

## Key Decisions

| Decision | Rationale | Date | Status |
|----------|-----------|------|--------|
| Ontological replacement, not augmentation | Data is already a relationship graph; RDF makes relationships first-class | 2026-05-29 | Active |
| Three tiers as named graphs | Shared vocabulary global, instances per-tier, cross-tier edges | 2026-05-29 | Active |
| Two write paths (CLI imperative + extraction declarative) | Separates BASE-owned mutable state from file-owned/foreign state | 2026-05-29 | Active |
| PAUL indexed into graph, read-only | Write-ownership ≠ graph-presence; buys full cross-graph queryability | 2026-05-29 | Active |
| IRI-keyed idempotent extraction | Structurally eliminates v1 duplicate-satellite + stale-fact rot | 2026-05-29 | Active |
| Pre-filtered SPARQL in hooks | Relevance is a query, not a firehose | 2026-05-29 | Active |
| Injection-engine headline (demote PM) | System lives on suppression, not detection; PM/CRUD is supporting | 2026-05-31 | Active |
| Rust single binary | Fastest hooks (~1-5ms), cleanest distribution, Claude builds it | 2026-05-31 | Active |
| CLI, not MCP server | One surface for Claude/hooks/user; zero standing context cost | 2026-05-31 | Active |
| TTL-is-the-store (Flavor B) | Git-native, trivial at operator scale, no binary blobs | 2026-05-31 | Active |
| Hooks call CLI directly | `base hook <event>` stdin/stdout; no wrapper scripts | 2026-05-31 | Active |
| CARL absorbed fully into BASE v2 | Rules/decisions/domains become graph entities; Python CARL retires | 2026-05-31 | Active |
| Namespace URI + prefix runtime-configurable | Product is for other operators — nothing hardcoded to Chris's setup | 2026-06-01 | Active |
| SPARQL queries operator-configurable | queries.toml with tiered override (embedded → global → workspace) | 2026-06-01 | Active |
| Session-end nudge dropped | Hook unreliable (users close VS Code); nudge via signal layer instead | 2026-06-01 | Active |
| No dirty-file tracking | Graph self-manages via mtime vs ops:lastExtracted; zero external state | 2026-06-01 | Active |
| Session dedup via .base/.session with rules-hash | Bounded, self-cleaning, detects rule changes for re-injection | 2026-06-01 | Active |
| domains.toml TOML format with [[domain]] tables | Clean, operator-readable, same tiered-override pattern as queries.toml | 2026-06-01 | Active |

## Success Metrics

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Hook latency | <5ms (session-start, user-prompt-submit) | - | Not started |
| Extraction idempotency | Re-scan = identical graph (zero dup/drift) | - | Not started |
| Signal relevance | Filtered injection beats v1 firehose on noise | - | Not started |
| Migration fidelity | 100% of v1 records represented in graph | - | Not started |

## Tech Stack / Tools

| Layer | Technology | Notes |
|-------|------------|-------|
| Language | Rust | Single binary, no runtime deps |
| Graph store | Oxigraph (embedded, in-memory) | Loaded from TTL per invocation; Rust-native |
| Persistence | TriG text files | `.base-gbl/graph.trig`, `{workspace}/.base/graph.trig` |
| CLI framework | clap | Subcommands + hook mode |
| Serialization | serde / serde_json | Event parsing, structured output |
| Config | toml crate | `base.toml`, `queries.toml`, `domains.toml` |
| Time | chrono | ISO 8601 timestamps for lastActive |
| Paths | dirs | Cross-platform home directory resolution |
| Query | SPARQL / SPARQL UPDATE | Open query surface; hooks run pre-filtered reads |
| Hooks | Claude Code settings.json | `base hook <event>` — direct binary call |
| Authoring | Markdown frontmatter + TOML | Human-writable source of truth; graph is derived |

## Links

| Resource | URL |
|----------|-----|
| Architecture | `ARCHITECTURE.md` |
| Ideation | `planning/base-v2/PLANNING.md` |
| v1 reference | `apps/base/` |

---
*PROJECT.md — Updated when requirements or context change*
*Last updated: 2026-06-01 — Phase 5 (Signal Layer) complete*
