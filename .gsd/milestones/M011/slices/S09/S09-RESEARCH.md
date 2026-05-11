# M011/S09 — Research

**Date:** 2026-04-28
**Slice:** Numerical rebalance pass + UAT scenarios

## Summary

S09 closes M011 with three things: (1) a numerical rebalance of `units.ron` / `skills.ron` so TTK lands on the R083 targets (boss 4–7, mini-boss 3–5, minion 2–3), (2) three deterministic scenario fixtures in `tests/` that lock those TTK targets in (so the rebalance is verifiable rather than vibes-based), and (3) a 30-minute UAT script driven through the existing `combat_cli` harness, plus the final pass on `docs/combat_design.md` sez. 9 to mark Form Identity as wired.

The research surfaced three real blockers that must be planned around before any rebalance happens:

1. **No mini-boss or minion fixtures exist.** `assets/data/units.ron` ships exactly one enemy (`Devimon` UnitId 101, HP 500, Armored, tempo-resistant). S09 must define at least one mini-boss and one minion fixture (probably as new entries in `units.ron`) before TTK targets for those tiers are even meaningful.
2. **`combat_cli` does not spawn enemies.** S04's `bootstrap_system` accepts a `SelectionRequest { rookie_ids }` and `bootstrap_encounter` always returns `EncounterComposition { allies, enemies: Vec::new() }`. The S04 summary explicitly flagged this as deferred to S09 ("No enemies in the bootstrap encounter — enemy encounter wiring is deferred to S09"). Without enemies the UAT is impossible.
3. **`BonusToughnessDamage` and `BonusDamageVsAttribute` are parsed but not wired.** S08 added the variants to `Effect` enum and round-trip-tested them, but `apply_effects` does not consume them. DORUgamon and Angemon Form Identity currently use a "fire a separate skill" workaround; S08's follow-up explicitly handed the modifier-in-place evaluation to S09. The slice can either (a) wire both effects properly and switch the two Adults to modifier-in-place form, or (b) declare the workaround acceptable and remove the dead variants. The S09 rebalance is the right place to make that call.

The recommended path: build scenario fixtures *first* (against current numbers, expecting them to fail), then rebalance until they pass, then run UAT. This is classic test-first rebalance — the fixtures define what "balanced" means before any number changes.

## Recommendation

Sequence the slice as four tasks:

1. **T01 — Enemy roster extension + encounter definitions.** Add a minion (low HP, low toughness, Standard, no follow-up) and a mini-boss (mid-tier HP/toughness, Armored or Standard, no Form Identity, possibly tempo_resistant=false) to `units.ron`. Optionally introduce a thin `assets/data/encounters.ron` (or hardcoded `EncounterPreset` enum in `src/combat/bootstrap.rs`) listing the three canonical compositions: `MinionWave` (3 minions), `MiniBossEncounter` (1 mini-boss + 2 minions), `BossEncounter` (Devimon + 0 or 1 supporting minion). Extend `bootstrap_encounter` to accept an `EncounterPreset` and populate `enemies` accordingly. Wire the CLI to prompt the user for an encounter selection after party selection.

2. **T02 — Three TTK scenario fixtures (`tests/scenarios/`).** Create one integration test per tier. Each test loads real `units.ron` + `skills.ron` (so it tests the shipped numbers), spawns the canonical encounter, and runs a deterministic scripted action sequence representative of "competent play" — e.g., basic attacks until SP available, then highest-impact skill, ult when ready. Assert: encounter ends in Victory, turn count falls within R083 band, no panics. The "competent play" script is intentionally a fixture, not an AI — it locks the rebalance to a specific representative play pattern, which is what makes the test reproducible.

3. **T03 — Numerical rebalance pass.** Iterate on `units.ron` / `skills.ron` HP, toughness, base damage, SP costs, ult triggers, status durations until all three scenario tests pass. Decide here whether to wire `BonusToughnessDamage` (DORUgamon Form Identity ramp) and `BonusDamageVsAttribute` (Angemon vs Virus) into `apply_effects`, or remove the unused variants. Update `docs/combat_design.md` sez. 9 to reflect final wiring (mark "wired" vs the design intent).

