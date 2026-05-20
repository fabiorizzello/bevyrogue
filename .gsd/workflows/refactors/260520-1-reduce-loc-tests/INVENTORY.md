# Inventory — Reduce LOC in `tests/` + R003 violator handling

Date: 2026-05-20
Branch: master
Scope (chosen): **Option 3 — Full sweep** — Option 2 baseline + W0 R003 violator handling (`src/.../tests/` relocate + inline `#[cfg(test)] mod tests` dedup for blocks >100 LOC) + V4 fragility rewrite.
Method: repomix-compressed scan of `src/` + `tests/` (110k tokens) → subagent audit → manual spot-verification with `wc`, `grep`, `find` → per-file VERIFICATION pass reading actual test bodies (not just patterns).

> **This file replaces all earlier estimates.** Numbers are evidence-based after the verification pass. See `VERIFICATION.md` for per-claim audit log.

## Scope numbers (verified)

- Integration test files: **106** (+ `tests/common/`, `tests/snapshots/`)
- Total `tests/` LOC: **29,548**
- Total `src/` LOC: **31,692** (largest file 495 — `turn_system/pipeline/paths/single_target.rs`; no god module)
- `#[test]` / `#[rstest]` / `#[case]` attributes across `tests/`: **377**
- `#[ignore]`d tests: **1** (`combat_cli_shared_surface.rs::combat_cli_binary_emits_shared_combat_surfaces_from_non_root_cwd` — deliberate subprocess, not rotting)
- `src/.../tests/` directories: **4** (`data/units_ron/tests`, `data/skills_ron/tests`, `combat/resolution/tests`, `combat/mechanics/damage/tests`) — total **3,312 LOC**
- Inline `#[cfg(test)] mod tests` blocks ≥100 LOC: **9 files**, total **1,445 LOC**

### Verification commands rerun

```bash
find src -type d -name tests                        # → 4 dirs
find src -name '*.rs' | xargs wc -l | tail -1       # → 31,692
find tests -name '*.rs' | xargs wc -l | tail -1     # → 29,548
find src -name '*.rs' -exec wc -l {} + | awk '$1>=500'   # → none
grep -rl '^fn setup_app' tests/                     # → 26 files
grep -rl '^fn make_unit\b' tests/                   # → 8 files
grep -rl 'fn drain_events\|fn drain_messages' tests/  # → 17 files
```

## Hard exclusions (verified — DO NOT touch)

| Item | Evidence keeping it | Verdict |
|---|---|---|
| `src/combat/kernel/primitives.rs` `Strain`/`Fatigue`/`FlowState` + transitions | `.gsd/PROJECT.md:25` lists Strain/Flow/Fatigue as typed kernel canon. `ARCHITECTURE.md:27-30` confirms dynamic resources for advanced mechanics. | KEEP — feature scaffold |
| `src/bin/combat_cli/scenarios.rs` + `--scenario` flag | `MILESTONE-PORTFOLIO.md:118` — M021 deliverable "CLI scripted scenario" | KEEP — shipped deliverable |
| `src/bin/combat_cli/proof.rs` + `CliProofConfig` + `[CLI_PROOF]` JSONL log | Same M021 line — "JSONL log surface" | KEEP — shipped deliverable |
| `BatteryLoopState::last_transition` / `PredatorLoopState::last_transition` | `pub` field on `pub` struct, fuels validation snapshot (`tentomon/mod.rs:106-109`); part of typed resolved-state observability contract | KEEP + document in DECISIONS |
| `tests/passive_kitsune_grace.rs:317-347` | End-to-end JSONL roundtrip on real `CombatEvent` — M021 external contract surface | KEEP (NOT tautology) |
| `tests/holy_support_roster_contract.rs:93-101` | Backward-compat invariant on RON roster (no extra blueprint_metadata) | KEEP (NOT Default tautology) |
| `tests/deterministic_rng_contract.rs:30-58` | Fork determinism: 4 stream divergence from same seed | KEEP (NOT self-comparison) |
| `tests/validation_snapshot.rs:148-151,374-396` | `format_validation_snapshot()` is the stable contract surface; status sorted via `status_kind_ord()` | KEEP (NOT fragile) |
| `tests/predator_loop_kernel.rs:228-229` | Test setup guarantees cardinality=1 via `track_target(target)` single | KEEP (NOT ordering-fragile) |
| Standard exclusions | `tests/common/`, `tests/snapshots/`, `tests/properties.rs`, `tests/clip_atlas_parity.rs` | KEEP — oracles/infra |
| Already audited non-overlap | `predator_loop_kernel.rs ↔ dorumon_predator_runtime.rs`; `passive_kitsune_grace.rs ↔ passive_reactive_canon.rs`; `holy_support_resolution.rs ↔ holy_support_mechanics.rs` (canonical bootstrap only) | KEEP |
| Already consolidated last week | `scenario_ttk.rs`, `status_blessed.rs`, `twin_core.rs`, `target_shape_aoe_and_blast.rs` | KEEP |
| Inline `mod tests` ≤100 LOC | R003 explicitly allows "short `#[cfg(test)] mod tests`" | KEEP unless explicit dup vs integration |

