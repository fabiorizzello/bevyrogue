# Plan — Reduce LOC: tests + R003 violators (Option 3 — Full)

Scope: **Option 3 — Full sweep** — Option 2 + W0 (W0a dir-relocate + W0b inline `mod tests` dedup) + V4 fragility.

Target (revised after W0a + W0b-1 + W0b-2 landed — see INVENTORY.md "Realized so far" + "Reconciliation"):
- **~1,843 LOC delete** (was 2,260; W0b reclassified ~417 LOC from delete to relocate based on per-test evidence)
- **~4,520 LOC moved** (was 4,095; same cause)
- **~174 LOC rewrite** (V3 helpers + V4 fragility + H3 RON port — unchanged)

### Progress

| Wave | Status | Commit | Actual delete | Actual moved/added |
|---|---|---|---|---|
| W0a-1 `data/` relocate | ✅ | `4fee00a` | 0 | ~1,309 |
| W0a-2 `combat/` relocate | ✅ | `d73ad89` | 0 | ~2,003 |
| W0b-1 enemy_ai + timeline + toughness | ✅ | `84ea3cc` | 167 | 216 |
| W0b-2 ultimate + status_effect | ✅ | `d31e1d6` | 78 | 425 |
| W0b-3 kernel + passive_runner + event_bridge + follow_up | ✅ | `8ef435e` | 0 | 576 |
| W1 `tests/common/` infra (TestAppBuilder + drain + constants) | ✅ | `a426c2d` | — | +306 infra |
| W5 H5 + T1 + V2 batch delete | ✅ | `af7932e` | 467 | — |
| W2 H1 compiled_timeline merge | ✅ | `d2fe4a4` | 195 | — |
| W3 H2 blueprint dispatch rstest | ✅ | `7ab7184` | 58 | — |
| W4 H3 party port + H4 holy_support tautology | ✅ | `4fc97cc` | 62 | — |
| W6a ultimate_event fold → ultimate_meter | ✅ | `e69ea04` | 303 | — |
| W6b status_amp_pipeline + anim_graph_parse rstest | ✅ | `1d08b23` | 61 | — |
| W6c anim_graph_asset fold + reject case | ✅ | `75700f7` | 5 | — |
| W7a migrate setup_app to TestAppBuilder (9 files) | ✅ | `add9c2f` | 83 | — |
| W7b skill_book_runtime + form_identity_runtime helpers (4 files) | ✅ | `77ed2f4` | 114 | — |
| W7c skill_resolve_app helper (4 files) | ✅ | `c559cfe` | 48 | — |
| W7d turn_av_base_app helper (3 files) | ✅ | `6c7b210` | 5 | — |
| W8 status_paralyzed_skip proptest rewrite | ✅ | `4a01aef` | 0 | 5 (rewrite) |

**Realized through W8**: W0 done (245 delete + ~4,529 moved); H/V batch (W1–W6) done (1,151 delete + 306 infra); W7 done (250 delete across 4 sub-waves + 4 new common helpers); W8 done (proptest rewrite, 0 LOC net). R003 restored on all originally identified violators. **V4 fragility rewrite (originally planned for W8) ANNULLATO** — `battery_loop_kernel::last_transition` confirmed as observability contract (see commit `4a01aef` body for full evidence chain). All sub-waves green between commits.

Each wave = one atomic commit. Verify gate between waves: `cargo test --tests` (nextest `agent` profile) must be green. No `--no-verify`, no skips.

Conventional commit prefix per wave: `refactor(tests): wave N — <slug>`.

**Universal wave pattern**:
1. **Verify (read-only)** — re-confirm the per-item assumption is still true given current HEAD: read inline block + suspected integration equivalent, diff in writing in the wave's commit message body. If any item fails verification, drop it from the wave and note the reason; the rest of the wave proceeds.
2. **Apply** — delete / move / rewrite.
3. **Test gate** — `cargo test --tests` (scoped first, full last).
4. **Land** — atomic commit.

> The verify step is non-skippable. It is what catches the "looked like a tautology, was actually a contract" failure mode that VERIFICATION caught for `last_transition`, `holy_support_roster_contract`, etc.

---

## Wave 0a — Relocate `src/.../tests/` directories (R003)

Pure `git mv` waves. Splits in two by layer for review-friendly diffs.