4. **T04 — UAT execution + sign-off.** Build the S09-UAT.md script that walks a tester through three encounters via `cargo run --bin combat_cli` against minion / mini-boss / boss compositions, with expected observations (turn counts, Form Identity firings, break sealing, energy ramps). Capture verdict in `S09-ASSESSMENT.md`.

## Implementation Landscape

### Key Files

- `assets/data/units.ron` — currently 6 ally Children + 6 ally Adults + 1 boss (Devimon). **Add minion(s) and mini-boss UnitDef entries** (UnitId range 102+). Devimon HP=500, Armored, tempo_resistant=true is the boss anchor.
- `assets/data/skills.ron` — 65 skills, all 6 Form Identity entries present. **Tune base damage / SP / ult numbers** here during T03 rebalance. Enemy skills (`enemy_skill_fire`, `enemy_ult_fire`) are minimal — may need new tag-flexible enemy skill set for mini-boss / minion variety.
- `src/combat/bootstrap.rs:59-103` — `bootstrap_encounter` always returns `enemies: Vec::new()`. **Extend to accept an encounter preset** that drives enemy population. `taichi_def` (l. 167) is the only built-in def; mirror that pattern or load enemies from RON.
- `src/bin/combat_cli.rs:60-112` — `bootstrap_system` consumes `SelectedAllies` resource. **Add an `SelectedEncounter` resource and a pre-`App::run()` `inquire::Select` prompt** for encounter type. The current dashboard already iterates `Query<(&Unit, &Team, ...)>` and prints both teams, so it will display enemies once they exist.
- `src/data/skills_ron.rs:33-35` — `BonusToughnessDamage(i32)` and `BonusDamageVsAttribute { attribute, bonus_pct }` parsed but not consumed in `src/combat/resolution.rs::apply_effects`. T03 decision point: wire them or strip them.
- `src/combat/resolution.rs::apply_effects` — the call site where the unwired bonus variants must be added if T03 chooses to wire them. Pattern: extend `ResolvedAction` struct, extract from `Effect`, apply at the right point in `apply_hit` (toughness damage path for `BonusToughnessDamage`, base damage path for `BonusDamageVsAttribute`).
- `src/data/units_ron.rs::UnitDef` — schema is stable; reuse for new minion / mini-boss entries. `tempo_resistant` and `toughness_category` fields with `#[serde(default)]` mean new entries can omit them.
- `src/combat/events.rs::CombatEventKind` — `OnDamageDealt`, `OnBreak`, `OnKO` are the events scenario tests will count to verify TTK turn-by-turn. `OnActionResolved` marks turn-end.
- `src/combat/turn_system/mod.rs::advance_turn_system` + `check_victory_system` — the existing AV pump that drives the encounter loop. Scenario tests register both, plus `resolve_action_system`, `follow_up_listener_system`, `form_identity_listener_system`, `resolve_follow_up_action_system`, and the ult/jsonl systems (mirror the wiring in `src/bin/combat_cli.rs:414-432`).
- `tests/encounter_e2e.rs` — closest existing precedent for end-to-end victory assertion. Uses inline `SkillBook`, `App::new()` + `Update` chain, manual `ActionIntent` writes, drives to `CombatPhase::Victory`. Scenario tests should reuse this pattern but load real RON via `DataPlugin`.
- `tests/toughness_categories.rs` — recent (S07) reference for `MessageCursor<CombatEvent>` event counting between actions. Use the same pattern in scenario tests.
- `docs/combat_design.md` — sez. 9 ("Form Identity") currently holds design intent; T03 finalizes by adding a "wired in S08" annotation and updating any number references that change.

### Build Order

**Build T01 first** — without minion / mini-boss UnitDefs and without `bootstrap_encounter` populating enemies, neither the scenario tests (T02) nor the UAT (T04) can run. T01 is purely additive: it does not change combat logic, only data and a single bootstrap function signature.

**T02 second** — write the three scenario tests *expecting them to fail* against current numbers. The failing tests define the rebalance target precisely. This is the "test-first rebalance" discipline.