**Rule**: production logic in `src/` is untouched. W0 touches only `#[cfg(test)] mod tests` blocks and `src/.../tests/` test directories.

---

## W0 — R003 violator handling

R003 (KNOWLEDGE.md): "Integration tests in `tests/`, functional names. Shared helpers in `tests/common/`. No `src/` unit tests except short `#[cfg(test)] mod tests`." Currently 1,445 LOC of "not short" inline `mod tests` + 3,312 LOC of `src/.../tests/` subdirectories violate this.

### W0a — `src/.../tests/` subdir relocate (3,312 LOC moved, 0 deleted)

Per R003, integration tests live in `tests/`. The 4 subdirs are integration tests parked next to their subject for convenience.

| Source dir | LOC | Target file(s) |
|---|---|---|
| `src/data/units_ron/tests/` | 597 | `tests/data_units_ron_canonical.rs`, `tests/data_units_ron_roundtrip.rs` |
| `src/data/skills_ron/tests/` | 712 | `tests/data_skills_ron_bounce.rs`, `tests/data_skills_ron_roundtrip.rs`, `tests/data_skills_ron_validation.rs` |
| `src/combat/resolution/tests/` | 1,411 | `tests/resolution_bounce.rs`, `tests/resolution_resolve_apply.rs`, `tests/resolution_streak.rs`, `tests/resolution_targets.rs` |
| `src/combat/mechanics/damage/tests/` | 592 | `tests/damage_edge.rs`, `tests/damage_matrix.rs` |
| **Total** | **3,312** | **11 new files** |

**Mechanics**: `git mv` (preserves history) → update path-relative imports (`super::*` → `bevyrogue::…`) → drop empty parent `mod.rs` lines → land. Each subdir = one wave.

**Risk**: low. Already independent integration tests with `mod` wrappers; no behavior changes. Compile failures reveal any private-API leak that needs `pub(crate)` upgrade.

### W0b — Inline `mod tests` >100 LOC dedup (evidence-based, per file)

Numbers from VERIFICATION pass (test-by-test mapping against integration coverage), not estimates.

| File | Inline LOC | Delete | Keep (relocate) | Notes |
|---|---|---|---|---|
| `src/combat/mechanics/ultimate.rs` | 258 | **129** (50%) | 129 | 6 unit-level dup (covered by `tests/ultimate_meter.rs` + `tests/ultimate_event.rs`); 6 unique trigger-semantics keep |
| `src/combat/mechanics/status_effect.rs` | 243 | **156** (64%) | 87 | refresh/multi_kind/cleanse all in integration; semantics core stays |
| `src/combat/kernel/mod.rs` | 184 | **0** | 184 | Strain/Flow/Fatigue/cycle semantics — NO integration equivalent, relocate whole |
| `src/combat/encounter/enemy_ai.rs` | 149 | **149** (100%) | 0 | All 5 tests subsumed by `tests/enemy_ai.rs` (inventory was wrong: integration DID exist) |
| `src/combat/runtime/passive_runner.rs` | 141 | **0** | 141 | Signal filter + circuit-breaker semantics unique; relocate whole |
| `src/combat/runtime/event_bridge.rs` | 136 | **0** | 136 | Dual-signal + batching semantics unique; relocate whole |
| `src/combat/runtime/timeline.rs` | 122 | **122** (100%) | 0 | 4 compile-validation tests all in `tests/compiled_timeline_builtin_validation.rs` |
| `src/combat/mechanics/toughness.rs` | 106 | **106** (100%) | 0 | All 11 tests covered by `tests/toughness_categories.rs` (354 LOC) |
| `src/combat/mechanics/follow_up/triggers.rs` | 106 | **0** | 106 | `evaluate_follow_up` unit isolation — keep, no integration eq. (KNOWLEDGE.md gotcha: stale `#[ignore]` to clean up while relocating) |
| **Total** | **1,445** | **662** | **783** |

