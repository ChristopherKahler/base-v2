---
phase: 08-command-center-dashboard
plan: 01
subsystem: dashboard
tags: [axum, svelte, d3, graph-viz, websocket-prep, huly-design]

requires:
  - phase: 07-v1-migration-cutover
    provides: Graph query patterns (SPARQL, crud helpers, AST query)
provides:
  - Embedded axum HTTP server (`base dashboard` command)
  - SPARQL-backed JSON API (4 endpoints)
  - Svelte SPA with D3.js Graph Explorer panel
  - Huly design system CSS implementation
  - `base sync --repair` edge backfill command
  - AST script installation via `base install`
  - Updated CLAUDE.md with AST query commands
affects: [operations-panel, session-activity, usage-analytics, operatorNotes]

tech-stack:
  added: [axum 0.8, tokio 1, tower-http 0.6, include_dir 0.7, open 5, svelte, vite, d3]
  patterns: [embedded-http-server, spa-embedded-in-binary, sparql-to-json-api, huly-design-tokens]

key-files:
  created: [src/dashboard/mod.rs, src/dashboard/server.rs, src/dashboard/api.rs, dashboard/src/App.svelte, dashboard/src/panels/GraphExplorer.svelte, dashboard/src/app.css, dashboard/src/lib/api.js]
  modified: [Cargo.toml, src/lib.rs, src/cli.rs, src/install.rs, src/crud/mod.rs, src/crud/decision.rs, src/extract/frontmatter.rs, src/hook/pre_tool_use.rs, README.md, .gitignore]

key-decisions:
  - "Svelte over vanilla JS — component model from day 1, no build-step-later refactor"
  - "Huly design system — dark canvas #090A0C, blue-purple accent axis, gradient depth over shadows"
  - "0.0.0.0 bind for WSL2 compatibility — localhost forwarding is unreliable"
  - "Custom left-edge resize handle — CSS resize only goes bottom-right"
  - "D3 force scaling by node count — tight for small graphs, looser for large"
  - "Decision CRUD now creates hasDecision edge to domain — was missing"

patterns-established:
  - "Dashboard panels are Svelte components in dashboard/src/panels/"
  - "API endpoints in src/dashboard/api.rs share AppState via Arc"
  - "Graph store loaded once at server start, shared across requests"
  - "npm run build → cargo build → include_dir! embeds dist/ in binary"

duration: ~90min
started: 2026-06-02T11:27:00-05:00
completed: 2026-06-02T12:37:00-05:00
---

# Phase 8 Plan 01: Server + Graph Explorer Summary

**Embedded axum dashboard with Svelte/D3.js graph explorer — `base dashboard` serves the knowledge graph as an interactive force-directed visualization from a single binary.**

## Performance

| Metric | Value |
|--------|-------|
| Duration | ~90 min |
| Started | 2026-06-02T11:27 CDT |
| Completed | 2026-06-02T12:37 CDT |
| Tasks | 3 completed + 5 post-plan fixes |
| Files modified | 17 |

## Acceptance Criteria Results

| Criterion | Status | Notes |
|-----------|--------|-------|
| AC-1: Dashboard server starts and serves SPA | Pass | axum on 0.0.0.0:3741, opens browser, Svelte SPA loads |
| AC-2: Graph API returns real data | Pass | 4 endpoints (nodes, edges, search, node detail) — verified with curl |
| AC-3: Graph Explorer renders interactive viz | Pass | D3 force-directed, color-coded nodes, click detail, search, type filters |

## Accomplishments

- `base dashboard` (alias `base dash`) starts embedded axum server, opens browser, serves Svelte SPA with embedded assets
- Graph Explorer: D3.js force-directed visualization with entity-type color coding (Huly palette), click-to-select detail panel, search bar, entity type filter chips, Align/Fit controls
- 4 JSON API endpoints backed by live SPARQL queries against the workspace graph
- Huly design system implemented: dark canvas, surface ladder, blue-purple accent axis, glass effects, proper typography
- Resizable detail panel with left-edge drag handle
- Rich markdown extraction (headings, links, wikilinks, @-mentions, individual tags, relatedTo edges)
- MOP PreToolUse hook for markdown authoring guidance
- `base sync --repair` command backfills missing edges (decisions→domains, milestones→projects, tasks→projects)
- Decision CRUD now creates `hasDecision` edge — was silently orphaning decisions
- AST scripts copied to `~/.base-gbl/scripts/ast/` during install
- CLAUDE.md updated with AST query commands (fixes Claude defaulting to `base recall`)

## Files Created/Modified