### Wave 0a-1 — `data/` test dirs

**Step 1 — Verify** (read-only):
- `find src/data -type d -name tests` → must still show `units_ron/tests` + `skills_ron/tests`.
- For each `.rs` file in those dirs: confirm `use super::*;` / `use crate::…` paths and list every private item touched (`grep -E 'pub\(crate\)|pub fn|pub struct' …` on parent module).

**Step 2 — Apply**:
- `git mv src/data/units_ron/tests/canonical.rs tests/data_units_ron_canonical.rs`
- `git mv src/data/units_ron/tests/roundtrip.rs tests/data_units_ron_roundtrip.rs`
- `git mv src/data/skills_ron/tests/bounce.rs tests/data_skills_ron_bounce.rs`
- `git mv src/data/skills_ron/tests/roundtrip.rs tests/data_skills_ron_roundtrip.rs`
- `git mv src/data/skills_ron/tests/validation.rs tests/data_skills_ron_validation.rs`
- Delete now-empty `mod.rs` lines / files.
- Rewrite imports: `super::*` → `bevyrogue::data::units_ron::…` / `bevyrogue::data::skills_ron::…`.
- For each compile failure, upgrade the leaked symbol to `pub(crate)` in its module, NOT in a new "test helper" surface.

**Step 3 — Test gate**: `cargo test --tests data_units_ron data_skills_ron` then full `cargo test --tests`.

**Diff stat**: ~597 + 712 = ~1,309 LOC moved, 0 deleted.

**Commit**: `refactor(tests): wave 0a-1 — relocate src/data/{units_ron,skills_ron}/tests/ to tests/`

### Wave 0a-2 — `combat/` test dirs

**Step 1 — Verify**:
- `find src/combat -type d -name tests` → `resolution/tests` + `mechanics/damage/tests`.
- List private items used (same scan as 0a-1).

**Step 2 — Apply**:
- `git mv src/combat/resolution/tests/{bounce,resolve_apply,streak,targets}.rs tests/resolution_*.rs`
- `git mv src/combat/mechanics/damage/tests/{edge,matrix}.rs tests/damage_*.rs`
- Rewrite imports; promote leaked privates to `pub(crate)`.

**Step 3 — Test gate**: `cargo test --tests resolution damage` then full run.

**Diff stat**: ~1,411 + 592 = ~2,003 LOC moved.

**Commit**: `refactor(tests): wave 0a-2 — relocate src/combat/{resolution,mechanics/damage}/tests/ to tests/`

---

## Wave 0b-1 — Inline `mod tests` pure deletions (4 files, 100% subsumed)

Files where VERIFICATION confirmed **every** `#[test]` is covered by an existing integration test.

**Step 1 — Verify** (per file, pasted into commit body):
- `src/combat/encounter/enemy_ai.rs` (149 LOC, 5 tests) — confirm each test name has a counterpart in `tests/enemy_ai.rs`. Read both end-to-end. Diff scenarios.
- `src/combat/runtime/timeline.rs` (122 LOC, 4 tests) — confirm coverage by `tests/compiled_timeline_builtin_validation.rs`.
- `src/combat/mechanics/toughness.rs` (106 LOC, 11 tests) — confirm coverage by `tests/toughness_categories.rs` (354 LOC).
- For each: if even one test is NOT covered, downgrade that file to "partial" and split the action (relocate uncovered tests).

**Step 2 — Apply**:
- Delete the `#[cfg(test)] mod tests { … }` blocks in the 3 files.
- Remove their `use` imports that become dead.

**Step 3 — Test gate**: `cargo test --tests enemy_ai timeline toughness` then full.

**Diff stat**: −377 LOC (149 + 122 + 106).

**Commit**: `refactor(tests): wave 0b-1 — delete inline mod tests subsumed by integration suites`

---

## Wave 0b-2 — Inline `mod tests` mixed delete/relocate (`ultimate.rs` + `status_effect.rs`)

Files where part is duplicate, part is unique.

**Step 1 — Verify**:
- `src/combat/mechanics/ultimate.rs` (258 LOC, 12 tests): for each test, mark COVERED (by `tests/ultimate_meter.rs:1109` LOC or `tests/ultimate_event.rs:328` LOC) or UNIQUE (trigger semantics). VERIFICATION pass expected ~6 covered + ~6 unique.
- `src/combat/mechanics/status_effect.rs` (243 LOC, ~21 tests): mark by coverage in `tests/status_blessed.rs` + `tests/status_amp_pipeline.rs` + `tests/status_paralyzed_skip.rs`. Expected: ~64% covered.