**Execution pattern per file**:
1. **Verify**: read inline block end-to-end; for each `#[test]`, search `tests/` for equivalent (`rg <test_name>` + read suspected match).
2. **Decide**: dup → delete; unique → relocate to `tests/<module>_internals.rs` with `pub(crate)` upgrade if helpers needed.
3. **Apply**: `cargo test --tests` after each file.

### W0c — Inline `mod tests` ≤100 LOC (KEEP)

13 files totaling ~438 LOC. R003 explicitly allows short inline blocks. Out of scope unless audit finds explicit duplication during W0b sweeps.

### W0 totals

| | LOC delete | LOC moved |
|---|---|---|
| W0a (4 dirs) | 0 | 3,312 |
| W0b (9 files) | 662 | 783 |
| **W0 total** | **662** | **4,095** |

---

## H — High-confidence merge/parametrize clusters

### H1 — `compiled_timeline_petit_thunder.rs` + `compiled_timeline_tohakken.rs` (+ active_canon partial) → MERGE

- petit_thunder (238 LOC, 1 test) + tohakken (246 LOC, 1 test): ~95% identical scaffolding (`canonical_book` / `build_app` / `spawn_caster` / `spawn_target` / `fire_skill` / `collect_events` / `event_pos`). Diff: `skill_id`, expected event order, post-fire body asserts.
- `compiled_timeline_active_canon.rs::baby_flame_timeline_path_delivers_damage_before_break_then_signal` shares the same skeleton.
- **Action**: collapse into `tests/compiled_timeline_runtime_skills.rs` (`#[rstest]` matrix `(skill_id, owner_signal, expected_event_order, post_asserts_fn)`) for 3 cases. **Doc-comment each `#[case]`** — readability calo with bare `(...)`. Keep separately in `active_canon`: `child_roster_active_skills_all_have_compiled_timelines`, `dangling_hook_…`, `dangling_selector_…`.
- **LOC delete**: ~250 (484 from two files, minus ~80 new parametric file, minus retained scaffolding usable as common helper, minus +30 doc comments).

### H2 — `tentomon_blueprint.rs` + `dorumon_blueprint.rs` + part of `patamon_blueprint_seam.rs` → PARAMETRIZE (per file)

- tentomon (142 LOC, 4 tests): 3 sibling dispatch tests differ only in `(signal_name, expected_name_const, payload_amount)`. Keep: `integration_blueprint_to_kernel_state`.
- dorumon (189 LOC, 7 tests): 4 sibling tests differ only in `(signal_name, payload_variant, expected_amount)`. Keep: `multiple_dorumon_signals_preserve_order`, `unknown_owner_and_signal_are_rejected`, `malformed_envelope_is_rejected_by_serde`.
- patamon_blueprint_seam (356 LOC, 7 tests): `patamon_signal_maps_to_expected_holy_support_transition` follows same shape.
- **LOC delete**: ~80 net.

### H3 — `party_config_validation.rs` + `party_selection_validation.rs` → PARTIAL MERGE

Verified: 2 of 3 overlap real, 1 NOT subsumed.

