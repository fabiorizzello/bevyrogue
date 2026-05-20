# Summary — Reduce LOC in tests/ + R003 violator handling (Option 3 — Full)

Closed: 2026-05-20
HEAD at close: `6c7b210`
Branch: master (atomic per-wave commits, no force pushes, no rebases)

## Outcome

| Metric | Pre-refactor | Post-refactor | Delta |
|---|---|---|---|
| `tests/` LOC | 29,548 | ~30,929 | +1,381 (relocated from src/) |
| `src/` LOC | 31,692 | ~26,500 | −5,192 (combined relocate + dedup) |
| Inline `#[cfg(test)] mod tests` >100 LOC | 9 files (1,445 LOC) | **0** | R003 restored |
| `src/.../tests/` directories | 4 (3,312 LOC) | **0** | R003 restored |
| Test files using rstest/proptest | 5 | **12+** | +7 named parametric suites |
| Test binaries green at close | — | **120** | full suite, zero failure |

## What changed

**Wave 0 — R003 violator handling** (5 sub-waves, 5 commits)

- W0a-1, W0a-2: 11 test files relocated from `src/.../tests/` to `tests/` via `git mv`. Private symbols promoted to `pub(crate)` only where unavoidable.
- W0b-1, W0b-2, W0b-3: 9 inline `#[cfg(test)] mod tests` blocks ≥100 LOC dedupped — 3 files pure-delete (subsumed by integration), 2 files mixed delete+relocate, 4 files pure-relocate to `tests/<module>_internals.rs`. Promoted `FollowerSnapshot` + `evaluate_follow_up` to `pub`.

**Wave 1 — Common infrastructure** (1 commit)

- New `tests/common/app.rs::TestAppBuilder` chainable builder covering 3 shape classes (minimal / passive-dispatch / kernel) + 4 standalone convenience fns. New `tests/common/events.rs::drain<T>()`. New `tests/common/constants.rs`. Extended `tests/common/units.rs::make_unit`.

**Waves 2–6 — Mechanical consolidation** (8 commits, −1,151 LOC)

- W2: 3 compiled_timeline files → 1 parametric `#[rstest]` matrix (fn-pointer for event predicates and post-asserts).
- W3: tentomon + dorumon blueprint dispatch tests parametrized; patamon left intact (1 case = no value).
- W4: party RON test ported into selection validation; holy_support struct-roundtrip tautology dropped.
- W5: 3 test files deleted entirely + 9 trimmed (H5 + T1 + V2 batch).
- W6a-c: ultimate_event folded into ultimate_meter (303 LOC dominant win); status_amp + anim_graph_parse + anim_graph_asset rstest folds.

**Wave 7 — Common-helper migration** (4 commits, −250 LOC, 20 files)

- W7a: 9 files migrated to W1 helpers across 4 shape buckets.
- W7b: added `skill_book_runtime_app` + `form_identity_runtime_app` helpers; 4 files.
- W7c: added `skill_resolve_app(book, seed)` helper; 4 files.
- W7d: added `turn_av_base_app()` helper; 3 files + 1 inline duplicate fold.

**Wave 8 — Quality rewrite** (1 commit, +5 LOC net)

- Status_paralyzed_skip: 100-iter no-variance loop → `proptest!` invariant with 256 cases over `(turn_seed, prior_state, duration)`.
- V4 fragility rewrite ANNULLATO — `last_transition` confirmed as typed observability contract surface (NOT internal latch). See `4a01aef` commit body for full evidence.

## Deviations from plan

| Plan claim | Reality | Reason |
|---|---|---|
| W0b 662 LOC delete + 783 LOC moved | 245 delete + 1,217 moved | Per-file evidence: pure-function tests had no integration coverage; correct R003 reading is relocate, not delete. |
| Wave 6 8 targets, 360+79 LOC | 3 targets, 369 LOC | Pairwise read revealed 5 targets would degrade readability or had no shared shape; PLAN over-listed for symmetry. |
| Wave 7 1 bulk commit, ~250 LOC | 4 sub-commits, 250 LOC exact | Split during execution by setup-pattern cluster to keep diffs reviewable. |
| Wave 8 V4 fragility, 95 LOC rewrite | Annulled | `last_transition` is observability contract per typed display API + ValidationField surface. Rewriting would remove the regression guard. |

**Net**: realized −1,646 delete vs PLAN target −1,843 = **89.3% hit**. Gap explained by W0b reclassification (relocate vs delete) and Wave 6 reframe (5 marginal targets dropped). All R003 acceptance criteria met regardless of LOC variance.

## Verification at close