**Step 2 — Apply**:
- For each unique test → relocate to `tests/ultimate_mechanics_internals.rs` / `tests/status_effect_internals.rs`. Promote any helper to `pub(crate)`.
- For each covered test → delete.
- Drop now-orphan helpers / imports from inline block.

**Step 3 — Test gate**: `cargo test --tests ultimate status_effect` then full.

**Diff stat**: −285 LOC (129 + 156); +216 LOC moved (129 + 87).

**Commit**: `refactor(tests): wave 0b-2 — split ultimate.rs + status_effect.rs inline tests into delete + relocate`

---

## Wave 0b-3 — Inline `mod tests` pure relocate (4 files, 100% unique)

Files with no integration equivalent — relocate whole `mod tests` block to `tests/`.

**Step 1 — Verify**:
- `src/combat/kernel/mod.rs` (184 LOC) — confirm Strain/Flow/Fatigue/cycle invariants have NO `tests/` counterpart (`rg 'strain|flow_state|fatigue' tests/`).
- `src/combat/runtime/passive_runner.rs` (141 LOC) — confirm circuit-breaker + signal filter tests not duplicated.
- `src/combat/runtime/event_bridge.rs` (136 LOC) — confirm dual-signal batching not duplicated.
- `src/combat/mechanics/follow_up/triggers.rs` (106 LOC) — confirm `evaluate_follow_up` isolation. **Also**: clean up the stale `#[ignore]` flagged in KNOWLEDGE.md gotcha.

**Step 2 — Apply**:
- For each: extract `#[cfg(test)] mod tests` block into `tests/<file>_internals.rs`.
- Promote helpers to `pub(crate)` as needed.
- Drop the `mod tests` block from the src file.
- In `follow_up/triggers.rs`: remove the stale `#[ignore]` reason or replace with proper skip mechanism while relocating.

**Step 3 — Test gate**: `cargo test --tests kernel_internals passive_runner_internals event_bridge_internals follow_up_triggers_internals` then full.

**Diff stat**: +567 LOC moved (184 + 141 + 136 + 106), 0 deleted.

**Commit**: `refactor(tests): wave 0b-3 — relocate kernel/passive_runner/event_bridge/follow_up inline tests`

---

## Wave 1 — `tests/common/` builder infrastructure (V3 prep)

**Step 1 — Verify**: `grep -l '^fn setup_app' tests/` → 26 files. Pull 5 minimal-class, 3 full-class, 2 parametrized variants side-by-side. Confirm the 3-class taxonomy (A/B/C-F) still holds at HEAD.

**Step 2 — Apply** (new files + extend existing):
- `tests/common/app.rs` — `pub struct TestAppBuilder` with chainable: `.minimal()` (default), `.with_combat()`, `.with_rng_seed(u64)`, `.with_roster(&[…])`, `.with_warmup_ticks(u32)`, `.build() -> App`. Cover all 3 shape classes.
- `tests/common/events.rs` — `pub fn drain<T: Event + Clone>(app: &mut App) -> Vec<T>` and `pub fn drain_messages<T: Message + Clone>(app: &mut App) -> Vec<T>`.
- `tests/common/constants.rs` — `pub const BASIC_HP, BASIC_SP, LOW_HP_THRESHOLD, …`.
- Extend `tests/common/units.rs` — `pub fn make_unit(id, hp, sp) -> UnitBundle`.
- `tests/common/mod.rs` — wire the new modules.

**Step 3 — Test gate**: `cargo test --tests` (no callers yet, must still compile + pass).

**Diff stat**: +~150 LOC (helpers).

**Commit**: `refactor(tests): wave 1 — extend tests/common/ with TestAppBuilder, drain, constants`

---

## Wave 2 — H1 compiled timeline merge

**Step 1 — Verify**:
- Read `tests/compiled_timeline_petit_thunder.rs` + `tests/compiled_timeline_tohakken.rs` + the `baby_flame_…` test in `compiled_timeline_active_canon.rs`.
- Confirm the 3 share scaffolding within ±5% LOC; confirm differences reduce to `(skill_id, owner_signal, expected_event_order, post_asserts_fn)`.
- Pre-write the `#[case]` doc comments to confirm event order is documentable (readability calo mitigation).