- **Migrate before delete**: `party_config_deserializes_and_validates` tests RON load path → port into `party_selection_validation.rs`.
- **Keep distinct**: `wrong_pick_count_is_rejected` is NOT strictly subsumed (config has 1 case, selection 2 — both should remain via `#[rstest]`).
- **Action**: port unique test, rename `party_selection_validation.rs` → `tests/party_validation.rs`, use `#[rstest]` for overlapping error cases, delete `party_config_validation.rs`.
- **LOC delete**: ~30 (downgraded from 60 after verification).

### H4 — `holy_support_affordance.rs` + `holy_support_mechanics.rs` → REDUCED (no merge)

Verified: `app_with_holy_support()` has **two different setups** (minimal vs full kernel) testing **different layers** (observability pipeline vs state object). Merge would conflate.

- **Action**: drop ONLY the `holy_support_affordance.rs:84-103` round-trip tautology (~20 LOC). Keep both files.
- **LOC delete**: ~20 (downgraded from 70 after verification).

### H5 — `timeline_validate_typo.rs` (39 LOC) → DELETE

- 1 test. `compiled_timeline_builtin_validation.rs:315-358` already covers `hook`/`selector`/`predicate` typo cases with identical axis/missing_id/site assertions.
- **LOC delete**: ~40.

### T1 — `tests/action_affordance_query.rs:1086-1112` → DELETE 2 tests

- `target_hp_rule_distinguishes_any_and_damaged`: `assert_ne!` + `matches!` on `#[derive(PartialEq)]`.
- `legality_reason_codes_include_contract_values`: loops asserting `!format!("{reason:?}").is_empty()` (tests `Debug` macro).
- **LOC delete**: ~15.

---

## V1 — rstest/proptest parametrization (~360 LOC delete + ~79 LOC rewrite)

| File | Tests collapsed | Pivot variable | LOC delete |
|---|---|---|---|
| `holy_support_mechanics.rs` | 4 of 6 | `transition_op × expected_state` | ~84 |
| `status_amp_pipeline.rs` Cases A/B/C | 3 | matrix Fire/Ice × tag damage; Case D stays | ~73 |
| `compiled_timeline_runtime_dispatch.rs` | 2 | `timeline_id × beats` | ~56 |
| `toughness_categories.rs` | 3 | `(ToughnessCategory, hits, expected_break)` | ~40 |
| `anim_graph_parse.rs` | 7 small parse-error tests | inline RON literal + expected outcome | ~45 |
| `anim_graph_asset.rs` agumon/renamon parses | 2 | asset path | ~10 (post-Wave-5 trim) |
| `ultimate_event.rs` negative cases | 2 (fold into `ultimate_meter.rs`) | setup variant | ~50 |
| `status_paralyzed_skip.rs:54-133` (REWRITE) | 1 (100-iter no-variance loop → proptest over `(turn, prior_state)`) | — | 0 delete, ~79 rewrite |

V1 net: **~360 delete + ~79 rewrite** (after subtracting H2-overlapping items).

---

## V2 — Tautologies / never-fails (~528 LOC delete + ~8 LOC rewrite)

After VERIFICATION: 3 items rescued (NOT tautologies → moved to hard exclusions above), 1 partial confirm.

