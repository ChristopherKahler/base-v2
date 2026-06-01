# Phase 6 — CARL Absorption: Discussion Context

**Created:** 2026-06-01 12:23
**Source:** /paul:discuss session

---

## Goals

### 1. Merge CARL into BASE
CARL becomes a feature of the BASE Rust binary, not a separate Python tool. The separation was the wrong architecture from the start. Design so CARL's functionality *could* theoretically extract later, but the primary path is one unified binary.

### 2. `domains.toml` = Trigger Config Only
Two keyword types in `domains.toml`:
- **Prompt keywords** — user-configured, natural language oriented. Users know the terms they use. Matched on `user-prompt-submit`.
- **File keywords** — code-oriented, system-suggestable. More specific (imports, module names, patterns). Matched on `pre/post-tool-use` when a file is involved.

Paths, sticky, exclude, always-on stay in `domains.toml`. Rule *content* moves out of TOML and into the graph as queryable entities with edges.

### 3. Graph-Backed Rule Injection with Relational Context
Domain match → SPARQL fetches rules AND relational neighborhood (edges, entities, decisions, related concepts). The whole point of ontology is the relational aspect. Surgical precision AND comprehensive context simultaneously.

Two-layer architecture:
- **TOML** = coarse triggers (globs, keywords) — operator-configured, stable
- **Graph** = fine-grained relationships (file → concept → rule → decision) — machine-maintained via extraction

### 4. Pre-Tool-Use File Matching
Post-tool-use hook sees file path + scans file for keywords → loads relevant domain rules + related graph context. Example: editing `src/auth/middleware.rs` in CaseGate → path matches CaseGate domain → file keywords match auth concept → graph traversal surfaces CaseGate rules + auth-specific rules + related security decisions.

Extraction layer (Phase 4, already built) pre-indexes file→concept edges so hooks do graph lookups at runtime, not file reads. File scan is extraction-time, not hook-time.

### 5. DEVMODE Dashboard
Implement early alongside domain matching. Operational telemetry block showing what was matched, what was injected, and why. Core operational feature of BASE, not CARL-specific.

### 6. Context Brackets
Session context awareness. Tracks session depth/state and adjusts injection behavior accordingly (e.g., LEAN mode for fresh sessions, heavier injection when deep in a domain).

### 7. `base learn`
Graph-backed structured memory system replacing PSMM. Also replaces Claude's native `/memory` system. Notes are entities with edges — they relate to projects, domains, people, decisions. Recall is relational (graph traversal), not keyword search over flat files.

### 8. Retire Python CARL Hooks
End state: Python CARL hooks are removed. All functionality lives in the Rust `base` binary.

---

## Approach

- Prompt-keywords trigger on `user-prompt-submit` hook
- File-keywords trigger on `pre/post-tool-use` hook (file path + content signal)
- Extraction layer pre-indexes file→concept edges (already built in Phase 4)
- `domains.toml` path globs handle directory tagging (no co-located config files this phase)
- Rules stored as graph entities with edges to domains, concepts, projects
- SPARQL queries traverse 1-2 hops from matched domain to surface neighborhood

---

## Open Questions (For Planning)

- **`base learn` CLI interface** — `base learn "X relates to Y"` vs structured subcommands vs freeform text that gets entity-extracted
- **DEVMODE output format** — redesign from current Python CARL block, or faithful port first then iterate
- **Context bracket implementation** — session-scoped state machine in the binary; how bracket transitions trigger
- **Migration path** — port `carl.json` content into the graph, or fresh start with new domain configs
- **Rule priority** — how priority ordering works when multiple domains match (current CARL has numeric priority)
- **Dedup signatures** — port CARL's dedup logic or redesign against graph-native dedup

---

## Deferred (Not This Phase)

- Directory-scoped `.base.toml` config files (path globs in `domains.toml` suffice for now)
- Auto-suggestion of file keywords (Claude proposes, user approves)
- `base learn` advanced features (cross-workspace recall)
- CARL as standalone extractable tool (design for it, don't build for it)

---

## Prior Art

Phase 2 already built:
- `domains.toml` parser with keyword/path/exclude/sticky/always-on triggers
- Multi-signal matcher (prompt keywords + file paths + active-project edges)
- Session dedup via `.base/.session` with rules-hash
- `base hook user-prompt-submit` → match → inject rules to stdout

Phase 6 extends this by moving rule content into the graph and adding file-keyword matching, DEVMODE, context brackets, and `base learn`.

---

*Context for handoff to /paul:plan*
