# BASE

The intelligence layer Claude Code doesn't have.

Your codebase, your projects, your people, your decisions, your business - mapped into one ontological graph and wired directly into Claude's hook pipeline. When Claude touches a file, it already knows what that file contains, what calls it, and what depends on it. When Claude greps for code, the graph intercepts and says "I already know where that is." When you ask about a project, it knows the milestones, the tasks, who's involved, and what decisions led to the current state.

This isn't a CLAUDE.md file. It isn't a prompt you paste. It's a live knowledge graph that tracks everything across every workspace and injects exactly the right slice of context at exactly the right moment - then goes silent until something changes.

Your code. Your projects. Your people. Your decisions. One graph. Every session. Every agent. Automatic.

```
base p l                          # list your projects
base a q -c "auth"                # find any entity named "auth" in the graph
base t a -p myapp -n "Fix login"  # add a task to a project
```

You can run these yourself in a Claude Code session with `!` prefix, or Claude runs them automatically through hooks. Same binary, same graph, same data.

## Why this exists

Claude Code is the best coding agent on the planet. But it starts every session blind. It doesn't know your codebase structure. It doesn't know your active projects. It doesn't know the decision you made last week about auth patterns. It doesn't know your teammate is blocked on the API. It doesn't know what files depend on the module you're about to change.

CLAUDE.md helps. But it's a static document you wrote once. It doesn't know what you're touching right now. It doesn't adapt. It doesn't suppress itself when irrelevant.

BASE is the layer underneath - the one that turns Claude Code from a coding assistant into a business-aware operating system. It maps your entire operation into an ontological graph (code structure, projects, milestones, tasks, people, decisions, domain rules) and wires that graph into every hook in the pipeline. Session start, prompt submit, pre-tool-use, post-tool-use - at every stage, the graph surfaces what's relevant and suppresses what isn't.

Every agent gets it. Main session, subagents, explore agents, workflow agents - they all inherit the same hooks, the same graph, the same intelligence. Your whole fleet sees the same map.

You can run these yourself in a Claude Code session with `!` prefix, or Claude runs them automatically through hooks. Same binary, same graph, same data.

## What the graph knows

BASE maps three layers into one queryable graph:

**Your code** - every function, struct, class, import, and call relationship across 35+ languages. Tree-sitter extracts the structure, SPARQL makes it queryable. "What calls this function?" is a graph query, not a grep.

**Your business** - projects with milestones and tasks. People and their roles. Decisions and the rationale behind them. Goals and deadlines. Domain rules that fire when you're working in the right context. This is the stuff that lives in your head or in scattered docs - BASE puts it in the graph where every agent can reach it.

**Your operations** - which domains are active, what rules apply where, which files are stale, what changed since last session. Signals that fire at session start to orient Claude before you type a word.

All of it is relational. A project has milestones. A milestone has tasks. A task touches files. Those files contain functions. Those functions call other functions in other files that belong to other projects. One graph. Cross-cutting. Queryable.

## What this looks like in practice

You open a file. Before the content even loads, the pre-tool-use hook fires and injects:

```
[AST] auth.rs - 8 entities
  Key: fn login (line 12), fn validate_token (line 45), struct AuthConfig (line 3)
  Imports: config.rs, database.rs
  Imported by: api.rs, middleware.rs, cli.rs
```

You didn't ask for that. You didn't type a command. The hook saw the file path, queried the graph, and told you the shape of what you're about to read. Now you know there are 8 things in this file, what they are, and how they connect to the rest of the codebase. Before reading a single line of code.

You read lines 45-80. Post-tool-use fires:

```
[AST] Lines 45-80: fn validate_token
  Calls: decode_jwt, check_expiry, load_user
  Called by: middleware.rs -> require_auth()
```

Just for those 35 lines. Not the whole file. Not a report. The exact context for the exact code you're looking at.

You reach for grep. The hook intercepts:

```
<ast-hint>
AST graph available for this workspace. Try:
  base ast query --contains "validate_token"
The graph knows file locations, line numbers, and call relationships.
</ast-hint>
```