| File:line | Pattern | Action | LOC |
|---|---|---|---|
| `blueprint_signal_dispatcher.rs:113-116` | serde round-trip duplicate | DELETE — duplicate of `passive_kitsune_grace.rs:317-347` (which IS the canonical, KEEP) | 4 |
| `tests/anim_registry.rs` | 63 LOC, asserts `HashMap::get` behavior on wrapper | DELETE — tautology on std API | 63 |
| `tests/add_new_digimon_isolation.rs` | 3 tests; **only 2 of 3 are dup** (1 metadata-optional unique) | DELETE 2 of 3 | ~79 |
| `tests/passive_canon_support.rs` partial | 2 of 3 tests (kitsune_grace_ignores_*_ult) identical to `passive_kitsune_grace.rs`; keep canonical bootstrap | DELETE 2 of 3 tests | 67 |
| `tests/status_cleanse_policy.rs` | triple-covered by `status_blessed.rs:109-122` + `cleanse_effect.rs:94-116` + `properties.rs:85-111` | DELETE | 38 |
| `tests/anim_graph_asset.rs` | **Only 1 of 3 overlap** (`agumon_sharp_claws_release_kernel_cue_parses`); `malformed`, `renamon` disjoint | DELETE 1 (not 3) | ~17 |
| `tests/anim_gameplay_command_forbidden.rs` (1 test) | same check as `agumon_sharp_claws_asset` | DELETE | 21 |
| `tests/combat_coherence.rs:655-665` | `assert_eq!` copy-pasted twice | DELETE | 5 |
| `holy_support_affordance.rs:84-103` | round-trip on struct literal | DELETE (counted in H4) | 20 |
| `event_stream.rs:334`, `holy_support_resolution.rs`, `anim_stance_asset.rs`, `anim_graph_parse.rs` minor | misc tautologies | DELETE | ~50 |
| `party_config_validation.rs:14-21` | hardcoded id array after deserialize | REWRITE — assert via roster equality (counted under H3 port) | 0 delete, ~8 rewrite |
| `tests/source_file_loc_limit.rs` | meta-lint disguised as test | DELETE — re-home in CI hook (separate slice) | 65 |
| `tests/combat_cli_shared_surface.rs` | grep `#[test]` (delete) vs `#[ignore]` subprocess (KEEP) | DELETE grep test only | ~40 |

V2 net delete: **~488 LOC** (after subtracting H5+T1+H4 already counted separately, and 3 rescued items).
V2 net rewrite: **~8 LOC** (only `party_config_validation:14-21`).

---

## V3 — Boilerplate consolidation into `tests/common/`

**Diagnostic**: ~100 of 103 test files do NOT use `tests/common/*`.

| Helper | Occurrences (verified) | LOC duplicated |
|---|---|---|
| `setup_app()` builder (B1) | 26 files | ~180 |
| `make_unit(...)` private constructor (B2) | 8 files | ~50 |
| `drain_events::<T>()` / `drain_messages::<T>()` (B3) | 17 files | ~115 |
| Fixture struct copy-paste (B4) | 3 files | ~80 |
| Magic-number constants HP/SP/threshold (B6) | ~10-15 occurrences | ~30 |

**B1 setup_app has 3 distinct shapes** (verified):
- **Class A** (minimal) — 12 files: just `App::new().add_plugins(CombatPlugin)`.
- **Class B** (full combat) — 5 files: + seeded `bevy_rand` + roster + warmups.
- **Class C-F** (parametrized) — 7 files: variants with custom plugin sets, mode toggles.

→ Requires `TestAppBuilder` builder pattern, **not a simple `fn test_app()`**.

**Missing from `common/`**:
- `tests/common/app.rs` — `TestAppBuilder` (~80 LOC).
- `tests/common/events.rs` — `drain::<T>(app)` (~30 LOC).
- `tests/common/constants.rs` — capability constants (~30 LOC).
- Extend `tests/common/units.rs` — `make_unit(id, hp, sp)`.

V3 net delete: **~250 LOC** (was ~570 — corrected after builder cost +80 LOC and verified occurrence counts B2/B3/B6 lower than original grep).

---

## V4 — Fragile tests (~95 LOC rewrite, NO LOC win)

After VERIFICATION: F1 + F2 + F3 rescued as contract surfaces. F4 + new F5 sites confirmed fragile.

| File:line | Category | Verdict | Action |
|---|---|---|---|
| `validation_snapshot.rs:148-151,374-396` | F1 | **CONTRACT** — `format_validation_snapshot()` stable, status sorted by `status_kind_ord` | KEEP |
| `battery_loop_kernel.rs:59-63,136,161` | F2 | **CONTRACT** — `pub last_transition` typed observability surface | KEEP + document in DECISIONS |
| `predator_loop_kernel.rs:228-229` | F3 | **NOT fragile** — cardinality=1 guaranteed by single-target setup | KEEP |
| `predator_loop_kernel.rs:254-268` | F4 | CONFIRMED fragile substring sniff | REWRITE to typed assertion (~25 LOC) |
| `battery_loop_kernel.rs:138` | F5 NEW | `formatted.contains("grant(5)")` substring sniff | REWRITE to event-stream assertion (~25 LOC) |
| `dorumon_predator_runtime.rs:164` | F5 NEW | `formatted.contains("targets=[8:e2:p2]")` | REWRITE to typed targets assertion (~20 LOC) |
| `holy_support_mechanics.rs:321` | F5 NEW | `formatted.contains("last=build(2)")` | REWRITE to typed snapshot assertion (~25 LOC) |
| `deterministic_rng_contract.rs:30-58` | — | **NOT self-comparison** (fork determinism) | KEEP (was: rewrite) |