**Step 2 — Apply**:
- Add `tests/compiled_timeline_runtime_skills.rs` (~80 LOC `#[rstest]` matrix + 3 doc-commented `#[case]`s).
- Delete `tests/compiled_timeline_petit_thunder.rs`, `tests/compiled_timeline_tohakken.rs`.
- Modify `tests/compiled_timeline_active_canon.rs` — remove `baby_flame_timeline_path_delivers_damage_before_break_then_signal`; keep `child_roster_active_skills_all_have_compiled_timelines`, `dangling_hook_…`, `dangling_selector_…`.

**Step 3 — Test gate**: `cargo test --tests compiled_timeline` then full.

**Diff stat**: −~250 LOC.

**Commit**: `refactor(tests): wave 2 — merge compiled_timeline runtime tests into parametric matrix`

---

## Wave 3 — H2 blueprint dispatch parametrization

**Step 1 — Verify**:
- `tests/tentomon_blueprint.rs` lines 48-103: confirm the 3 dispatch tests differ only in `(signal_name, expected_name_const, payload_amount)`.
- `tests/dorumon_blueprint.rs` lines 54-116: confirm the 4 tests differ only in `(signal_name, payload_variant, expected_amount)`.
- `tests/patamon_blueprint_seam.rs`: confirm same shape for the mapping tests.

**Step 2 — Apply** (no file merges — collapse in place):
- `tentomon_blueprint.rs` — `#[rstest]` over 3 cases; keep `integration_blueprint_to_kernel_state`.
- `dorumon_blueprint.rs` — `#[rstest]` over 4 cases; keep `multiple_dorumon_signals_preserve_order`, `unknown_owner_and_signal_are_rejected`, `malformed_envelope_is_rejected_by_serde`.
- `patamon_blueprint_seam.rs` — `#[rstest]` over mapping; keep non-dispatch tests.

**Step 3 — Test gate**: `cargo test --tests blueprint`.

**Diff stat**: −~80 LOC.

**Commit**: `refactor(tests): wave 3 — parametrize blueprint dispatch tests with rstest`

---

## Wave 4 — H3 party (port + reduce) + H4 holy_support (tautology drop)

VERIFIED-corrected: H3 is partial merge (port 1 unique test before delete), H4 is NOT a merge (different layers) — drop only one tautology block.

**Step 1 — Verify**:
- H3: confirm `party_config_deserializes_and_validates` (in `party_config_validation.rs`) tests RON load path NOT covered in `party_selection_validation.rs`. Confirm `wrong_pick_count_is_rejected` differs between the two (1 case vs 2).
- H4: read both `app_with_holy_support()` setups side-by-side. Confirm they are **distinct layers** (minimal vs full kernel). Confirm `affordance.rs:84-103` is a struct-literal round-trip tautology.

**Step 2 — Apply**:
- H3:
  - Port `party_config_deserializes_and_validates` from `party_config_validation.rs` into `party_selection_validation.rs`.
  - In `party_selection_validation.rs`: add `#[rstest]` for error cases covering both `wrong_pick_count` shapes.
  - Rename `tests/party_selection_validation.rs` → `tests/party_validation.rs`.
  - Delete `tests/party_config_validation.rs`.
  - Rewrite `party_config_deserializes_and_validates:14-21` to assert via roster equality (V2 rewrite, ~8 LOC).
- H4:
  - Delete `tests/holy_support_affordance.rs:84-103` only (~20 LOC tautology). Keep the rest of both files.

**Step 3 — Test gate**: `cargo test --tests party_validation holy_support`.

**Diff stat**: −~50 LOC (~30 H3 + ~20 H4) + ~8 LOC rewrite.

**Commit**: `refactor(tests): wave 4 — port party RON test + drop holy_support tautology`

---

## Wave 5 — H5 + T1 + V2 batch deletions

Pure deletions / in-place test drops. Big diff, mechanical.

