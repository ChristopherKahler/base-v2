# BASE — Proactive Context Engine for Claude Code

> A Rust single binary that hooks into Claude Code and injects only the context that matters. Rules, decisions, notes, and project state live in an RDF knowledge graph. The system matches on what you're touching (files, prompts, projects) and surfaces the relational neighborhood. It stays silent until something changes.

**Built by Chris Kahler · Chris AI Systems**
**Community & support:** https://chrisai.cv/skool
**Tutorials:** https://www.youtube.com/@chris-ai-systems

---

## Install

```bash
# Clone and build
git clone <repo-url>
cd base-v2
cargo build --release

# Run the installer (copies binary, creates global config, wires hooks, updates CLAUDE.md)
./target/release/base install
```

That's it. `base install` handles:
1. Copies binary to `~/.local/bin/base`
2. Creates `~/.base-gbl/` with default configs (devmode, brackets, signals)
3. Wires all 4 hooks in `~/.claude/settings.json`
4. Appends BASE CLI section to `~/.claude/CLAUDE.md` (Claude knows the tools exist)

## Scaffold a Workspace

```bash
cd ~/my-workspace
base scaffold
```

Creates `.base/` with workspace-specific config, registers the workspace globally. Every registered workspace gets scanned for projects on session start.

---

## How It Works

### Hooks (automatic, every session)

| Hook | When | What |
|------|------|------|
| **SessionStart** | New session opens | Syncs domains, ingests paul.toml projects, runs signals (pulse, staleness, active-awareness) |
| **UserPromptSubmit** | You type a prompt | Matches keywords against domains, injects rules + decisions + notes from graph |
| **PreToolUse** | Claude is about to edit a file | Matches file path against domain triggers, injects rules BEFORE the edit |
| **PostToolUse** | Claude finishes editing | Updates lastActive timestamps in graph |

### CLI (proactive, called by Claude or you)

```bash
# Rules — graph-native, not in config files
base rule add --domain DEVELOPMENT --text "Always use motion, not framer-motion"
base rule list --domain GLOBAL
base rule remove --domain GLOBAL --index 3

# Memory — structured notes with relational edges
base learn --text "JWT is the auth pattern for this project" --domain CASEGATE --type decision
base recall --keyword "auth" --domain CASEGATE

# Decisions
base decision log --domain GLOBAL --decision "Use Rust for hot path" --rationale "Performance critical"
base decision search --keyword "auth"

# Projects — auto-creates domain trigger for file matching
base project add --name "my-app" --path "apps/my-app" --status active
base project list

# Tasks, Entities, Goals, Reminders
base task add --project my-app --name "Fix auth flow"
base entity add --name "Charlie" --type person --domain CASEGATE
base goal add --name "Ship v1" --target "2026-07-01"
base reminder add --name "Review PR" --due 2026-06-05
```

---

## Architecture

### Two-Layer Config

**`domains.toml`** = triggers only (keywords, file paths, exclude patterns)
```toml
[[domain]]
name = "DEVELOPMENT"
mode = "triggered"
prompt_keywords = ["write code", "fix bug", "implement"]
file_keywords = ["use crate", "import", "fn main"]
paths = ["src/"]
```

**Graph** = content (rules, decisions, notes, entities with relational edges)

Triggers fire the match. The graph provides the context. SPARQL queries traverse 1-2 hops from the matched domain to surface the neighborhood.

### Three Tiers

| Tier | Config | Graph | Scope |
|------|--------|-------|-------|
| Global | `~/.base-gbl/base.toml` | `~/.base-gbl/graph.trig` | All workspaces |
| Workspace | `{ws}/.base/base.toml` | `{ws}/.base/graph.trig` | One workspace |
| Project | `.paul/paul.toml` | Ingested into workspace graph | One project |

### Context Brackets

Session depth tracking: FRESH (lean injection) → MODERATE (full injection) → DEPLETED (force-refresh dedup) → CRITICAL. Configurable thresholds in `base.toml`.

### DEVMODE

When enabled, every prompt injection includes a telemetry block: which domains loaded, which were deduped, bracket state, available domains. Operator reads this to tune the system.

### Mandatory Edges

Every entity-creating command requires a domain edge. No orphans in the graph:
- `base learn` requires `--domain`
- `base entity add` requires `--domain`
- `base project add` requires `--path` (auto-creates domain trigger)
- `base decision log` requires `--domain`
- `base rule add` requires `--domain`

---

## Workspace Registry

Registered in `~/.base-gbl/base.toml`:

```toml
[[workspace]]
path = "/home/user/my-workspace"

[[workspace]]
path = "/home/user/another-workspace"
```

Session start scans all registered workspaces for `paul.toml` project manifests and ingests them into the graph automatically.

---

## Stack

- **Language:** Rust (single binary, ~16MB release)
- **Graph:** Embedded Oxigraph (in-memory, TriG-persisted)
- **Query:** SPARQL SELECT/UPDATE
- **Config:** TOML (domains.toml, base.toml, paul.toml)
- **Persistence:** TriG text files (git-native, atomic write-back)
- **Hooks:** stdin JSON → stdout injection, fail-open on all errors

---

## Design Principles

1. **Suppression, not detection.** Detection is trivial. The product is the gate that stays silent until the one thing that matters changes.
2. **CLI, not MCP server.** One surface for Claude, hooks, and the user. Zero standing context cost.
3. **Graph is the query layer, TOML is the authoring surface.** Operators edit TOML, the system queries the graph.
4. **Fail-open.** Every hook catches all errors, logs to stderr, exits 0 with empty stdout. Never block Claude.
5. **Mandatory edges.** No orphan entities. Every node connects to the relational fabric.
6. **Deterministic matching.** Keywords, paths, excludes. No fuzzy/semantic matching in the core.

---

*Built by Chris Kahler · Chris AI Systems · https://chrisai.cv/skool*