One SPARQL query instead of scanning 15 files.

## How this compares

**LSP** tells you where a symbol is defined. You ask, it answers. One language at a time. No project context, no session memory, no idea what you've already looked at.

**Graphify** (58k stars) extracts your code into a JSON graph and generates a report. You query it by typing `/graphify query "question"` and an LLM interprets your question against the JSON. That's inference tokens spent on a database lookup. And you have to remember to ask every time.

**BASE** doesn't wait for you to ask.

|  | LSP | Graphify | BASE |
|---|---|---|---|
| How you ask | Point at a symbol | Type a query | You don't - it injects on file touch |
| Query engine | Language server | LLM interprets JSON | SPARQL - deterministic, zero inference cost |
| What it knows | Symbols in one language | Code across languages | Code + projects + tasks + decisions + people |
| When it speaks | When asked | When asked | When relevant. Silent otherwise. |
| Section awareness | Symbol-level | None | Knows the 40 lines you just read |
| Session memory | None | None | Tracks what it injected, never repeats |
| Subagent support | None | Manual | Automatic - all agents inherit hooks |

The extractor that builds the graph is the same tree-sitter parser Graphify uses (we forked it). What happens after extraction is completely different. Graphify dumps JSON and generates a report. BASE loads triples into an ontological graph and wires SPARQL queries into the hook pipeline so the data flows automatically at the moment it matters.

## Quick start

```bash
# build
git clone <repo-url>
cd base-v2
cargo build --release

# install (copies binary, creates config, wires hooks)
./target/release/base install

# scaffold your workspace
cd ~/my-workspace
base scaffold
```

That's three commands. `base install` handles everything: binary goes to `~/.local/bin/base`, global config goes to `~/.base-gbl/`, hooks get wired into `~/.claude/settings.json`, and your CLAUDE.md gets a CLI reference section so Claude knows the commands exist.

`base scaffold` creates `.base/` in your workspace with domains.toml and default config.

## Quick win - see it work in 5 minutes

After install, try this sequence:

**1. Add a project**
```bash
base p a -n "My App" -p "src"
```

**2. Extract the codebase structure**
```bash
base sync --ast --target src
```

This runs tree-sitter across your source files (supports 35+ languages) and loads every function, struct, class, import, and call relationship into the graph.

**3. Query the graph**
```bash
# what's in a specific file?
base a q -f "main.rs"

# where does a function live?
base a q -c "handle_request"

# what imports from this module?
base a q -i "database.rs"

# who calls this function?
base a q --calls "validate"
```

**4. Open a Claude Code session in your workspace**

Read any source file. Watch the AST map appear automatically in the hook output. That's the graph doing its job without you asking.

**5. Run commands inside your Claude session**

Type `!` followed by any base command:

```
! base p l                    # see your projects
! base a q -c "auth"          # find entities
! base t a -p myapp -n "Todo" # add a task
! base m a -p myapp -n "MVP"  # add a milestone
```

You manage your projects, tasks, milestones, and codebase graph from inside the same session where Claude is working. No switching tools. No separate dashboard.

## Command reference

Every command has a short alias. Long flags have short versions too.

### Projects (initiative level)

```bash
base project list                            # or: base p l
base project add --name "X" --path "src/x"   # or: base p a -n "X" -p "src/x"
base project update myapp --status blocked    # or: base p u myapp -s blocked
base project get myapp                       # accepts slug OR display name
```

### Milestones (epic level)

```bash
base milestone list --project myapp          # or: base m l -p myapp
base milestone add --project myapp \
  --name "MVP" --description "Core features" # or: base m a -p myapp -n "MVP" -d "Core features"
base milestone update myapp.mvp --status done # or: base m u myapp.mvp -s done
```

### Tasks

```bash
base task list                               # or: base t l
base task list --project myapp               # or: base t l -p myapp
base task list --milestone myapp.mvp         # or: base t l -m myapp.mvp
base task add --project myapp --name "Fix X" # or: base t a -p myapp -n "Fix X"
base task add --project myapp --name "Fix X" \
  --milestone myapp.mvp                      # or: base t a -p myapp -n "Fix X" -m myapp.mvp
base task done myapp.fix-x                   # mark complete
```