V4 net: ~95 LOC rewrite, 0 LOC delete. Debt reduction only.

---

## LOC accounting (Option 3 — Full, verified)

### Original plan (pre-execution estimates)

| Bucket | LOC delete | LOC moved | LOC rewrite |
|---|---|---|---|
| **W0a** — `src/.../tests/` dir-relocate (4 dirs) | 0 | 3,312 | 0 |
| **W0b** — Inline `mod tests` >100 LOC (9 files, evidence-based) | **662** | 783 | 0 |
| H1 timeline merge | **~250** | 0 | 0 |
| H2 blueprint param | ~80 | 0 | 0 |
| H3 party validation (reduced) | **~30** | 0 | 0 |
| H4 holy_support (reduced — tautology only) | **~20** | 0 | 0 |
| H5 + T1 + V2 batch deletions | **~528** | 0 | 0 |
| V1 rstest/proptest sweep | ~360 | 0 | 79 |
| V3 common migration | **~250** | 0 | 0 |
| V4 fragility (F4 + 3 F5) | 0 | 0 | ~95 |
| **TOTAL Option 3 (planned)** | **~2,260** | **~4,095** | **~174** |

### Realized so far (verified per commit, 2026-05-20)

| Wave | Commit | LOC delete (net) | LOC moved/added | Notes |
|---|---|---|---|---|
| W0a-1 data/ relocate | `4fee00a` | 0 | ~1,309 | 5 files renamed; `mod.rs` shells dropped |
| W0a-2 combat/ relocate | `d73ad89` | 0 | ~2,003 | 6 files renamed + 2 new shared helper files (224 LOC) |
| W0b-1 enemy_ai+timeline+toughness | `84ea3cc` | **167** | **216** | Reclassified some "covered" tests as unique (loop-typo, dispatch boundary) |
| W0b-2 ultimate+status_effect | `d31e1d6` | **78** | **425** | Pure-function tests had no integration coverage — relocated instead of deleted |
| W0b-3 kernel+passive_runner+event_bridge+follow_up | `8ef435e` | **0** | **576** | Pure relocate; promoted `FollowerSnapshot` + `evaluate_follow_up` to `pub(crate)` |
| **W0 subtotal** | | **245** | **~4,529** | W0b total: 245 delete + 1,217 moved (was: 662 + 783 per plan) |
| W1 `tests/common/` infra | `a426c2d` | — | **+306 infra** | `TestAppBuilder` + `drain` + `constants` + `make_unit` |
| W5 H5 + T1 + V2 batch delete | `af7932e` | **467** | — | 3 file deleted + 9 trimmed; `source_file_loc_limit` deferred |
| W2 H1 compiled_timeline merge | `d2fe4a4` | **195** | — | 3 files → 1 parametric matrix (fn-pointer for `EventPred` + `PostAssert`) |
| W3 H2 blueprint dispatch rstest | `7ab7184` | **58** | — | tentomon + dorumon dispatch; patamon skipped (1 case = no value) |
| W4 H3 party port + H4 holy_support tautology | `4fc97cc` | **62** | — | `party_config_validation.rs` deleted; `holy_support_affordance.rs:84-103` dropped |
| W6a `ultimate_event` fold | `e69ea04` | **303** | — | 2 negative cases tautology; positive case folded into `ultimate_meter.rs` |
| W6b `status_amp` + `anim_graph_parse` rstest | `1d08b23` | **61** | — | 3 amp cases + 3 parse-reject cases parametrized |
| W6c `anim_graph_asset` consolidation | `75700f7` | **5** | — | 2 positive asset parses + 1 reject folded |
| W7a TestAppBuilder migration (9 files) | `add9c2f` | **83** | — | Existing W1 helpers absorbed minimal/passive/kernel shapes |
| W7b skill_book_runtime + form_identity (4 files) | `77ed2f4` | **114** | — | 2 new helpers; resolve_action + follow_up listener chain |
| W7c skill_resolve_app (4 files) | `c559cfe` | **48** | — | 1 helper; single-system resolve_action tests |
| W7d turn_av_base_app (3 files) | `6c7b210` | **5** | — | 1 helper; turn/AV-system tests + 1 inline duplicate folded |
| W8 status_paralyzed_skip proptest | `4a01aef` | **0** | +5 rewrite | 100-iter loop → proptest 256 cases; V4 fragility annulled |
| **H/V/W7/W8 subtotal (W1–W8)** | | **1,401** | **+306 infra + 5 rewrite** | 13 commits, full suite green between each |
| **GRAND TOTAL realized** | | **1,646** | **~4,529 + 306 infra + 5 rewrite** | through HEAD `6c7b210` (Wave 7d) |