**T03 third** — iterate on numbers until T02 turns green. The decision on `BonusToughnessDamage` / `BonusDamageVsAttribute` wiring belongs here because it's a balance-feel call; keep both options on the table (wire or strip) and let UAT in T04 decide if dead variants are acceptable.

**T04 last** — UAT only makes sense after the rebalance lands. Sign-off depends on subjective feel (pacing, readability of events, Form Identity satisfaction) which automated tests cannot capture.

### Verification Approach

- **Per-task, T01:** `cargo check && cargo test --test roster_catalog` (existing test that validates `units.ron` parse + invariants — confirms new minion / mini-boss entries don't break the catalog) plus a new `bootstrap_spawn_composition` test variant for each preset.
- **Per-task, T02:** `cargo test --test scenario_minion_ttk` / `scenario_miniboss_ttk` / `scenario_boss_ttk` — each asserts (a) terminal `CombatPhase::Victory`, (b) turn counter within R083 band, (c) at least one `OnBreak` (boss/mini-boss only), (d) at least one Form Identity `EnergyGained` event (boss only — enough rounds to trigger).
- **Per-task, T03:** full `cargo test` green; `git diff assets/data/` shows tuned numbers; if Bonus* variants are wired, new tests for those paths.
- **Per-task, T04:** `cargo run --bin combat_cli` walks all three encounters; `S09-UAT.md` checklist signed; `docs/combat_design.md` sez. 9 updated.
- **Slice-level:** all 21+ integration binaries green; R083 marked validated; M011 ready for milestone closure.

## Don't Hand-Roll

| Problem | Existing Solution | Why Use It |
|---------|------------------|------------|
| Counting events of a specific kind between actions | `MessageCursor<CombatEvent>` (MEM055; pattern from `tests/event_stream.rs` and `tests/toughness_categories.rs`) | Dual-surface verification: event count + component state. Self-diagnosing failures. |
| Loading real RON in tests | `DataPlugin` (registered in `src/data/mod.rs`) handles `units.ron` + `skills.ron` async loading; `tests/roster_smoke.rs` is a precedent | Tests the shipped data, not a parallel inline fixture that drifts |
| Encounter composition definition | `EncounterComposition { allies, enemies }` (`src/combat/bootstrap.rs:24`) is already the abstraction | Extend rather than introduce a parallel concept |
| Deterministic RNG in scenario tests | `CombatRng::default()` uses fixed seed `[42u8;32]` (MEM034) | No need to re-seed per test; explicit seed only when overriding default behavior |

## Constraints

- **Headless-first (D015, R019):** scenario tests must run under `App::new()` without `MinimalPlugins`. CLI works under `MinimalPlugins + ScheduleRunnerPlugin`; tests do not need scheduler.
- **No per-Digimon code (D020):** all rebalance happens in RON. Engine code touched only to wire `BonusToughnessDamage` / `BonusDamageVsAttribute` if T03 decides to.
- **Determinism (R019):** action sequences in scenario tests must be hardcoded, not RNG-driven. Status accuracy uses `CombatRng` — fixed seed gives reproducible miss/hit (MEM034).
- **Append-only DECISIONS:** any rebalance choice that supersedes prior numerical assumptions is a candidate decision (e.g., "removed unwired Bonus* variants" → DXX). Don't rewrite D043–D046; supersede explicitly if needed.
- **MEM053:** `follow_up.rs` has a private `ResolveActorsQuery` alias structurally identical to `turn_system/mod.rs`. Wiring `BonusToughnessDamage` / `BonusDamageVsAttribute` may need to extend `ResolvedAction`, which does not change the query — so this constraint should not bite, but verify if extending the query becomes necessary.
- **MEM029:** in multi-update scenario tests, drain the `CombatEvent` cursor between updates or events from earlier updates are pruned before the final assertion.

## Common Pitfalls

- **"TTK depends on player skill" trap.** A scenario test asserting "boss dies in 4–7 turns" only works if the action sequence is fixed. If the test allows any winning sequence, the assertion becomes "is theoretically reachable in 4–7 turns" which is hard to bound. Pin the action sequence; rebalance the numbers.
- **Loading RON in tests requires `DataPlugin` + waiting for `DataReady`.** The `bootstrap_system` in `combat_cli.rs` runs every tick until `DataReady` is present and `units.is_empty()`. Tests must drive `app.update()` enough times for the asset loader to flip `DataReady`. Alternative: load RON synchronously via `std::fs::read_to_string` + `ron::from_str` in the test setup (mirror S04 T05 pattern in `combat_cli.rs:336-344`).
- **Stale OnDamageDealt events re-trigger Form Identity (MEM029-class, S08 finding).** When asserting "Form Identity fired exactly N times" in a scenario test, drain the event queue between rounds or use a fresh `App` per round (S08 T04 used this for Angemon negative test).
- **Adding a new enemy with `tempo_resistant: true` triggers `TempoResistance` component spawn**; if the rebalance scenario depends on Slow/Delay landing for TTK, verify the test's RNG seed produces a hit, not a miss-by-curve.
- **Devimon resists Fire & Ice.** The current boss has `resists: [Fire, Ice]`. Scenario test for boss TTK must use a non-resisted tag (Light = weakness, Dark / Electric / Physical = neutral) or the rebalance is fighting a 0.75× modifier.
- **`bootstrap_encounter` injects Taichi (UnitId 0) as a Commander.** Scenario tests must account for the 5th ally; `check_victory_system` likely already excludes Commander from "all KO'd" check, but verify before asserting Victory conditions.

## Open Risks

- **No automated player AI.** Scenario tests use a hardcoded action script, which means the test reflects "this specific sequence beats this encounter in N turns" not "any reasonable play beats it in 4–7." If the encounter tests pass but UAT feel says TTK is wrong, the gap is the test fixture itself, not the numbers.
- **Bonus* effect wiring decision is non-trivial.** Wiring `BonusToughnessDamage` and `BonusDamageVsAttribute` properly requires extending `ResolvedAction` and `apply_effects`. This is engine work in a slice scoped for "rebalance + UAT." T03 should declare upfront which option (wire or strip) is taken; if wire is chosen, the cost may push T03 over budget.
- **UAT subjectivity.** R083 is half automated (3 fixtures) and half subjective (30-minute manual play). The product owner sign-off is the gate, and there is no formal rubric. Consider drafting a checklist in T04 before play (TTK turn count, Form Identity readability, Break Seal correctness, Tempo Resistance feel, status effect impact) so sign-off is structured.
- **`tests/scenarios/` directory pattern.** Cargo treats every `.rs` file directly under `tests/` as a separate integration binary. Three flat files (`tests/scenario_minion_ttk.rs` etc.) is the simplest fit; a `tests/scenarios/` subdirectory needs a `tests/scenarios/main.rs` driving sub-modules, which is a heavier pattern. Plan should pick one upfront; flat files are recommended.
- **CLI encounter prompt UX.** Adding an `inquire::Select` for encounter type before `App::run()` is straightforward, but the current `is_terminal` branch in `combat_cli.rs:349` defaults non-interactive runs to a fixed party. The encounter prompt needs the same fallback (default to `BossEncounter` non-interactively) to keep CI smoke-tests green.

## Sources

- `docs/combat_design.md` v5.3 — sez. 9 (Form Identity intent), sez. 11 (Archetype Weaknesses), sez. 12 (Roster MVP) — read in research to understand intended Adult roles for rebalance feel.
- `.gsd/milestones/M011/slices/S04/S04-SUMMARY.md` — flagged "No enemies in bootstrap encounter — deferred to S09" as a Known Limitation.
- `.gsd/milestones/M011/slices/S08/S08-SUMMARY.md` — Follow-ups: "S09 rebalance should evaluate whether BonusToughnessDamage and BonusDamageVsAttribute deliver better numerical feel than the current separate-skill approach."
- `.gsd/DECISIONS.md` D043, D044, D045, D046 — formula, tag rename, Form Identity model, re-entrancy bounding (all relevant to interpreting rebalance levers).
- `.gsd/REQUIREMENTS.md` R083 — TTK target spec.