```
cargo test --tests           # 120 binaries, 0 failures
cargo check --tests          # 0 warnings on changed files
find src -type d -name tests # empty (R003 W0a)
# Inline mod tests ≥100 LOC scan: empty (R003 W0b)
```

## Follow-up backlog (out of scope of this slice)

### Closed post-initial-close (2026-05-20, same day)

| # | Item | Commit | Note |
|---|---|---|---|
| 1 | DECISION `last_transition` as observability contract | `7f067a5` | Added D026 in `.gsd/DECISIONS.md` + P005 in `.gsd/KNOWLEDGE.md` with `file:line` for all 5 consumers (snapshot field + format fn + ValidationField + 2 blueprints). |
| 3 | `source_file_loc_limit` → CI/lint check | `00a4310` | Moved 65 LOC meta-test to `scripts/check_loc_cap.sh`; verified positive (no offenders) and negative (synthetic 600 LOC probe → exit 1). |
| 5 | `status_slowed_delay.rs` outlier annotation | `f38078c` | Documented as intentional W7c outlier (unique scheduling). |
| 2 (partial) | W0c — Tier-A inline `mod tests` relocate | `aa22a1f` `a3b53aa` `ff6e069` `ba3c9f1` | 4 files ≥50 LOC blocks relocated (`CastRng`, `TempoResistance`, `Energy/RoundEnergyTracker`, `SignalBus`); ~287 LOC moved. `SignalBus` required rewrite of private `try_consume` assertion as integration via emit/observe. |

### Remaining backlog

1. **W0c — Tier-B inline `mod tests`** (~281 LOC across 7 files): `runtime/registry.rs` (46), `combat/state.rs` (44), `windowed/render.rs` (44), `mechanics/sp.rs` (41), `runtime/event_filter.rs` (39), `mechanics/modifiers.rs` (35), `windowed/mod.rs` (32). 3 of these are `windowed/*` and need `#[cfg(feature = "windowed")]` on the relocated file.
2. **W0c — Tier-C inline `mod tests`** (~159 LOC across 6 files): `headless.rs` (30), `mechanics/buffs.rs` (29), `mechanics/stun.rs` (28), `encounter/bootstrap.rs` (27), `bin/combat_cli.rs` (25), `observability/log.rs` (20). Diminishing return — R003 only mandates >100 LOC blocks be relocated.
3. **Impl-coupling debt** — 93% of test files import `bevyrogue::combat::*`. Separate slice could introduce a typed test-API surface.
4. **Backward R003 direction** (`tests/` → `src/`) — for cases where a test is truly unit-level on a pure function, allow back-relocate when it improves cohesion. Separate slice (architectural, not cleanup).

## Safeness-first audit (post-closure verification)

Re-audited all 14+ commits from `4fee00a` through `ba3c9f1`. Every delete falls into one of three justification classes:

1. **Coverage citation with `file:line`** — e.g. W5 `status_cleanse_policy.rs` ("triple-covered by status_blessed.rs:109-122 + cleanse_effect.rs:94-116 + properties.rs:85-111"); W4 holy_support tautology.
2. **Structural tautology argument** — e.g. W6a "`UltimateUsed` is gated by `UltEffect::Reset`, set only in the ult skill path: Basic and Skill intents structurally cannot emit it"; W5 "5 tests asserting `HashMap::get` on a thin wrapper".
3. **Reclassified delete → relocate** when coverage could not be proven — W0b-1 verify step downgraded the wave from pure-delete to mixed (5 tests rewritten as 3 deletes + 2 relocates); W0b-2 fully reclassified to relocate-only (12+22 tests moved, 0 deleted).

Strongest example of safeness-first: **Wave 8 V4 fragility rewrite was annulled** after verify proved `battery_loop_kernel::last_transition` is a typed observability contract surface, not an internal latch. Documented as D026 + P005 to prevent future ricorrence.

Verdict: **safeness-first respected**. No delete without (a) coverage citation, (b) structural tautology proof, or (c) downgrade to relocate. Full suite green between every commit (14/14). No fabricated coverage claims, no aggressive deletions.

## Workflow artifacts

- `INVENTORY.md` — verified per-file accounting (final).
- `PLAN.md` — 14-sub-wave execution plan (final, Progress table populated).
- `SUMMARY.md` — this file.
- `VERIFICATION.md` — per-claim audit log (read-only, pre-execution).
- `scans/repomix-src-tests.md` — compressed src+tests dump (110k tokens).
- `STATE.json` — phase tracking, marked complete at HEAD `6c7b210`.

All numeric claims here are reconciled against `git log --shortstat` and `cargo test --tests` output at HEAD.
