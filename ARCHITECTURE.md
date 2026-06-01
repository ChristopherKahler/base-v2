# BASE v2 — Architecture Reference

> Captures the architecture worked out in the 2026-05-31 design session. Supersedes the seed brief's Open Questions 1–3 (resolved) and reframes 4 & 6. Source of truth for `/paul:plan` Phase 0.

**Status legend:** 🔒 LOCKED (Chris decided) · 🟡 RECOMMENDED (my lean, awaiting confirm) · ⚪ OPEN (undecided)

---

## 1. Headline thesis (reframed)

BASE v2 is **not** project management with a graph. It is a **proactive, deterministic context-injection engine** for Claude — an "all-seeing eye" that anchors on what Claude is touching (file, path, prompt, project) and injects only the salient slice of the operator's graph: rules, decisions, relations, history, gotchas. PM/CRUD is a supporting capability, not the headline.

The whole system lives or dies on one thing: **suppression, not detection.** Detection (fire a hook, match a path) is trivial. The product is the gate that stays silent until the one thing that matters changes. This is the brief's Phase 5 proof ("filtered injection beats the firehose").

⚪ **OPEN:** confirm this injection-engine framing as the headline (demote CRUD/PM to supporting). I recommend yes.

---

## 2. Locked architecture

| # | Decision | Status | Notes |
|---|----------|--------|-------|
| 1 | **Shared store, named graphs** (not per-workspace federated) | 🔒 | Realized physically as TTL-per-tier, unioned in-memory per call (§3) — gets federated's portability *and* shared's cross-tier query |
| 2 | **Interface = CLI, not MCP server** | 🔒 | One surface callable by Claude (Bash), hooks (settings.json), and the user (terminal). MCP serves only Claude. Near-zero standing context (no tool schemas) |
| 3 | **Language = Rust, single binary** | 🔒 | `base` = CRUD CLI + hook handler + embedded Oxigraph (Rust-native, no binding layer). Cleanest distribution, fastest hooks. Builder is Claude, so Chris's non-fluency in Rust is irrelevant |
| 4 | **Hooks call the CLI directly** | 🔒 | settings.json command = `base hook <event>`. No wrapper scripts. Binary has a `hook` mode: reads event JSON on stdin, writes injection/decision to stdout |
| 5 | **Hooks = reflexes; rules + CLI = judgment** | 🔒 | Deterministic/must-happen → hook. Semantic/judgment → Claude drives the CLI |