### AST graph queries

```bash
base ast query --contains "auth"             # or: base a q -c "auth"
base ast query --file "main.rs"              # or: base a q -f "main.rs"
base ast query --imports "config.rs"         # or: base a q -i "config.rs"
base ast query --calls "handle_request"      # find callers
```

### Decisions, rules, memory

```bash
# decisions with rationale (base d)
base decision log --domain myapp \
  --decision "Use Postgres" --rationale "Team knows it"  # or: base d log ...
base decision search --keyword "database"                 # or: base d search ...

# rules live in the graph, not config files
base rule add --domain DEVELOPMENT --text "Always validate inputs at boundaries"
base rule list --domain GLOBAL

# structured memory with relational edges
base learn --text "Auth uses JWT with 15min expiry" --domain myapp --type decision
base recall --keyword "auth"
```

### Entities, goals, reminders

```bash
base entity add --name "Alice" --type person --domain myapp  # or: base e a ...
base entity list                                              # or: base e l

base goal add --name "Ship v1" --target "2026-07-01"          # or: base g a ...
base goal list                                                # or: base g l

base reminder add --name "Review PR" --due 2026-06-10         # or: base r a ...
base reminder list                                            # or: base r l
```

### All command aliases

| Full command | Short form |
|---|---|
| `base project` | `base p` |
| `base milestone` | `base m` |
| `base task` | `base t` |
| `base ast` | `base a` |
| `base decision` | `base d` |
| `base entity` | `base e` |
| `base goal` | `base g` |
| `base reminder` | `base r` |

Subcommands: `add` = `a`, `list` = `l`, `update` = `u`, `query` = `q`

### Sync and AST extraction

```bash
base sync                        # extract markdown frontmatter + paul.toml into graph
base sync --incremental          # only changed files
base sync --ast                  # extract code structure (tree-sitter, 35+ languages)
base sync --ast --target src     # target a specific directory
```

## Hierarchy: projects, milestones, tasks

Three levels. Projects are the top (an initiative or app). Milestones group work inside a project (like an epic). Tasks are the individual things to do.

```
Project: "My App"
  └── Milestone: "MVP"
       ├── Task: "Build auth"
       ├── Task: "Create API"
       └── Task: "Write tests"
  └── Milestone: "Launch"
       ├── Task: "Deploy to prod"
       └── Task: "Write docs"
```

Slugs use dot notation: project slug is `my-app`, milestone is `my-app.mvp`, tasks are `my-app.build-auth`.

Every command that takes a slug also accepts the display name. The system tries three things in order: exact slug match, slugify your input, then case-insensitive name lookup against the graph. All three resolve to the same entity. So these are identical:

```bash
base p u my-app -s blocked          # slug
base p u "My App" -s blocked        # display name
base p u "MY APP" -s blocked        # case doesn't matter
```

## How the hooks work

Four hooks, all wired automatically by `base install`:

| Hook | Fires when | What it does |
|------|-----------|--------------|
| SessionStart | New session opens | Syncs domains, ingests projects, runs signals |
| UserPromptSubmit | You type a prompt | Matches keywords against domains, injects rules from graph |
| PreToolUse | Claude is about to read/edit a file | Injects AST file map for source files. Intercepts grep with graph hint. Injects domain rules for matched paths. |
| PostToolUse | Claude finishes reading | Updates timestamps. Injects section-specific AST context for partial reads. |

All hooks fail open. If anything errors, it logs to stderr and exits with empty stdout. Claude is never blocked by a hook failure.

## What lives where

```
~/.base-gbl/                  # global tier (all workspaces)
├── domains.toml              # global domain triggers
├── operator.toml             # your identity profile
├── base.toml                 # global config
└── docs/                     # reference docs

~/my-workspace/.base/         # workspace tier (one per workspace)
├── graph.trig                # the knowledge graph
├── domains.toml              # workspace domain triggers
├── ast.ttl                   # raw AST extraction output
├── base.toml                 # workspace config
└── .session                  # ephemeral (dedup tracking, prompt count)
```

