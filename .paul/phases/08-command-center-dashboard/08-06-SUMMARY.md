---
phase: 08-command-center-dashboard
plan: 06
subsystem: dashboard, extraction
tags: [d3, svelte, usage-analytics, tokenbbq, provider-abstraction, paul-toml]

requires:
  - phase: 08-command-center-dashboard
    provides: Existing dashboard with 4 panels, basic usage analytics
provides:
  - Provider-abstracted usage parsing (UsageEvent normalization layer)
  - TokenBBQ-competitive Usage Analytics panel (charts, tables, modals, tooltips)
  - Projects endpoint (/api/usage/projects)
  - Auto-refresh (30s polling)
  - Clickable stat card drill-down modals
  - paul.toml > paul.json priority in extraction
  - PAUL document → Project auto-linking in graph
affects: [phase-9-multi-provider, paul-framework-integration]

tech-stack:
  added: []
  patterns: [provider-abstraction (UsageEvent → collect_all_events → aggregate_usage), paul.toml-first extraction priority]

key-files:
  created: []
  modified:
    - src/dashboard/api.rs
    - src/dashboard/server.rs
    - dashboard/src/panels/UsageAnalytics.svelte
    - dashboard/src/lib/api.js
    - src/extract/frontmatter.rs
    - src/extract/mod.rs
    - src/extract/paul_json.rs

key-decisions:
  - "Provider abstraction via UsageEvent struct — all providers normalize to same shape"
  - "Remove mtime pre-filter — read ALL JSONL files, filter by content timestamp"
  - "Project names from cwd field in JSONL, not directory path splitting"
  - "paul.toml takes priority over paul.json — skip JSON when TOML exists"
  - "paul.json now produces Project type (not PaulProject) at project/ IRI"
  - "PAUL documents auto-link to project via cwd-derived project_hint"
  - "Orange (#E4934A) + Green (#3FB950) chart palette — replaces indistinguishable blue/purple"

patterns-established:
  - "collect_all_events(days) dispatcher pattern for multi-provider"
  - "extract_with_project() for project-hint injection during sync"
  - "paul.toml > paul.json precedence in extraction pipeline"

duration: 120min
started: 2026-06-03T08:00:00-05:00
completed: 2026-06-03T10:00:00-05:00
---

# Phase 8 Plan 06: Usage Analytics Parity + Extraction Fixes Summary

**Provider-abstracted usage analytics with TokenBBQ-competitive frontend, PAUL document graph linking, and paul.toml extraction priority.**

## Performance

| Metric | Value |
|--------|-------|
| Duration | ~120 min |
| Started | 2026-06-03T08:00 |
| Completed | 2026-06-03T10:00 |
| Tasks | 3 planned + significant scope expansion |
| Files modified | 9 |

## Acceptance Criteria Results

| Criterion | Status | Notes |
|-----------|--------|-------|
| AC-1: Provider-abstracted parsing layer | Pass | UsageEvent struct, parse_claude_code(), collect_all_events() dispatcher |
| AC-2: Enriched summary and daily data | Pass | active_days, cost_per_day, top_model, daily cache fields |
| AC-3: Project-level aggregation | Pass | /api/usage/projects endpoint live, sorted by cost |
| AC-4: Projects table and daily breakdown | Pass | Both tables with sortable columns, full comma numbers, show-all toggle |
| AC-5: Stat cards show Cost/Day and Active Days | Pass | 5 clickable cards with drill-down modals |

## Accomplishments

