# S02 — Research: TargetShape resolver (Blast + AoE) with slot_index tie-break

**Date:** 2026-05-13
**Lane:** research (scout report for the planner)

## Summary

S02 expands the `TargetShape` resolver from `Single` to `Blast` (primary + slot_index ±1) and `AoE(All)` (every alive enemy), with `slot_index` ascending as the canonical deterministic tie-break. The two non-`Single` shapes that already exist in the enum (`Row`, `AllEnemies`, `SelfOnly`) are validation-gated as `UnimplementedTargetShape` in three sites (`skills_ron::validate_skill_def`, `action_query::target_status_for_*`, `resolution::target_shape_is_executable_now`); making any non-`Single` shape **executable** is therefore a known surgical change at three call sites plus the pipeline applicator, which today applies effects to one `target_entity`. There is **zero prior use of `slot_index`** in `src/` or `tests/` (Q3 from milestone CONTEXT) — it must be introduced. The roster also already uses `shape: Row` / `shape: AllEnemies` / `shape: SelfOnly` in `assets/data/skills.ron` (11 non-`Single` occurrences across ~72 total) but every such skill must currently have `implementation: Deferred{UnimplementedTargetShape}` to pass `validate_skill_book`, so flipping a couple of those to `Implemented` is the cheapest path to a real scenario fixture.

The minimum-surface design that satisfies all milestone constraints is: (1) add `SlotIndex(u8)` as a Bevy `Component` assigned at `spawn_unit_from_def` (per-team contiguous 0..N), (2) extend `TargetShape` with `Blast` (and align on whether `AoE(All)` is a renamed `AllEnemies` or a new variant — recommended: **reuse `AllEnemies` and treat `AoE(All)` as its RON-facing alias** to avoid churn), (3) add a pure `resolve_targets(shape, primary: UnitId, snapshot) -> Vec<UnitId>` helper colocated with the resolver, (4) replace the single `target_entity` lookup inside `pipeline::step_app` with a loop over `resolve_targets(...)`, calling `apply_effects` per target, (5) emit one `OnDamageDealt` per target so the existing JSONL logger surface needs no schema change. For KO'd-adjacent behaviour (Q2), recommend **absorb-on-KO** (spillover slot lost, deterministic, matches Honkai Blast convention) — emit `OnActionFailed{reason:"target ko"}` or simply skip the per-target apply when defender is KO'd, no special new event.

## Recommendation

**Path:** introduce `SlotIndex` Component first, then plumb a `resolve_targets()` helper, then promote `Blast` + `AllEnemies` (renamed in DSL surface to `AoE(All)` if the roadmap wording is binding) through the three validation gates and the pipeline applicator. Cap S02 strictly at `Blast` + `AoE(All)` — leave `Bounce(N)` and the four extended selectors to S03/S04 as already scoped.

**Open decisions to lock at planning T01:**
- **Q2 KO'd adjacent (Blast):** absorb (lose the spillover slot, no damage event). Rationale: deterministic; trivial JSONL fixture; matches HSR Blast.
- **Q3 slot_index representation:** **new `SlotIndex(u8)` Component**, assigned at `spawn_unit_from_def` based on per-team insertion order in `apply_composition`. Durable across the encounter, decoupled from `UnitId`. Cost: one component, ~5 lines in bootstrap, no schema migration.
- **`AoE(All)` vs `AllEnemies`:** keep enum variant `AllEnemies`; treat `AoE(All)` as a roadmap label only, **no new variant**. Mention in S02 SUMMARY so the roadmap wording is reconciled.

## Implementation Landscape

### Key Files

- `src/data/skills_ron.rs:9-14` — `TargetShape` enum (`Single` / `Row` / `AllEnemies` / `SelfOnly`). Add `Blast` variant. `AoE(All)` = alias of `AllEnemies` (no new variant).
- `src/data/skills_ron.rs:277-289` — `validate_skill_def` rejects `Implemented` skills whose shape != `Single`. Allowlist must extend to `Blast` and `AllEnemies`. `Row` and `SelfOnly` stay deferred.
- `src/data/skills_ron.rs:291-330` — secondary `Effect::Damage{target, ..}` consistency check; ensure new shapes pass when `Damage.target == targeting.shape`.
- `src/combat/resolution.rs:185-194` — `target_shape_is_executable_now` + `target_shape_rejection_reason`. Extend the `matches!` allowlist to `Blast` and `AllEnemies`.
- `src/combat/resolution.rs:56-95` — `resolve_action` produces `ResolvedAction.target` (single `UnitId`) and `.target_shape`. **No change** — target stays the *primary* target; the multi-target fan-out happens at apply time, not at resolve time. This keeps existing snapshot/log/event tests untouched.
- `src/combat/resolution.rs` (new) — add `pub fn resolve_targets(shape: TargetShape, primary: UnitId, snapshot: &TargetableSnapshot) -> Vec<UnitId>` ordered by slot_index asc. Pure, table-tested.
- `src/combat/action_query.rs:485-489` — `target_status_for_*` deferral gate. Allow `Blast` and `AllEnemies` past the gate.
- `src/combat/turn_system/pipeline.rs:160-292` — `step_app`. Today: finds one `target_entity` (line 168), calls `apply_effects` once (line 278). Change: build a target list from `resolve_targets(action.target_shape, action.target, …)`, then loop. **Watch:** the `attacker_ult`, `sp_tracker`, `basic_streak`, ult-charge mutations must happen **once** (per cast), not per hit; only the per-defender state (defender_unit, defender_tough, defender_bag) is iterated. Easiest split: hoist resource consumption out of the loop, or pass a flag to `apply_effects` (`is_primary_hit: bool`) and gate consumption on that.
- `src/combat/bootstrap.rs:141-197` — `spawn_unit_from_def`. Insert new `SlotIndex(u8)` component. **Caller responsibility:** `apply_composition` (lines 199-211) must hand a running per-team counter into spawn — simplest is to inline two `for (idx, def) in …enumerate()` loops and pass `SlotIndex(idx as u8)` into the spawn.
- `src/combat/unit.rs` or new `src/combat/slot.rs` — declare `#[derive(Component, …)] pub struct SlotIndex(pub u8);`. Per-team scoped (Team is already a component, so `(SlotIndex, Team)` together gives global uniqueness).
- `src/combat/jsonl_logger.rs` — already emits one line per `CombatEvent`; emitting one `OnDamageDealt` per resolved target gives stable JSONL list-shape automatically. **No schema change needed.** Verify the line-ordering test fixture covers slot_index asc.
- `src/bin/combat_cli.rs` — extend the existing `--scenario` dispatcher (S01 added `advance-delay-cap`) with `--scenario aoe-blast` that loads a 3-enemy encounter, casts one `Blast` and one `AllEnemies` skill, prints the resolved target list and per-target damage step-by-step.
- `assets/data/skills.ron` — flip one existing `shape: Row` skill (or add a new fixture) to `shape: AllEnemies` with `implementation: Implemented` for the scenario. Add one fixture `shape: Blast` skill. Keep all 11 existing non-Single deferred skills untouched to avoid scope creep.
- `tests/` — new files (functional naming per CLAUDE.md): `target_shape_blast_spillover.rs`, `target_shape_aoe_all_order.rs`, `slot_index_tiebreak.rs`.