| File | Change | Purpose |
|------|--------|---------|
| `src/dashboard/mod.rs` | Created | Dashboard module root |
| `src/dashboard/server.rs` | Created | Embedded axum HTTP server, static file serving, graceful shutdown |
| `src/dashboard/api.rs` | Created | SPARQL-backed JSON API (nodes, edges, search, node detail) |
| `dashboard/src/App.svelte` | Created | SPA shell with sidebar nav and panel routing |
| `dashboard/src/panels/GraphExplorer.svelte` | Created | D3.js force-directed graph visualization |
| `dashboard/src/app.css` | Created | Huly design system implementation |
| `dashboard/src/lib/api.js` | Created | Fetch wrapper for graph API endpoints |
| `dashboard/vite.config.js` | Created | Vite config with relative base paths |
| `Cargo.toml` | Modified | Added axum, tokio, tower-http, include_dir, open |
| `src/lib.rs` | Modified | Added `pub mod dashboard` |
| `src/cli.rs` | Modified | Dashboard command + handler, sync --repair flag, AST script path fix |
| `src/install.rs` | Modified | Script installation, updated CLAUDE.md section with AST commands |
| `src/crud/mod.rs` | Modified | `repair_edges()` function, fixed IRI extraction |
| `src/crud/decision.rs` | Modified | Added `hasDecision` edge creation |
| `src/extract/frontmatter.rs` | Modified | Rich body extraction (headings, links, wikilinks, @-mentions, tags, relatedTo) |
| `src/hook/pre_tool_use.rs` | Modified | MOP markdown authoring guidance injection |
| `README.md` | Modified | Documentation layer, practice scenario, comparison table rows, sync description |
| `.gitignore` | Modified | Added huly-DESIGN.md, node_modules, dist |

## Decisions Made

| Decision | Rationale | Impact |
|----------|-----------|--------|
| Svelte from day 1 | Component model prevents vanilla JS mess at 4-panel scale; compiles to vanilla JS anyway | All future panels are Svelte components |
| Huly design system | Professional dark-canvas aesthetic for marketing screenshots; purpose-built > generic framework | CSS tokens established for all panels |
| 0.0.0.0 bind + post-bind browser open | WSL2 localhost forwarding unreliable; race condition on early browser open | Dashboard works reliably from WSL |
| Custom resize handle (left edge) | CSS `resize: horizontal` only goes bottom-right; panel anchored right | Better UX for right-anchored panel |
| Force scaling by node count | Small graphs were too spread, large graphs too tight | Adaptive layout without manual tuning |
| Decision CRUD creates domain edge | Decisions were orphaned — no `hasDecision` edge created | Future decisions auto-connect; existing backfilled via repair |
| `base sync --repair` as permanent command | One-time fix needed, but useful ongoing for any orphaned entities | Users can self-heal graph edge gaps |

## Deviations from Plan

### Summary

| Type | Count | Impact |
|------|-------|--------|
| Auto-fixed | 3 | Essential WSL/asset/edge fixes |
| Scope additions | 4 | Pre-dashboard quality (markdown extraction, MOP hook, CLAUDE.md, repair) |
| Deferred | 0 | None |

### Auto-fixed Issues

**1. WSL2 connection refused**
- Found during: Task 1 (server startup)
- Issue: Server bound 127.0.0.1, browser opened before listener ready
- Fix: Changed to 0.0.0.0, moved browser open after bind
- Verification: Dashboard loads from Windows browser

**2. Static asset 404 on nested paths**
- Found during: Task 3 (SPA serving)
- Issue: `Path<String>` extractor can't handle `/assets/index-xxx.js` (multi-segment)
- Fix: Changed to `Uri` extractor, parse path from URI directly
- Verification: All JS/CSS assets load correctly

**3. Floating decision nodes in graph**
- Found during: Dashboard testing
- Issue: `decision::log()` never created `hasDecision` edge to domain
- Fix: Added edge creation in CRUD + `sync --repair` for existing data
- Verification: 5 edges backfilled, nodes connected in visualization

### Scope Additions (pre-dashboard quality)

- Rich markdown extraction (frontmatter.rs overhaul) — from handoff queue
- MOP PreToolUse hook for .md files — from handoff queue
- README.md updates for markdown extraction features
- CLAUDE.md AST query section + script install fix

## Next Phase Readiness

**Ready:**
- Axum server infrastructure for additional panels
- Svelte component pattern established
- API layer extensible (add routes to api.rs)
- Huly CSS tokens available for all panels
- Graph store shared via Arc<AppState>

**Planned for Plan 02:**
- Operations panel (kanban + table views)
- OperatorNotes feature (first write mutation via dashboard)
- People, decisions, reminders display

**Planned for Plan 03:**
- Live session activity (WebSocket)
- Usage analytics (ccusage JSONL parsing)

**Concerns:**
- Graph loads once at server start — no hot reload if graph changes during session
- Binary size increased to 19MB (from ~16MB) with embedded dashboard assets

**Blockers:** None

---
*Phase: 08-command-center-dashboard, Plan: 01*
*Completed: 2026-06-02*