- **Provider-abstracted backend**: UsageEvent normalization layer — Phase 9 providers are drop-in parsers
- **Project names fixed**: Reads `cwd` from JSONL events instead of broken path splitting
- **No mtime filter**: ALL session files read, filtered by content timestamp — accurate counts across all time ranges
- **4-chart grid**: Daily Token Usage (stacked bars + tooltips), Top Models (orange gradient bars + grid + tooltips), Monthly Trend (area chart, zero-start, grid lines, dot hover), Activity Heatmap (90-day, centered)
- **5 clickable stat cards**: Total Tokens → Token Breakdown modal, Total Cost → Cost Analysis modal, Cost/Day → Daily Cost Trend modal, Active Days → Activity Analysis modal (streaks, coverage ring, DOW bars, monthly consistency), Top Model → Model Usage detail modal
- **Projects table**: TokenBBQ-style with sortable columns, comma-formatted numbers, provider badges, date format
- **Daily Breakdown table**: Per-day cache read/write/sources columns
- **Sessions table with inline filter pills**: Model filter directly above table, no more scrolling
- **Auto-refresh**: 30s polling, numbers update live
- **PAUL document graph linking**: All .paul/ docs auto-link to their project via cwd-derived project_hint
- **paul.toml > paul.json priority**: Extraction skips JSON when TOML exists, eliminates duplicate nodes
- **paul.json → Project type**: Legacy JSON extractor produces `Project` (not `PaulProject`) at `project/` IRI

## Files Created/Modified

| File | Change | Purpose |
|------|--------|---------|
| `src/dashboard/api.rs` | Modified | UsageEvent, provider abstraction, enriched structs, projects endpoint, cwd-based project names, no mtime filter |
| `src/dashboard/server.rs` | Modified | Wired /api/usage/projects route |
| `dashboard/src/panels/UsageAnalytics.svelte` | Modified | Complete visual overhaul — charts, tables, modals, tooltips, filter pills, auto-refresh |
| `dashboard/src/lib/api.js` | Modified | Added getUsageProjects() |
| `src/extract/frontmatter.rs` | Modified | extract_with_project() for project_hint, improved .paul/ path linking |
| `src/extract/mod.rs` | Modified | paul.toml > paul.json priority, project IRI override for legacy JSON |
| `src/extract/paul_json.rs` | Modified | Project type instead of PaulProject, project/ IRI |

## Decisions Made

| Decision | Rationale | Impact |
|----------|-----------|--------|
| Orange/Green chart palette | Blue/purple indistinguishable on dark background | All charts consistent |
| Remove mtime pre-filter | mtime excludes valid files, causes inaccurate session counts | Reads all files, slower but correct |
| cwd field for project names | Directory path splitting is lossy (base-v2 → v2) | Accurate project names matching TokenBBQ |
| paul.toml priority | Deprecating paul.json, avoid duplicates | Single Project entity per project |
| 30s polling over SSE | Simpler, no backend change needed | Live-enough updates, SSE is future optimization |

## Deviations from Plan

### Summary

| Type | Count | Impact |
|------|-------|--------|
| Scope additions | 5 | Significant — plan grew from 3 tasks to cover charts, modals, tooltips, extraction fixes, paul.toml priority |
| Auto-fixed | 3 | Button color inheritance, tooltip positioning, chart height collapse |
| Deferred | 2 | SSE for instant updates, chart tooltips on daily usage bars need parity with trend chart |

**Total impact:** Plan expanded significantly beyond original 3 tasks. Original scope (backend + stat cards + tables) was too conservative — TokenBBQ comparison exposed the gap. Extraction fixes (paul.toml priority, graph linking) were unplanned but essential data integrity fixes.

### Deferred Items

- SSE/WebSocket for instant usage updates (currently 30s polling)
- Full TokenBBQ visual parity still has gaps (interactive project drill-down, daily usage chart tooltip parity)

## Next Phase Readiness

**Ready:**
- Provider abstraction in place for Phase 9 multi-platform support
- paul.toml extraction pipeline is clean, no duplicates
- Dashboard is functional and data-accurate

**Concerns:**
- UsageAnalytics.svelte is large (~1050 lines) — may benefit from component decomposition
- Visual polish gap vs TokenBBQ still exists (their chart styling is more refined)
- 30s polling means N API calls re-parse all JSONL files — may need caching for large datasets

**Blockers:**
- None

---
*Phase: 08-command-center-dashboard, Plan: 06*
*Completed: 2026-06-03*