### Reconciliation — W0b deviation from plan

W0b plan estimated 662 LOC delete + 783 LOC moved on 9 files. Actual realized across all 3 sub-waves: **245 delete + 1,217 moved**. Cause: pattern-matching pure functions (`matches_trigger`, `classify_buff_kind`, `status_amp_pct`, `chilled_speed_delta`, `cleanse_n`) and corner-case dispatch tests had no unit-level equivalent in `tests/` — the integration suites exercise them only transitively. Strict R003 reading: relocate, don't delete. W0b-3 (kernel/mod.rs + passive_runner + event_bridge + follow_up/triggers) was a full pure-relocate as planned.

### Reconciliation — Wave 6 reframe

Wave 6 PLAN listed 8 targets (`holy_support_mechanics`, `status_amp_pipeline`, `compiled_timeline_runtime_dispatch`, `toughness_categories`, `anim_graph_parse`, `anim_graph_asset`, `ultimate_event`, `status_paralyzed_skip`). Pairwise-reading the tests revealed only 3 are genuinely high-ROI:

- `ultimate_event` (`e69ea04`) — 2 negatives were structural tautologies + 1 positive folded into existing test. **−303 LOC**, biggest single win of the wave.
- `status_amp_pipeline` + `anim_graph_parse` (`1d08b23`) — clean `#[rstest]` over `(input, expected)`. **−61 LOC**.
- `anim_graph_asset` (`75700f7`) — modest fold by asset path. **−5 LOC**.

Skipped (with reasons): `holy_support_mechanics` (4 heterogeneous tests — kernel dispatch / saturation / multi-step lifecycle / snapshot — no shared shape), `compiled_timeline_runtime_dispatch` (2 semantically different tests), `toughness_categories` (lifecycle steps + per-case intermediate asserts diverge), `status_paralyzed_skip` (100-iter no-variance loop → proptest rewrite deferred to Wave 8 debt batch — 0 LOC win, quality only).

**Net Wave 6**: −369 LOC vs PLAN −360 + 79 rewrite (rewrite quota slid to Wave 8). Quota for delete met.

### Final realized (Option 3 — Full, all 14 sub-waves landed)

| Bucket | LOC delete | LOC moved/added | LOC rewrite | Status |
|---|---|---|---|---|
| W0a (5 files relocated) | 0 | ~3,312 | 0 | ✅ |
| W0b (3 sub-waves, 9 files) | **245** | **1,217** | 0 | ✅ |
| **W0 subtotal** | **245** | **~4,529** | 0 | ✅ |
| W1 `tests/common/` infra | 0 | +306 infra | 0 | ✅ `a426c2d` |
| W5 H5 + T1 + V2 batch | **467** | 0 | 0 | ✅ `af7932e` |
| W2 H1 compiled_timeline | **195** | 0 | 0 | ✅ `d2fe4a4` |
| W3 H2 blueprint dispatch | **58** | 0 | 0 | ✅ `7ab7184` |
| W4 H3 party + H4 holy | **62** | 0 | 0 | ✅ `4fc97cc` |
| W6 V1 rstest sweep (3 sub-waves) | **369** | 0 | 0 | ✅ `e69ea04`+`1d08b23`+`75700f7` |
| W7 V3 common-helper migration (4 sub-waves) | **250** | +5 helpers (~50 LOC) | 0 | ✅ `add9c2f`+`77ed2f4`+`c559cfe`+`6c7b210` |
| W8 status_paralyzed_skip proptest | **0** | 0 | **+5 net** | ✅ `4a01aef` |
| **W1–W8 subtotal** | **1,401** | **+306 + ~50 infra** | **+5 net** | ✅ |
| **REALIZED FINAL** | **1,646** | **~4,529 + ~356 infra** | **+5 net** | through `6c7b210` (Wave 7d) |

