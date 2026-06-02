# PAUL Session Handoff

**Session:** 2026-06-01 20:49 - 21:43 CDT
**Phase:** 6 of 8 — CARL Absorption (COMPLETE) — pre-Phase 7
**Context:** DEVMODE match-reason tags + AST serializer overhaul + dogfood

---

## Session Accomplishments

- **DEVMODE match-reason tags** — `MatchReason` enum (`Always`, `Keyword`, `Filepath`, `KeywordAndFilepath`) added to domain matcher. DEVMODE block now shows WHY each domain fired. Template updated so Claude renders it.
- **AST serializer — 3-gap fix** — Type inference (Struct vs Class vs Method vs Function vs Rationale vs Module), import target resolution (phantom → real IRI), per-file sourceFile with relative paths via `file_map`.
- **Graphify scrub** — All references to competitor framework removed across 4 files. Renamed to `base-ast`, `.base-ast-cache`, `.baseignore`, `BASE_AST_*` env vars.
- **`detect.py` created** — Missing module dependency for `collect_files()`. Provides `_load_baseignore`, `_is_ignored`, `_is_noise_dir`.
- **`base sync --ast` fixed** — CLI was passing `--output`/`--format` flags that `onto_ast.py` doesn't accept. Now uses `--out`/`--full`, writes `ast.ttl`, merges into `graph.trig` with `# --- AST BEGIN/END ---` markers for clean replacement on re-sync.
- **Dogfood complete** — Ran AST extraction on base-v2's own `src/` directory: 37 files → 289 entities, 1,074 relationship edges, 3,638 lines TTL. Merged into workspace graph (3,858 total lines).
- **Binary installed** — `~/.local/bin/base` updated with MatchReason + sync --ast fixes.

---

## Decisions Made

| Decision | Rationale | Impact |
|----------|-----------|--------|
| MatchReason only tracked when DEVMODE=true | Avoid overhead in production | Match reason string falls back to "always_on"/"matched" when devmode off |
| Graphify → base-ast naming | Competitor framework branding has no place in our tool | env vars, cache dirs, ignore files all renamed |
| Struct vs Class inferred from file extension | `.rs`/`.go`/`.c` → Struct, `.py`/`.js` → Class | Language-aware type assignment without modifying vendored extractor |
| Import targets resolved via label matching | Extractor produces shortened IDs for cross-file imports | `confidence: "resolved"` marks rewritten edges; unresolved externals (stdlib) left as-is |
| AST merged into graph.trig with markers | One graph file, not separate stores | `# --- AST BEGIN/END ---` markers allow clean replacement on re-sync |
| Dedup entries show original trigger in DEVMODE | Diagnostic value — know why something matched even when deduped | Format: `dedup [always]`, `dedup [filepath]` |

---

## Files Changed (7 modified, 1 new)

| File | What |
|------|------|
| `src/domain/matcher.rs` | `MatchReason` enum, `DomainMatch` struct, `is_matched()` returns `Option<MatchReason>`, new test `both_keyword_and_path` |
| `src/hook/user_prompt_submit.rs` | Uses `DomainMatch.reason`, dedup shows trigger, DEVMODE template includes `[reason]` |
| `src/cli.rs` | `base sync --ast` fixed — correct flags, ast.ttl merge with markers |
| `scripts/ast/ttl_serializer.py` | Complete serialize overhaul — role map, file membership, import resolver, struct/class/method/rationale types |
| `scripts/ast/onto_ast.py` | Passes `file_map` to serializer in `--full` mode |
| `scripts/ast/extractor.py` | Graphify scrub (4 references) |
| `scripts/ast/cache.py` | Graphify scrub — `BASE_AST_OUT`, `.base-ast-cache` |
| `scripts/ast/detect.py` | **NEW** — `_load_baseignore`, `_is_ignored`, `_is_noise_dir` |

---

## Known Issues (carried forward)

- `base sync` frontmatter extraction bug (errors on SUMMARY.md files) — pre-existing
- `base install` can't copy over itself when the running binary IS the target — pre-existing
- 3 sync tests failing (triple count mismatches) — pre-existing, unrelated to this session's changes
- `signal/mod.rs` has 43 entities — complexity hotspot, refactor candidate

---

## What's NOT Done

- Changes are NOT committed — 7 modified files + 1 new file sitting in working tree
- Phase 6 → Phase 7 transition not formally started
- PAUL → paul.toml port not done
- Orientation skill → operator.toml port not done

---

## Prioritized Next Actions

| Priority | Action | Effort |
|----------|--------|--------|
| 1 | Commit this session's changes (MatchReason + AST serializer + sync --ast) | 5 min |
| 2 | Formally close Phase 6, transition to Phase 7 | `/paul:plan` |
| 3 | Port PAUL to emit paul.toml instead of paul.json | Medium |
| 4 | Fix base sync frontmatter extraction bug | Small |
| 5 | Wire AST data into pre-tool-use hook (query graph on file touch) | Phase 7+ |

---

## State Summary

**Current:** Phase 6 COMPLETE, loop ✓/✓/✓, binary live, AST dogfooded
**Next:** Commit changes → Phase 7 planning
**Resume:** `/paul:resume` → read this handoff

---

*Handoff created: 2026-06-01 21:43 CDT*
