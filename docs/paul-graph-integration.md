# PAUL → BASE Graph Integration

## What This Is

BASE is a proactive context-injection engine for Claude Code. It maintains an RDF knowledge graph (`graph.trig`) per workspace that stores metadata about the codebase, project state, and engineering history. Hooks query this graph at runtime and inject relevant context into Claude sessions automatically.

PAUL is a plan-apply-unify framework for structured software development. Each cycle produces markdown artifacts (PLAN.md, SUMMARY.md) with structured metadata about what was planned, what was built, what decisions were made, and what passed verification.

Starting in BASE v0.1.4, a specialized PAUL extractor (`paul_md.rs`) parses PAUL artifacts into **typed graph entities** instead of generic document nodes. This means the graph contains queryable engineering history — not just "there's a file called 08-07-SUMMARY.md" but "plan 08-07 made 4 decisions, changed 5 files, and all 4 acceptance criteria passed."

## How It Works

### Extraction Pipeline

```
base sync
  ↓
Scans workspace files matching .base/base.toml include patterns
  ↓
Routes .paul/phases/*-PLAN.md and *-SUMMARY.md to paul_md.rs
  (all other .md files go through the generic frontmatter extractor)
  ↓
paul_md.rs parses:
  - YAML frontmatter → phase, plan, tags, dependsOn, affects, timestamps
  - Markdown body → typed entities from known section patterns
  ↓
Entities are inserted into the oxigraph store as RDF triples
  ↓
write_back() serializes the store to graph.trig
  (with post-dump validation — re-parses before overwriting)
```

### Entity Types Produced

| RDF Type | Source | What It Captures |
|----------|--------|-----------------|
| `PaulPlan` | `*-PLAN.md` | Plan metadata: phase, plan number, type, wave, dependencies, goal, scope limits, source files |
| `PaulSummary` | `*-SUMMARY.md` | Summary metadata: phase, plan, subsystem, tags, duration, started/completed timestamps, accomplishments |
| `Decision` | SUMMARY "Decisions Made" table | Individual engineering decisions with description, rationale, impact, and source plan ID |
| `FileChange` | SUMMARY "Files Created/Modified" table | Individual file modifications with path, change type, purpose, and source plan ID |
| `AcceptanceCriteria` | PLAN "AC-N:" sections | Acceptance criteria definitions with names and target files |
| `AcceptanceCriteriaResult` | SUMMARY "Acceptance Criteria Results" table | AC verification results with criterion name, pass/fail status, notes, and source plan ID |

### How Entities Link Together

```
PaulSummary (document node)
  ├── hasDecision → Decision
  │     ├── description: "Fixed overlay detail panel"
  │     ├── rationale: "User feedback — inline flex pushed content left"
  │     ├── impact: "Pattern reusable for other detail views"
  │     └── fromPlan: "08-07"
  ├── hasFileChange → FileChange
  │     ├── filePath: "src/dashboard/api.rs"
  │     ├── changeType: "Modified (+438 lines)"
  │     ├── purpose: "update_task, delete_task, ..."
  │     └── fromPlan: "08-07"
  └── hasACResult → AcceptanceCriteriaResult
        ├── criterion: "AC-1: Task Update Endpoint"
        ├── status: "Pass"
        └── fromPlan: "08-07"
```

The `fromPlan` field on every entity is the join key. It connects decisions to file changes to AC results across the same plan. This is what makes cross-entity queries work — you can ask "what decisions drove changes to this file?" by finding FileChanges by path, reading their fromPlan, then querying Decisions with the same fromPlan.

### ENTITY@@ Separator Pattern

PAUL entities use a multi-subject insertion pattern. The standard extract pipeline expects all triples to share a single document IRI as subject. PAUL entities need their own IRIs (e.g., `decision/08-07-fixed-overlay`).

The extractor returns triples with a special prefix format:
```
("ENTITY@@{entity_iri}@@{predicate}", "{value}")
```

The insert loop in `extract/mod.rs` detects this prefix, splits on `@@`, and inserts the triple with the entity IRI as subject instead of the document IRI. It also collects entity IRIs for idempotent cleanup (DELETE before INSERT).