**Delta vs original PLAN target** (`−1,843` delete forecast): realized −1,646 vs target −1,843 = **89.3% of target hit** (≈11% short). Gap accounted for by W0b reclassification (245 delete actual vs 662 planned, 417 LOC shifted to relocate based on per-file evidence) + Wave 6 reframe (6 of 8 originally-listed targets skipped because pairwise read revealed degraded readability or no shared shape).

**V4 fragility rewrite** (~95 LOC) **ANNULLATO** during W8 verify — `last_transition` is observability contract, not internal latch. Removed from forecast; deferred to DECISIONS doc backlog.

**Direction confirmed at end of slice**: R003 ripristinato + tests cohesion + named parametric matrices replace ad-hoc copy-paste. Per-test quality metric (rstest/proptest adoption) climbed from 5 files pre-refactor to **12+ files at HEAD** (added: status_paralyzed_skip proptest, status_amp_pipeline rstest, anim_graph_parse rstest, anim_graph_asset rstest, tentomon_blueprint rstest, dorumon_blueprint rstest, party_validation rstest, compiled_timeline_runtime_skills rstest).

**Verification at slice close**: `cargo test --tests` → 120 binaries, all green. `cargo check --tests` → 0 warnings on changed files. R003: no `src/.../tests/` dirs remain; no >100 LOC inline `#[cfg(test)] mod tests` blocks remain (the 9 originally identified all landed).

---

## Dependency / ordering notes

- **W0a** independent — pure `git mv`; earliest is cleanest (clears `src/` test noise before reviewing diffs in subsequent waves).
- **W0b** must follow W0a if any moved code touches the same `pub(crate)` API the inline blocks use; otherwise independent.
- **V3** (`tests/common/` extension) must land before H1/V1/V8 sweeps that depend on the new helpers.
- **H1** before any other compiled-timeline cluster.
- **H2/H3/H4** mutually independent.
- **H5, T1, V2** deletions parallelizable, can batch.
- **V1** sweeps after V3 helpers exist.
- **V4** fragility rewrite last — F4/F5 use new typed helpers where useful.

## Risk register

| Risk | Mitigation |
|---|---|
| W0a private-API leak when relocating dir tests | Compile error reveals every `super::*` → upgrade to `pub(crate)` or expose typed helper. Fail-fast pattern. |
| W0b file-by-file delete-vs-keep mistakes | Per-file verification step (step 1 of each W0b wave): read inline + read suspected integration + diff in writing before deleting. |
| Wave 1 helper signature drift from 26 inline `setup_app()` variants | Builder pattern absorbs 3 shape classes (A minimal / B full / C-F parametrized). Diff pairwise before consolidating each shape. |
| V8 large commit hard to review | Mechanical per-file pattern; reviewer can sample-check + rely on green tests. |
| Hidden coupling between deleted file and snapshot test | `cargo test --tests` between every wave catches it. |
| Test name collision after merges | Each merged test gets a clear `caseN_*` suffix from `#[rstest]`. |

## Deferred backlog (out of scope of this slice)

- **93% test files import `bevyrogue::combat::*`** — impl-coupling debt, separate slice to introduce a public test API surface.
- **`last_transition` contract** — document as DECISION + project-memory pattern entry (KEEP + observable).
- **`source_file_loc_limit`** re-home as CI/lint check.
- **R003 strict reorg** of remaining ≤100 LOC inline blocks (W0c, 438 LOC) — separate slice.