**Dissolved by the above:** seed Q2 (in-process vs MCP roundtrip — no server exists, each process loads text) and seed Q3 (Node vs Python — it's a Rust CLI, not an MCP server).

**Consequence to confirm —** ⚪ extraction logic (the fork's Python `extractor.py` etc.) ports to Rust, else Python re-enters as a runtime dep and the single-binary install is lost. Bounded work.

---

## 3. Persistence spine

**The TTL files *are* the store. No separate database.**

- Each tier is a TriG text file: `.base-gbl/graph.trig` (global), `{workspace}/.base/graph.trig` (per workspace). TriG = Turtle + named graphs → the three-tier named-graph model lives in the file layout.
- **Every `base` write** = short-lived process: load relevant TTL → apply SPARQL UPDATE → atomic write-back (temp + rename). Durable the instant the command returns. No save step, no long-running server, no in-memory-lost-on-crash.
- **Every `base` read** (incl. hooks) loads the file(s) it needs into in-memory Oxigraph in its own process, queries, exits.
- **Cross-tier query** = load global + N workspace files into one in-memory graph (single-digit ms at operator scale).
- **Git-sync is native** — TTL is text; `git pull` and the graph is there; file-derived parts rebuild via `base sync`. Nothing binary committed.

**Truth ownership:**
- File-owned entities (docs, `paul.json`, `domains.toml`) → the file is truth, graph is derived/rebuildable.
- BASE-owned entities (tasks, decisions via CLI) → the graph (TTL) is truth.

🟡 **Flavor B (TTL-is-the-store, above)** vs ⚪ **Flavor A (persistent on-disk Oxigraph/RocksDB + export TTL for git).** Lean B — simpler, git-native, trivial at scale. A only if profiling shows per-call parse cost biting (100k+ triples).

**Concurrency:** short-lived processes, sequential in practice; atomic write + tiny lockfile covers the rare overlap. The CLI model dissolves the earlier server-lock concern entirely.

---

## 4. End-to-end flows

### 4.1 Session start → "what should I focus on?"
1. SessionStart hook → `base hook session-start` → load workspace + global TTL → pre-filtered SPARQL (active + unblocked + high-priority + stale-but-important, ordered, LIMIT ~5) → print tight block to stdout → injected.
2. User asks → Claude answers from the injected top-5. Depth on demand via `base focus` / `base query`.
- Discipline: **tight summary injected; depth is a query, not standing context.**

### 4.2 Mid-stream building → how work stays logged
- File edits → PostToolUse → `base hook post-tool-use` records the **mechanical** fact (project touched @T, lastActive bumped). Silent, instant.
- Decision reached → Claude runs `base decision log …` (semantic; only Claude can recognize it; rule-nudged).
- Task done → `base task done <id>` (status mutates in place, prior fact retained as superseded).
- New entity → `base entity add` or PostToolUse proposes a candidate (propose, never auto-write).
- Session end → SessionEnd hook → integrity check + serialize + **threshold-gated** nudge ("12 edits, no semantic record — anything to log?"). Silent when nothing pending.

**Crux:** mechanical layer is automatic (hooks); high-value semantic layer (decisions, completions, status) depends on Claude's judgment + a small ruleset. That's the one real weakness — bounded by §4.3.

### 4.3 "Right amount" — floor / ceiling / safe band
- **Floor:** even with zero `base` calls from Claude, PostToolUse keeps activity/staleness fresh and `base sync` keeps file-owned data extracted. Graph never goes blind, just loses semantic richness. SessionEnd nudge catches misses.
- **Ceiling:** every write is IRI-keyed + idempotent → over-calling overwrites in place, never rots. Significance rules stop trivia.
- **Standing instructions (~15–20 lines):** mental model + when-to-write (significance-gated events only) + when-to-read (`base focus`/`query`) + discover via `base --help`. No tool schemas in context.

🟡 SessionEnd nudge: lean **yes** (backstop for missed judgment writes). ⚪ Per-turn (Stop hook) flush: avoid unless misses prove common (fires constantly).

---

## 5. CARL absorption

**CARL is already a vertical slice of BASE** — domain-matched query over a rule/decision store, injected on UserPromptSubmit, with a decision ledger. It's BASE's rule-and-decision layer, built separately in Python. `ops:Decision` already exists in the brief → CARL decisions and BASE decisions are the same entity stored twice (the duplication v2 exists to kill).

**Decision: absorb fully — data + mechanism.**
- **Data merge** — rules/decisions/domains become graph entities (`ops:Rule`, `ops:Decision`, `ops:Domain`); a rule's domain becomes an *edge* to project/workspace → "which rules apply to project X?" via traversal.
- **Mechanism merge** — the injection engine becomes `base hook user-prompt-submit` on the Rust binary; Python CARL retires.

**Rust on UserPromptSubmit = strongest Rust case in the system** — hottest path (every prompt), Python cold-start (~50–150ms) worst here, Rust (~1–5ms) wins biggest. Hard constraint: **fail-open** — on any error emit nothing, never block the prompt.

**Sequencing:** CARL is live and load-bearing → migrate it **last** (Phase ~6.5, pre-cutover), after graph/extraction/write/signal layers are proven. Don't replace working guardrails on an unproven engine.

**Costs:** faithful port (domain-matching, dedup signatures, rule priority, DEVMODE dashboard, PSMM coupling) + reconcile two overlapping taxonomies (CARL domains vs BASE workspaces/projects) into one model.

---

## 6. Deterministic matching + user-owned config

**Principle: take keyword authority away from Claude.** Matching happens in the hook (deterministic code reading a file), never in Claude's reasoning — which is why it's reliable. Claude mangles keyword lists; the file + matcher don't.

**`domains.toml`** (per tier, user-edited directly = the "simple file write"):
```toml
[casegate]
triggers = ["casegate", "legal intake", "intake app"]
paths    = ["apps/casegate/**"]
exclude  = ["casegate marketing"]
match    = "word"          # "substring" | "word" | "regex" — literal, predictable
sticky   = true            # once active, stays active for the session
rules = """
- NEVER expose client PII in logs
- ALWAYS validate intake against the schema first
"""
```
- File is truth (file-owned data pattern); `base sync` mirrors to graph for traversal. Hook reads the file directly for matching (fast, current).
- If Claude edits it: structured command `base domain add-trigger casegate "intake form"` appends the **exact** string — no embellishment.
- **No semantic/fuzzy matching** in the core — reintroduces the nondeterminism Chris rejects. Embeddings = later opt-in only.

**Multi-signal matcher (the robustness fix for "Claude didn't realize X has rules"):** active domains = union of deterministic facts, minus excludes:
1. **Prompt keywords** (CARL today — one signal, not the only one)
2. **Path / file match** — edit under `apps/casegate/**` activates casegate regardless of prompt text (the *filesystem* says X)
3. **Active-project edges** — graph knows active project → traverse project→domain→rules
4. **Sticky session** — once active by any signal, stays warm (deduped)

Robustness from *more facts*, not fuzzier matching. Anchor on the **edited file's path**, not process cwd (cwd barely moves). No native "cwd-changed" event — derive by comparing path-prefix to last-seen in session state.

---

## 7. The vision — proactive graph-neighborhood injection

A file edit is an **anchor into the graph**, not just a rule match. From `apps/casegate/src/auth/session.rs` traverse outward and pull everything connected, salience-filtered:
- Rules (selector-matched), Decisions about this module, Entities it touches, Notes/gotchas pinned to the path, cross-project links, recent history + reasoning.

Only a graph enables this (edges to traverse); flat keyword systems can't.

**Hook split:**
- **PreToolUse** (inject *before* the edit) — Claude gets the rules/decisions *before* writing → writes it right the first time. This is where the proactive magic lives.
- **PostToolUse** (after) — capture edges + flag violations.

**Tiered evaluation** (keeps the hot path fast): <1ms path gate → only then content-scan + graph-traversal. Another Rust justification.

**Suppression layer (the actual product):** dedup (never re-inject what's in context) · novelty gate (inject auth rules once on entering the context, not per edit) · salience threshold (top ~3) · budget cap. Without this, the all-seeing eye is just a new firehose.

**Make-or-break:** the authoring experience — how trivially a user pins a rule/decision/note to a path/pattern. Garbage selectors = blind eye; precise selectors = the vision.

⚪ **OPEN:** path/active-project triggers core to v1, or keyword-only v1 with path/project added later?

---

## 8. Open decisions (carry into Phase 0 planning)

| # | Question | Lean |
|---|----------|------|
| A | Persistence Flavor B (TTL-is-store) vs A (binary store) | B |
| B | SessionEnd threshold nudge as missed-write backstop | Yes |
| C | Per-turn (Stop) flush vs per-session only | Per-session only |
| D | Extraction ported to Rust (single-binary purity) | Yes |
| E | CARL path/active-project triggers in v1 vs later | TBD |
| F | Injection-engine as headline thesis (demote PM) | Yes |

---

## 9. Process notes / corrections made this session

- The open-ontologies fork's **MCP server is Rust + in-memory Oxigraph** (`graph.rs` `Store::new()`, `rmcp`), not Python + RocksDB. It ships Python utility scripts (`extractor.py`, `obsidian_sync.py`) — bilingual. Earlier "RocksDB persistence" reasoning was from Oxigraph's typical config, not the actual code.
- "Suppress ~46 onto tools" is an **open question** in the brief (`README.md:136`, `PLANNING.md:248`), **not a locked decision** — was earlier mischaracterized as settled.
- Rust-vs-language conclusion **flipped correctly** when premises changed: CLI (not server) + builder-is-Claude (not Chris) → Rust single binary, not Python.

---
*ARCHITECTURE.md — created 2026-05-31. Feeds `/paul:plan` Phase 0.*