## Query Surfaces

### `base recall --keyword <term>`

Searches across Note, Decision, FileChange, and AcceptanceCriteriaResult entities. Uses SPARQL UNION queries to hit all four types. Results show:
- **type**: note, decision, file-change, or ac-result
- **text**: the searchable content (noteText, description, filePath, or criterion)
- **context**: extra detail (rationale for decisions, status for ACs, purpose for file changes)
- **plan/date**: source plan ID or creation date

### Hook injection (pre_tool_use)

When Claude opens a file for editing, `pre_tool_use` runs `query_paul_context()`:
1. Queries FileChange entities where `filePath` contains the edited file's path
2. For each plan ID found, queries Decision entities from that plan
3. Injects a `<paul-context>` block with file change history and linked decisions

This fires automatically — no user action required. The result looks like:
```xml
<paul-context>
File history for: src/dashboard/api.rs
  Plan 08-01: SPARQL-backed JSON API (nodes, edges, search, node detail)
  Plan 08-07: OpsTask description, UpdateTaskBody, update_task, delete_task...
Decisions:
  [08-07] Fixed overlay detail panel — User feedback — inline flex pushed content left
  [08-07] Complete vs dismiss reminders — User requested
</paul-context>
```

### Dashboard (graph visualization)

The dashboard `nodes()` endpoint returns all typed entities. It uses `COALESCE(?rawName, ?desc)` to display Decision entities (which use `description` instead of `name`). The `is_edge_predicate()` function recognizes `hasDecision`, `hasFileChange`, `hasACResult`, `hasAC`, `dependsOn`, and `affects` as edge predicates for graph rendering. The `search()` endpoint searches both `name` and `description` fields.

## Data Safety

### IRI Sanitization

All IRI construction routes through `crud::slugify()` — converts any string to a URL-safe slug (lowercase, non-alphanumeric → hyphens, deduped). This prevents invalid characters (spaces, pipes, curly braces) from entering the graph. Before v0.1.3, ad-hoc sanitization missed these characters, causing oxigraph serializer corruption.

### CRLF Normalization

File content is stripped of `\r` before extraction. The `escape()` function in both `frontmatter.rs` and `paul_toml.rs` handles `\r`, `\n`, `\t`, `"`, and `\` in SPARQL string literals.

### Write-back Validation

`store::write_back()` serializes to a temp file, re-parses it to validate, and only renames to the real path if validation passes. If the oxigraph serializer produces corrupt output, the original file is preserved and an error is returned. This prevents the data loss scenario where a bad round-trip overwrites good data.

## What Users Should Know

1. **`base sync` must run** for PAUL artifacts to enter the graph. It runs automatically at session start via hooks, but manual runs are needed after bulk changes.

2. **`base sync --ast`** is separate — it indexes code entities (functions, classes, modules) via tree-sitter. PAUL extraction happens in the normal `base sync` path.

3. **`base recall --keyword <term>`** is the primary query interface. It searches notes, decisions, file changes, and AC results in one query.

4. **The graph is per-workspace.** Each workspace with a `.base/` directory has its own `graph.trig`. The global tier at `~/.base-gbl/.base/graph.trig` stores cross-workspace data (domains, rules). Both are loaded and merged at query time.

5. **PAUL artifacts are detected by path pattern.** Any file matching `.paul/phases/*-PLAN.md` or `*-SUMMARY.md` routes to the PAUL extractor. Other `.paul/` files (PROJECT.md, STATE.md, ROADMAP.md) go through the generic frontmatter extractor.

6. **Frontmatter matters.** The PAUL extractor reads YAML frontmatter fields: `phase`, `plan`, `subsystem`, `tags`, `depends_on`, `affects`, `duration`, `started`, `completed`. PAUL's template system produces these automatically — users don't need to add them manually.

7. **Table format matters.** Decisions, file changes, and AC results are extracted from markdown tables with specific column structures. The extractor expects pipe-delimited tables with a header row and separator row. Non-table content in those sections is handled as prose fallback.