**Step 1 — Verify** (per item, all read-only):
- `tests/timeline_validate_typo.rs` (40 LOC) — subsumed by `compiled_timeline_builtin_validation.rs:315-358`? Diff axis/missing_id/site assertions.
- `tests/anim_registry.rs` (64 LOC) — every test asserts `HashMap::get` on wrapper? Read all.
- `tests/add_new_digimon_isolation.rs` (119 LOC) — verify 2 of 3 tests dup (`digimon_signal_registry` + `holy_support_roster_contract`); confirm the 1st (metadata-optional) is unique.
- `tests/passive_canon_support.rs` (236 LOC, 3 tests) — confirm tests 2+3 (`kitsune_grace_ignores_*_ult`) match `passive_kitsune_grace.rs`; confirm test 1 (canonical 4-passive bootstrap) is unique → KEEP.
- `tests/status_cleanse_policy.rs` (39 LOC) — triple-coverage by `status_blessed.rs:109-122` + `cleanse_effect.rs:94-116` + `properties.rs:85-111`.
- `tests/source_file_loc_limit.rs` (65 LOC) — confirm meta-lint, no real assertion of behavior.
- `tests/action_affordance_query.rs:1086-1112` (T1) — derive-only tautologies.
- `tests/anim_graph_asset.rs` — confirm ONLY `agumon_sharp_claws_release_kernel_cue_parses` overlaps; `malformed` and `renamon` disjoint.
- `tests/anim_gameplay_command_forbidden.rs` — confirm the 1 test = same check as `agumon_sharp_claws_asset`.
- `tests/combat_coherence.rs:655-665` — confirm copy-paste twice.
- `tests/combat_cli_shared_surface.rs` — confirm grep `#[test]` block (delete) is distinct from `#[ignore]` subprocess (KEEP).
- `tests/blueprint_signal_dispatcher.rs:113-116` — confirm 4-LOC serde dup is fully covered by `passive_kitsune_grace.rs:317-347` (which IS the canonical, KEEP).
- Minor: `event_stream.rs:334`, `holy_support_resolution.rs`, `anim_stance_asset.rs`, `anim_graph_parse.rs` — each line audited individually.

**Step 2 — Apply** (deletes / in-place trims):
- DELETE files: `timeline_validate_typo.rs`, `anim_registry.rs`, `status_cleanse_policy.rs`, `source_file_loc_limit.rs`.
- In `add_new_digimon_isolation.rs`: delete 2 of 3 tests (~79 LOC).
- In `passive_canon_support.rs`: delete tests 2+3 (~67 LOC); KEEP test 1.
- In `action_affordance_query.rs`: delete lines 1086-1112 (~15 LOC).
- In `anim_graph_asset.rs`: delete the 1 overlap test (~17 LOC).
- In `anim_gameplay_command_forbidden.rs`: delete 1 test (~21 LOC).
- In `combat_coherence.rs`: delete duplicated block (~5 LOC).
- In `combat_cli_shared_surface.rs`: delete grep `#[test]` only (~40 LOC); KEEP `#[ignore]` subprocess.
- In `blueprint_signal_dispatcher.rs`: delete lines 113-116 (~4 LOC).
- Minor cleanups (~50 LOC total).

**Note**: `source_file_loc_limit.rs` re-home as CI/lint check → tracked as deferred backlog, NOT done in this wave.

**Step 3 — Test gate**: full `cargo test --tests`.

**Diff stat**: −~528 LOC.

**Commit**: `refactor(tests): wave 5 — batch delete tautologies and duplicates`

---

## Wave 6 — V1 rstest/proptest parametrization sweep (LANDED in 3 sub-commits)

Originally a single commit; split during execution into 6a/6b/6c after early gains exceeded the target delete budget.

### Wave 6a — `ultimate_event.rs` fold (`e69ea04`)

Folded positive observability assertion into `ultimate_meter.rs::ultimate_fire_resets_meter_and_damages_target`; deleted 2 structural-tautology negative cases (`UltimateUsed` is gated by `UltEffect::Reset`, structurally unreachable from non-ult intents). **−303 LOC**.

### Wave 6b — `status_amp_pipeline.rs` + `anim_graph_parse.rs` rstest folds (`1d08b23`)

- `status_amp_pipeline.rs` Cases A/B/C → single `#[rstest]` over `(Option<StatusEffectKind>, &str, DamageTag, i32)`. Case D unchanged.
- `anim_graph_parse.rs` 3 negative parse cases → single `#[rstest]` over `&str` literals. Positive parses stay separate.

