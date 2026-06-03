# PAUL Session Handoff

**Session:** 2026-06-03 08:14 - 14:55 CDT
**Phase:** v1.0 Agentic OS Distribution — milestone created, 0 of 4 phases started
**Context:** Agentic OS product definition, framework upgrades (SEED + SKILLSMITH), BASE v1.0 installer/updater spec

---

## Session Accomplishments

### Strategic
- Defined the Agentic OS product concept — Chris's answer to Charlie's "Charlie OS" (which resells Chris's frameworks)
- Audited all 4 frameworks: BASE v2 (production), PAUL v1.4 (shipped), SEED v0.1 (MVP), SKILLSMITH v0.1 (MVP)
- Identified gap areas for each framework's Agentic OS readiness
- Designed the install/update/attribution architecture

### SEED v1.0 — Agentic OS Integration (COMPLETE)
- **6 phases, 6 plans, 22 tasks, 0 deviations**
- Phase 1: BASE v2 Integration — detect_base in all 4 task files, ACTIVE.md replaced with graph registration
- Phase 2: Persistence & Resume — SEED-STATE.md checkpoints, resume detection, cleanup on completion
- Phase 3: PAUL v1.4 Alignment — paul.json → paul.toml, frontmatter on all templates, launch.md PAUL v1.4 detection
- Phase 4: Attribution & Branding — provenance footer on all templates and completion messages, entry point branded
- Phase 5: Custom Type Enhancement — add-type auto-generates planning template for full round-trip
- Phase 6: Packaging & Version Bump — README v1.0, package.json 1.0.0, ecosystem refs cleaned
- Location: `/home/chriskahler/ops-sys/toolbox/skills/seed/`

### SKILLSMITH v1.0 — Agentic OS Integration (COMPLETE)
- **4 phases, 11 tasks, 0 deviations**
- Phase 1: BASE v2 Integration — detect_base in scaffold.md, graph registration on scaffold completion
- Phase 2: Attribution & Provenance — skillsmith_version/source in scaffolded frontmatter, provenance footer on all generated files
- Phase 3: PAUL v1.4 Alignment — paul.json deleted, no stale CARL/CC Strategic refs
- Phase 4: Packaging & Version Bump — README v1.0, package.json 1.0.0, ecosystem table cleaned
- Location: `/home/chriskahler/ops-sys/toolbox/skills/skillsmith/`

### BASE v1.0 — Agentic OS Distribution (SPECCED, NOT STARTED)
- Created v1.0 milestone with 4 phases in ROADMAP.md
- Specced manifest.toml format, update check architecture, install/update commands, unofficial detection
- Location: `/home/chriskahler/chris-ai-systems/apps/base-v2/`

---

## Decisions Made

| Decision | Rationale | Impact |
|----------|-----------|--------|
| BASE is the single installer entry point (`base install --full`) | Already a compiled binary with install command and hook wiring | No new tool needed, unified install story |
| PAUL telemetry lives in BASE, not PAUL | PAUL stays open source and event-dumb, BASE observes via hooks | Clean open-core model — free tool → paid OS |
| 7-day TTL on version checks, not daily | Chris doesn't release daily; reduces CDN traffic by 7x | ~143 checks/day at 1000 users vs ~10,000 |
| Persistent update banner until resolved | Once detected, shows every session until `base update` or `--snooze` | Users can't passively ignore updates |
| `base update --snooze` = 24h dismiss | Respects user agency without letting them permanently ignore | Banner returns after 24h if not updated |
| CDN-backed static `versions.json` at chrisai.cv | Single HTTP call, Cloudflare edge-cached, origin sees ~24 req/day | Near-zero infra cost at any scale |
| HMAC token for direct vs community install | Compiled secret in binary — can't be bypassed by editing TOML | Casual tampering defeated, decompilation required |
| Token field named `token` not `install_channel` | Less obvious what it controls | Obscurity layer on top of cryptographic verification |
| Attribution CTA: "For the official Agentic OS and to permanently remove attribution, visit chrisai.cv/skool" | Attribution becomes a feature gate, not just a watermark | Free users see it everywhere, Skool members get clean install |
| Promote-then-fallback on BASE absence (not silent fallback) | Every touchpoint is a funnel entry | Users without BASE see what they're missing |
| BASE v1 hard stop across all frameworks | v1 is dead, forces upgrade | Clean break, no compatibility maintenance |
| CARL references removed everywhere | CARL absorbed into BASE v2 | One less framework to explain |

---

## Gap Analysis with Decisions

### npm publish for SEED and SKILLSMITH
**Status:** DEFER to manual
**Notes:** Code is ready. Chris needs to run `npm publish` with auth.
**Effort:** 5 min each

### versions.json endpoint on chrisai.cv
**Status:** CREATE (Phase 12 dependency)
**Notes:** Static JSON file hosted on Cloudflare CDN. Need to create the file and route.
**Effort:** 15 min

### HMAC secret selection
**Status:** CREATE (Phase 10)
**Notes:** Need to choose/generate the secret that gets compiled into the BASE binary. Chris's private installer needs the same secret to generate valid tokens.
**Effort:** Trivial — generate once, store securely

### PAUL attribution updates
**Status:** DONE (by Chris in earlier session)
**Notes:** PAUL v1.4 already has Chris AI Systems branding, provenance in paul.toml, footers on summaries. The same CTA wording should be applied to PAUL's footer lines.
**Reference:** `apps/paul-framework/src/templates/SUMMARY.md`

---

## Open Questions

- Should PAUL's attribution footer also use the "permanently remove attribution" CTA, or is the current branding sufficient since PAUL is more established?
- What's the installer binary name for Chris's private installer? `base install --full --direct`? Or a separate binary/script?
- GitHub release automation — manual or CI/CD on tag push?

---

## Reference Files for Next Session

```
# BASE v2 Rust source
@apps/base-v2/src/
@apps/base-v2/.paul/ROADMAP.md (Phases 10-13 specced)
@apps/base-v2/.paul/STATE.md
@apps/base-v2/.paul/paul.toml
@apps/base-v2/Cargo.toml

# Completed framework upgrades (reference patterns)
@/home/chriskahler/ops-sys/toolbox/skills/seed/ (SEED v1.0)
@/home/chriskahler/ops-sys/toolbox/skills/skillsmith/ (SKILLSMITH v1.0)
@apps/paul-framework/src/workflows/init-project.md (PAUL v1.4 detect_base pattern)
```

---

## Prioritized Next Actions

| Priority | Action | Effort |
|----------|--------|--------|
| 1 | Open session in `apps/base-v2/`, `/paul:plan` Phase 10 (Manifest & Version Infrastructure) | Rust implementation |
| 2 | Phase 11: Update check in session start hook | Rust — HTTP client, TOML read/write |
| 3 | Phase 12: `base install --full` + `base update` | Rust — GitHub release download, extraction |
| 4 | Phase 13: Unofficial install detection | Rust — framework scanning in session hook |
| 5 | npm publish SEED + SKILLSMITH | 5 min manual |
| 6 | Create `versions.json` on chrisai.cv | 15 min |
| 7 | Apply CTA wording to PAUL's SUMMARY footer | 2 min |

---

## State Summary

**Current:** BASE v1.0 Agentic OS Distribution — Phase 0 of 4, IDLE
**Next:** `/paul:plan` Phase 10 in `apps/base-v2/`
**Resume:** Open Claude Code in `apps/base-v2/`, run `/paul:resume` → read this handoff

---

*Handoff created: 2026-06-03T14:55:00-05:00*
