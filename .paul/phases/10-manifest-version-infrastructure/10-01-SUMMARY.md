---
phase: 10-manifest-version-infrastructure
plan: 01
subsystem: infra
tags: [toml, manifest, install, activation]

requires:
  - phase: 09-paul-integration-layer
    provides: Completed v0.1 milestone, stable install.rs

provides:
  - manifest.toml system (read/write/detect)
  - base install --full (component registration)
  - base activate <key> (Skool classroom activation)
  - curated section for third-party plugins

affects: [update-check, install-full, unofficial-detection]

tech-stack:
  added: []
  patterns: [manifest singleton at ~/.base-gbl/manifest.toml, atomic TOML write, compiled activation key]

key-files:
  created: [src/manifest.rs]
  modified: [src/lib.rs, src/install.rs, src/cli.rs]

key-decisions:
  - "Static activation key — one key compiled in binary, not per-user HMAC"
  - "Product name ChrisAI — replaces Agentic OS in manifest sections"
  - "Curated section added for third-party plugin tracking"
  - "Version checks via npm registry + GitHub API — no custom endpoint"

patterns-established:
  - "Manifest load/save with atomic temp+rename, same pattern as config.rs"
  - "Component detection via filesystem scan + package.json version read"

duration: 35min
started: 2026-06-03T15:00:00-05:00
completed: 2026-06-03T15:56:00-05:00
description: "manifest.toml module with component tracking, activation key, and install integration"
type: Summary
about: "base-v2"
---

# Phase 10 Plan 01: Manifest & Version Infrastructure Summary

**manifest.toml module with component tracking, `base activate` command, and install integration — single source of truth for what's installed**

## Performance

| Metric | Value |
|--------|-------|
| Duration | ~35 min |
| Started | 2026-06-03T15:00 CDT |
| Completed | 2026-06-03T15:56 CDT |
| Tasks | 3 completed |
| Files modified | 4 |

## Acceptance Criteria Results

| Criterion | Status | Notes |
|-----------|--------|-------|
| AC-1: TOML round-trip | Pass | 4 unit tests: round-trip, activated, empty token, wrong token |
| AC-2: `base install` writes manifest | Pass | Creates [chrisai] + [components.base] + [update_check] |
| AC-3: `--full` detects components | Pass | paul + seed detected; skillsmith absent (not at expected path) |
| AC-4: Re-install preserves data | Pass | Token and installed_at both preserved across re-install |

## Accomplishments

- New `src/manifest.rs` module: `Manifest`, `ChrisAiSection`, `ComponentEntry`, `CuratedEntry`, `UpdateCheck` structs with serde Serialize + Deserialize
- `Manifest::load()` / `Manifest::save()` with atomic write (temp + rename)
- `Manifest::is_activated()` — static key comparison against compiled constant
- `detect_component()` — filesystem scan for base/paul/seed/skillsmith with package.json version extraction
- `activate()` — validates key, writes token to manifest, prints success/error banners
- Install step 7: `write_manifest()` creates or updates manifest on every install
- `--full` flag on `base install` — registers all detected components
- `base activate <key>` CLI command wired into clap

## Files Created/Modified

| File | Change | Purpose |
|------|--------|---------|
| `src/manifest.rs` | Created | Manifest structs, I/O, activation, component detection (248 lines) |
| `src/lib.rs` | Modified | Added `pub mod manifest;` |
| `src/install.rs` | Modified | Added `write_manifest()` step, updated `run()` signature for `full: bool` |
| `src/cli.rs` | Modified | Added `--full` flag to Install, added Activate command + match arm |

## Decisions Made

| Decision | Rationale | Impact |
|----------|-----------|--------|
| Static activation key (not HMAC) | One key distributed via Skool classroom — simpler, no crypto deps needed | No sha2 dependency; key is the secret itself |
| Product name "ChrisAI" in manifest | Consistent branding across npm packages and manifest | [chrisai] section, not [agentic-os] |
| Curated section added to manifest | Tracks third-party plugins (e.g. context-mode) separately from core components | `HashMap<String, CuratedEntry>` with source field |
| No custom versions.json endpoint | npm registry + GitHub releases API already serve version data | Eliminates chrisai.cv infrastructure dependency for Phase 11 |
| Version fallback to "unknown" | Components without package.json still get registered | Resolves when npm publish happens |

## Deviations from Plan

### Summary

| Type | Count | Impact |
|------|-------|--------|
| Scope additions | 1 | Chris added `CuratedEntry` struct post-apply — tracked third-party plugins |
| Auto-fixed | 0 | — |
| Deferred | 0 | — |

**Total impact:** Minor scope addition (curated section), no creep.

## Issues Encountered

| Issue | Resolution |
|-------|------------|
| paul/seed versions show "unknown" | Expected — no package.json at detected paths. Resolves post npm-publish |
| skillsmith not detected | Not installed at `~/.claude/commands/skillsmith/`. Correct behavior |
| Pre-existing test failure (paul_json) | Unrelated to this plan. In boundaries (src/extract/) |

## Next Phase Readiness

**Ready:**
- Manifest exists at `~/.base-gbl/manifest.toml` — Phase 11 can read it for update checks
- `is_activated()` available — Phase 11/13 can branch on activation status
- Component detection working — Phase 12 can compare local vs remote versions

**Concerns:**
- Activation key is placeholder — must be replaced before any release build
- Paul/seed package.json needs to exist at detected paths for version accuracy

**Blockers:**
- None

---
*Built with PAUL Framework v1.4 · https://chrisai.cv/skool · https://youtube.com/@chris-ai-systems*
*Phase: 10-manifest-version-infrastructure, Plan: 01*
*Completed: 2026-06-03*