**−61 LOC** net.

### Wave 6c — `anim_graph_asset.rs` consolidation (`75700f7`)

Folded 2 positive `agumon`/`renamon` asset parses by path + 1 truncated-asset reject case into a 2-matrix structure across both anim parse files. **−5 LOC**.

### Wave 6 — items SKIPPED (with reasons, deferred / dropped)

PLAN target list reframed mid-flight after pairwise read:

| File | PLAN claim | Reality | Decision |
|---|---|---|---|
| `holy_support_mechanics.rs` (4 of 6) | rstest over `(transition_op, expected_state)` | The 4 tests are heterogeneous: kernel dispatch, saturation, multi-step lifecycle (6-step), snapshot field assertions. Parametrizing would require chained pre-cases per case → degraded readability for ~84 LOC of cluttered scaffolding | **Skip** (degraded readability) |
| `compiled_timeline_runtime_dispatch.rs` | rstest over `(timeline_id, beats)` | 2 tests are semantically different: one asserts ordering chain across beats, other asserts negative rejection | **Skip** (no real shared shape) |
| `toughness_categories.rs` | rstest over `(ToughnessCategory, hits, expected_break)` | Test 2 has different intermediate asserts; test 4 is 6-step lifecycle | **Skip / marginal** (~30 LOC gain at readability cost) |
| `status_paralyzed_skip.rs:54-133` (REWRITE to proptest) | 100-iter no-variance loop → proptest over `(turn, prior_state)` | Debt-only (0 LOC win), genuine quality improvement | **Defer to Wave 8 debt batch** |

**Wave 6 actuals**: 3 commits, **−369 LOC**, vs PLAN target of **−360 + 79 rewrite**. Quota met for delete; rewrite quota deferred to Wave 8.

**Reframed conclusion**: PLAN over-listed Wave 6 targets — some files were tagged for symmetry, not because parametrization actually paid off after reading the diff in detail. The shipped sub-set is the genuinely high-ROI subset.

---

## Wave 7 — V3 common-helper migration sweep (LANDED in 4 sub-commits)

Originally planned as a single bulk commit; split during execution into 4 sub-waves by setup-pattern cluster to keep diffs reviewable.

### Wave 7a — TestAppBuilder migration (`add9c2f`)

Migrated 9 files to existing W1 helpers (`minimal_intent_app`, `TestAppBuilder` chain, `passive_dispatch_app`, `kernel_app(seed)`). 4 shape buckets: A (minimal), A+ledger+rng, B (passive), C (kernel). **−83 LOC**.

### Wave 7b — skill_book_runtime + form_identity_runtime helpers (`77ed2f4`)

Added 2 new helpers to `tests/common/app.rs` covering the resolve_action + follow_up_listener + resolve_follow_up_action chain with compiled `TimelineLibrary`. Migrated 4 files (`follow_up_chains.rs`, `follow_up_triggers.rs`, `pipeline_dispatch.rs`, `form_identity.rs`). **−114 LOC**.

### Wave 7c — skill_resolve_app helper (`c559cfe`)

Added `skill_resolve_app(book, seed)` helper for single-system `resolve_action_system` tests with custom unit spawns. Migrated 4 files (`damage_breakdown_log.rs`, `status_accuracy.rs`, `status_multi_kind_coexist.rs`, `status_refresh_max_dur.rs`). `status_slowed_delay.rs` skipped (outlier). **−48 LOC**.

### Wave 7d — turn_av_base_app helper (`6c7b210`)

Added `turn_av_base_app()` covering `App + MinimalPlugins + CombatState + TurnOrder + (TurnAdvanced, ActionValueUpdated, ActionIntent, CombatEvent)` skeleton. Migrated 3 files (`enemy_ai.rs`, `tempo_resistance.rs`, `turn_system_av.rs`); also folded `tempo_resistance.rs:368` inline duplicate. **−5 LOC**.

**Wave 7 actuals**: 4 commits, **−250 LOC**, vs PLAN target of **−250 LOC**. Quota hit exactly. 20 files migrated total across 5 reusable helpers in `tests/common/app.rs`.

---

## Wave 8 — proptest rewrite of status_paralyzed_skip (LANDED `4a01aef`)

V4 fragility rewrite was annulled — see Wave 8 reconciliation below.

