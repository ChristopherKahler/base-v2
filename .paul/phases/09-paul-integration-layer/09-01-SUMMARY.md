---
phase: 09-paul-integration-layer
plan: 01
subsystem: extraction, api
tags: [ledger, cost-attribution, toml, sparql, session-join]

requires:
  - phase: 08-command-center-dashboard
    provides: UsageEvent provider-abstraction, collect_all_events(), session JSONL parsing
provides:
  - ledger.toml extractor (LedgerEntry triples in graph)
  - /api/ops/ledger endpoint with session-cost timestamp join
  - /api/ops/cost-summary endpoint with per-phase/action rollups
  - Frontend API: getLedger(), getCostSummary()
affects: [phase-9-plan-02-cost-dashboard, paul-framework-v1.4]

tech-stack:
  added: []
  patterns: [timestamp-match-join, deterministic-iri-hashing, append-only-ledger]

key-files:
  created:
    - src/extract/ledger.rs
    - .paul/ledger.toml
  modified:
    - src/extract/mod.rs
    - src/dashboard/api.rs
    - src/dashboard/server.rs
    - dashboard/src/lib/api.js

key-decisions:
  - "Deterministic IRI via djb2 hash of action+phase+plan+timestamp — stable, dedup-safe"
  - "30-minute window for timestamp-match session join — generous enough for session overlap"
  - "collect_all_events(365) called per request — no caching, correctness first"

patterns-established:
  - "Ledger extractor runs after file-walk sync, not through file discovery"
  - "Timestamp-match join pattern for cross-data-source attribution"

duration: 25min
started: 2026-06-03T11:15:00-05:00
completed: 2026-06-03T11:44:00-05:00
---

# Phase 9 Plan 01: Ledger Extractor + Cost Join Summary

**Ledger.toml extractor produces LedgerEntry triples; API endpoints join against session JSONL for per-action cost attribution with phase/milestone rollups.**

## Performance

| Metric | Value |
|--------|-------|
| Duration | ~25 min |
| Started | 2026-06-03T11:15 |
| Completed | 2026-06-03T11:44 |
| Tasks | 2 completed |
| Files modified | 6 |

## Acceptance Criteria Results

| Criterion | Status | Notes |
|-----------|--------|-------|
| AC-1: Ledger Extractor | Pass | 3 test entries extracted, idempotent on re-sync |
| AC-2: Session-Cost Join | Pass | Entries returned with session_tokens + session_cost populated via timestamp match |
| AC-3: Cost Summary Rollups | Pass | Per-phase grouping with per-action breakdown returned |

## Files Created/Modified

| File | Change | Purpose |
|------|--------|---------|
| `src/extract/ledger.rs` | Created | Ledger.toml parser → LedgerEntry triples with deterministic IRI |
| `src/extract/mod.rs` | Modified | Wire ledger extractor into sync pipeline |
| `src/dashboard/api.rs` | Modified | OpsLedgerEntry, ops_ledger (with join), ops_cost_summary |
| `src/dashboard/server.rs` | Modified | /api/ops/ledger + /api/ops/cost-summary routes |
| `dashboard/src/lib/api.js` | Modified | getLedger(), getCostSummary() |
| `.paul/ledger.toml` | Created | Test ledger with 3 entries for Plan 08-07 |

## Deviations from Plan

None — plan executed as written.

## Deferred Issues

- Session Activity panel groups all sessions into one card (cross-session hook-events.jsonl conflation) — **hotfix in progress**
- Ledger data has no dashboard UI yet (Plan 09-02)

## Next Phase Readiness

**Ready:** API endpoints work, data flows from ledger.toml → graph → API with cost join. Frontend functions ready for dashboard consumption.
**Concerns:** No caching on cost join (re-parses all JSONL per request). Ledger nodes orphaned until UI built.
**Blockers:** None

---
*Phase: 09-paul-integration-layer, Plan: 01*
*Completed: 2026-06-03*