### Build Order

1. **Introduce `SlotIndex` Component + bootstrap wiring first.** Highest-risk piece because it has to be durable across the encounter and visible to the snapshot query the resolver reads from. Prove it with a unit-test-style integration test that spawns a 3-ally / 3-enemy encounter and asserts each `Team`-scoped slot range is `0..3`. This unblocks every downstream tie-break decision.
2. **Pure `resolve_targets` helper.** Table-driven test (no Bevy world): given a synthetic snapshot of `(UnitId, Team, SlotIndex, alive)` tuples and a `TargetShape`, assert the returned `Vec<UnitId>` is deterministic and slot_index-ordered. KO-absorbing behaviour is exercised here.
3. **Validation gates (three sites).** Mechanical change; runs all 554 existing tests with zero impact (gates only widen). Confirm with `cargo test`.
4. **Pipeline fan-out in `step_app`.** Loop over `resolve_targets`, gate resource consumption to the primary hit. This is the highest-risk integration step — it touches the same code S01 just refactored. Re-run the full suite after.
5. **CLI scenario `aoe-blast` + JSONL stability test.** Run `combat_cli --scenario aoe-blast` 10× under hash-diff to prove determinism.
6. **One Blast + one AoE fixture skill in `skills.ron`, marked Implemented.** Last so the loader change is independent of the engine change.

### Verification

- `cargo check` (headless + `--features windowed`): clean.
- `cargo test`: 554+ tests green (S01 baseline) plus the three new test binaries.
- `rg -n 'TargetShape::(Blast|AllEnemies)' src/` shows all three gates (validate_skill_def, target_shape_is_executable_now, action_query) consistently allow the new shapes.
- `cargo run --bin combat_cli -- --scenario aoe-blast`: exit 0; JSONL diff stable across 10 invocations (`diff <(run) <(run)` empty).
- M017 regression: `tests/status_slowed_delay.rs`, `tests/tempo_resistance.rs`, `tests/turn_advance_split.rs` all still green (S02 does not touch advance/delay paths).

## Risks & Watch-outs

- **Resource-consumption double-fire** is the dominant risk: `apply_effects` today consumes SP, ult charge, basic-streak, and energy. Iterating it per-target would double-charge. Mitigation: factor resource consumption out of `apply_effects` into the caller, or thread an `is_primary_hit: bool` flag and short-circuit consumption when false. Planner should pick one before T03.
- **`UnimplementedTargetShape` legality cache.** `action_query` may be cached by an upstream snapshot system; widening the gate must not leave stale entries. Likely safe (snapshot is rebuilt each frame) but verify by reading `src/combat/action_query.rs` snapshot construction.
- **Roadmap label drift.** Roadmap says `AoE(All)`; enum has `AllEnemies`. If we keep `AllEnemies` and document the alias in S02 SUMMARY, no enum churn. If we rename, every existing 11 deferred skills in `skills.ron` plus the enum become a wide diff — recommend the alias.
- **Slot stability with revives.** `Revive` is supported (`Effect::Revive(pct)`). `SlotIndex` is assigned at spawn and never mutated, so revived units retain their slot — confirms slot_index is a stable tie-break across the encounter.
- **`Damage.target` vs `targeting.shape` consistency check** at `skills_ron.rs:291-330` already enforces equality; the Blast/AoE fixture skills must set both consistently.

## Do not hand-roll

- Iteration order over a Bevy `Query` is not deterministic without an explicit sort key. The resolver MUST sort by `(Team, SlotIndex)` before returning — do not assume query order is stable.

## Sources

- `src/data/skills_ron.rs` (TargetShape, validation), `src/combat/resolution.rs` (resolver + apply_effects), `src/combat/turn_system/pipeline.rs:160-340` (step_app), `src/combat/bootstrap.rs:141-211` (spawn), `src/combat/action_query.rs:485` (legality gate).
- `assets/data/skills.ron` (11 non-Single shapes already declared, all deferred).
- S01 SUMMARY (just-completed advance/delay split, same pipeline file).
- Milestone CONTEXT.md (Q2 / Q3 carry-forwards; SC#2 cap removal already applied in S01).