### Wave 8 — Apply

Rewrote `status_paralyzed_skip.rs:54-133` from 100-iter no-variance loop (1 fixture) → `proptest!` invariant over `(turn_seed, prior_state, duration)`. Now 256 cases per run, default config. **+5 LOC net** (52 del / 57 ins), but quality climb is real: zero-variance loop → randomized 256-case invariant.

### Wave 8 — V4 fragility rewrite ANNULLATO

Originally Wave 8 also targeted typed-substring rewrites in `predator_loop_kernel.rs:254-268`, `battery_loop_kernel.rs:138`, `dorumon_predator_runtime.rs:164`, `holy_support_mechanics.rs:321`. Pre-execution verify revealed:

`battery_loop_kernel::last_transition` is **NOT a latch internal** — it is an intentional observability seam exposed via:
- `BatteryLoopSnapshot::last_transition` (`src/combat/blueprints/tentomon/identity.rs:260`)
- `format_battery_loop_snapshot()` (`src/combat/blueprints/tentomon/apply.rs:185-202`)
- `ValidationField::new("last", ...)` UI (`src/combat/blueprints/tentomon/mod.rs:100-113`)

Same evidence chain for `predator_loop_kernel`. Rewriting the substring asserts would remove the guard against regressions of the typed-display contract. **Decision**: keep the asserts; document the contract via DECISIONS append (deferred backlog).

**Wave 8 actuals**: 1 commit, +5 LOC net, 1 proptest invariant gained. Originally-planned ~95 LOC rewrite quota deferred as DECISIONS doc work.

---

## Acceptance check (Phase 4 — verify)

After all 14 sub-waves (0a-1, 0a-2, 0b-1, 0b-2, 0b-3, 1, 2, 3, 4, 5, 6a, 6b, 6c, 7a, 7b, 7c, 7d, 8):

1. `cargo test --tests` full nextest run — all green.
2. `cargo check --features windowed` — green.
3. `cargo clippy --tests` — no new warnings.
4. `find src -type d -name tests` → empty (R003 W0a compliance).
5. `rg -l '#\[cfg\(test\)\]\s*mod tests' src/` → only files with ≤100 LOC blocks (R003 W0c compliance).
6. `rg -l 'fn setup_app\|fn make_unit' tests/` → only `tests/common/`.
7. `rg '#\[ignore\]' tests/` → only `combat_cli_shared_surface` subprocess test.
8. `wc -l tests/*.rs src/**/*.rs` → tests/ LOC reduced ~−2,260; src/ reduced ~−4,757 (relocate + delete combined).
9. Update `INVENTORY.md` with realized numbers; write `SUMMARY.md` covering: actual delete/move/rewrite totals, deviations from plan and why, follow-ups (deferred backlog).

---

## Risk register

| Risk | Mitigation |
|---|---|
| W0a private-API leak when relocating dir tests | Compile error reveals every `super::*` → upgrade to `pub(crate)`. Fail-fast pattern; no behavior change. |
| W0b file-by-file delete-vs-keep mistakes | Step 1 of each W0b wave: read inline + read suspected integration + diff in writing in commit body before deleting. |
| Wave 1 `TestAppBuilder` doesn't cover a variant | Wave 7 step 1 catches it; that file stays unmigrated this slice with a TODO in commit body. |
| Wave 7 large commit is hard to review | Mechanical pattern; reviewer can sample-check + rely on green tests. Per-category sub-headers in commit body. |
| Wave 8 typed field access doesn't expose the substring info | Step 1 verifies the alternative path exists per file BEFORE editing. If not exposed: skip that file, note follow-up. |
| Hidden coupling between deleted file and snapshot test | `cargo test --tests` between every wave catches it. |
| Test name collision after merges | Each merged test gets clear `caseN_*` suffix from `#[rstest]`. |

## Out of scope (deferred, tracked)

- **W0c** — remaining ≤100 LOC inline `mod tests` (438 LOC) — separate slice.
- **Impl-coupling debt** — 93% test files import `bevyrogue::combat::*` — separate slice with public test API.
- **`last_transition` contract docs** — add to DECISIONS + project-memory pattern entry.
- **`source_file_loc_limit`** re-home as CI/lint check — follow-up issue.
- **Backward direction R003** (tests/ → src/ in cases where it makes sense) — separate slice.