Global domains.toml loads first, workspace overlays by name. The graph is workspace-only, no global graph.

## Domain matching

`domains.toml` defines when a domain fires:

```toml
[[domain]]
name = "BACKEND"
mode = "triggered"
prompt_keywords = ["api", "endpoint", "database", "query"]
file_keywords = ["use crate", "impl", "async fn"]
paths = ["src/api/", "src/db/"]
rules = ["Always validate inputs at API boundaries"]

[[domain]]
name = "FRONTEND"
mode = "triggered"
prompt_keywords = ["component", "page", "style", "layout"]
paths = ["src/ui/", "src/components/"]

[[domain]]
name = "GLOBAL"
mode = "always"
```

When you mention "api" in a prompt, the BACKEND domain fires and its rules inject. When Claude opens a file under `src/api/`, same thing. The graph provides the rules, decisions, and notes associated with that domain. Matching is deterministic - keywords and paths, nothing fuzzy.

## DEVMODE

Set `DEVMODE=true` in your environment and every prompt injection includes a telemetry block:

```
🔧 DEVMODE
Bracket: [FRESH] (prompt 3)
Loaded: GLOBAL [always] (13 rules), BACKEND [keyword] (4 rules)
Available: FRONTEND, TESTING, DEPLOYMENT
Dedup: 2 skipped
Tools: Read, Edit
```

Shows you exactly which domains fired, why they fired, what was deduped, and what's available but didn't match. You use this to tune your domains.toml until the right context fires at the right time.

## Context brackets

BASE tracks how deep you are in a session:

| Bracket | Prompts | Behavior |
|---------|---------|----------|
| FRESH | 1-3 | Lean injection, skip verbose context |
| MODERATE | 4-10 | Full injection |
| DEPLETED | 11-20 | Force-refresh dedup on interval |
| CRITICAL | 21+ | Same as depleted |

Thresholds are configurable in `base.toml`. The idea: early in a session you don't need heavy injection (Claude has fresh context). Later, context has been compacted or summarized, so the system re-injects what matters.

## Design principles

**Suppression over detection.** Detection is trivial. The product is the gate that stays silent until something changes. If you touch the same file twice, the AST map doesn't re-inject. If the same domain rules are already in context, they don't repeat. The value is in what BASE doesn't say.

**CLI over MCP server.** One surface for Claude, hooks, and you. `base hook <event>` reads stdin, writes stdout. Same binary handles everything. Zero standing context cost.

**Deterministic matching.** Keywords, paths, excludes. No embeddings, no semantic search, no fuzzy matching in the core loop. You configure a trigger, it fires exactly when that trigger matches. Predictable.

**Mandatory edges.** Every entity connects to something. `base learn` requires `--domain`. `base entity add` requires `--domain`. `base project add` requires `--path` (which auto-creates a domain trigger). No orphans in the graph.

**Fail open.** Every hook catches all errors, logs to stderr, exits 0 with empty stdout. If the graph is corrupt, if SPARQL fails, if the file doesn't exist - Claude keeps working. The system never blocks the prompt.

## Stack

| Layer | Tech |
|-------|------|
| Language | Rust (single binary, ~16MB) |
| Graph | Oxigraph (embedded, in-memory, loaded from disk per invocation) |
| Query | SPARQL SELECT and UPDATE |
| Persistence | TriG text files (git-native, atomic write-back via temp+rename) |
| Config | TOML (domains.toml, base.toml) |
| AST extraction | Tree-sitter (Python scripts, 35+ languages) |
| Hooks | Claude Code settings.json (stdin/stdout JSON) |

---

Built by Chris Kahler
[Chris AI Systems](https://chrisai.cv) / [Community](https://chrisai.cv/skool) / [YouTube](https://www.youtube.com/@chris-ai-systems)
