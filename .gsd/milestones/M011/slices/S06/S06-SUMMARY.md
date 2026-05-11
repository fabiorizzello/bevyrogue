---
id: S06
parent: M011
milestone: M011
provides:
  - ["Action Value turn system (MAX_AV=10000, speed-proportional AV advancement, UnitId tiebreak)", "TempoResistance component with 100→50→25% Delay attenuation curve", "MIN_ACTION_THRESHOLD_AV=15000 floor preventing infinite tempo-lock", "tempo_resistant: bool UnitDef flag wiring boss units in units.ron", "14-test integration suite covering full resistance pipeline"]
requires:
  []
affects:
  - ["S07", "S08", "S09"]
key_files:
  - ["src/combat/av.rs", "src/combat/resistance.rs", "src/combat/turn_system/mod.rs", "src/combat/turn_order.rs", "src/combat/mod.rs", "src/data/units_ron.rs", "assets/data/units.ron", "src/combat/bootstrap.rs", "src/headless.rs", "tests/tempo_resistance.rs", "tests/turn_system_av.rs"]
key_decisions:
  - ["MAX_AV=10000 (HSR-inspired): granular enough for fractional speed differences, maps cleanly to percentage-based AV manipulation", "MIN_ACTION_THRESHOLD_AV=15000: boss needs at most 25000 AV gain after max delay, preserving recoverability without enabling infinite lock", "Resistance applies only to negative amount_pct (Delay); positive Advances bypass resistance entirely — Advance remains a reliable tool", "apply_turn_advance_system reads CombatEvent bus rather than inlining into pipeline.rs — follows single-source-of-truth event bus principle", "f64 intermediate math for multiplier computation avoids integer truncation at 25% (−2000 × 0.25 = −500, not 0)", "#[serde(default)] on tempo_resistant: bool in UnitDef — no migration needed for existing RON entries"]
patterns_established:
  - ["CombatEvent::TurnAdvance as the canonical AV manipulation surface — skills, traits, and items emit this event; resistance logic listens on the bus", "Conditional ECS component insertion in bootstrap.rs based on UnitDef flags — pattern for future optional mechanics (e.g., BreakSeal in S07)", "Pure function pairs (compute_av_change + apply_av_change) alongside Bevy systems — pure functions get unit-tested without a Bevy world, systems get integration-tested with one"]
observability_surfaces:
  - ["ActionValue component on each unit is directly queryable in ECS world — primary artifact for debugging turn order desync", "TurnAdvanced event includes av_at_turn and av_change fields — JSONL log shows attenuation effect per delay", "TempoResistance.hit_count queryable per unit — shows how many delay stacks a boss has accumulated"]
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-04-28T08:46:11.853Z
blocker_discovered: false
---

# S06: Tempo Resistance (100→50→25%) + Min Action Threshold

**Replaced VecDeque turn system with Action Value model and implemented TempoResistance (100→50→25% Delay attenuation) + MIN_ACTION_THRESHOLD_AV floor, verified by 14-test integration suite with zero regressions across the full test suite.**

## What Happened

S06 delivered two foundational changes to the combat engine: a full turn system refactor and the Tempo Resistance mechanic for bosses.

**T01 — Action Value turn system (src/combat/av.rs, turn_order.rs, turn_system/mod.rs)**

The VecDeque-based turn system was entirely replaced with an Action Value model. Each unit now carries an `ActionValue` component (i32 counter). Every tick, units advance their AV proportional to speed (`AV_PER_SPEED=100`); the unit whose AV first reaches `MAX_AV=10000` acts next, with `UnitId` as a tiebreak for identical values. KO'd and Stunned units are skipped entirely. The old `seed()` and `insert_out_of_queue()` entry points are retained as no-op shims so pre-AV tests compiled without change. `ValidationSnapshot` was updated to derive `turn_preview` from an ECS `ActionValue` query rather than the defunct `TurnOrder.future_preview`. Four new integration tests in `tests/turn_system_av.rs` cover basic advancement, stunned-skip, KO-skip, and UnitId tiebreaking.

**T02 — TempoResistance + MIN_ACTION_THRESHOLD_AV (src/combat/resistance.rs)**

A new `TempoResistance` component was added, tracking `hit_count` and computing a stepwise multiplier: 1.0 on the first Delay hit, 0.5 on the second, 0.25 on the third and beyond. `MIN_ACTION_THRESHOLD_AV=15_000` clamps the AV floor — a fully-delayed boss needs at most 25000 AV gain to act again, preventing infinite tempo-lock. Positive Advance amounts bypass resistance entirely. The `apply_turn_advance_system` reads `CombatEvent::TurnAdvance` from the message bus and applies resistance + clamping; it runs before `advance_turn_system` in the `.chain()`. All math uses f64 intermediates to avoid truncation at 25% (e.g., -2000 × 0.25 = -500.0). Ten tests in `tests/tempo_resistance.rs` cover the full surface: pure-logic unit tests for the curve, clamping, and advance bypass; plus Bevy integration tests confirming the system mutates `ActionValue` and `TempoResistance` correctly across two consecutive delay events.

**T03 — Data wiring + end-to-end boss scenario**

`UnitDef` in `src/data/units_ron.rs` gained a `tempo_resistant: bool` field with `#[serde(default)]`, so existing RON entries parse as `false` without migration. Devimon in `assets/data/units.ron` is marked `tempo_resistant: true`. `bootstrap.rs` inserts `TempoResistance::default()` conditionally on boss units. Four additional tests were added: boss spawn gets the component, ally spawn does not, the canonical `units.ron` parse confirms Devimon's flag, and `boss_scenario_three_slow_hits_show_resistance_curve` drives three consecutive -20% Delay events through the full pipeline (spawn → CombatEvent bus → apply_turn_advance_system) and asserts the AV changes follow the 100%→50%→25% curve. All 14 tempo_resistance tests pass; full suite (116+ tests) remains green.

## Verification

- `cargo test --test tempo_resistance -- --nocapture`: 14/14 pass (exit 0)
- `cargo test --test turn_system_av -- --nocapture`: 4/4 pass (exit 0)
- `cargo test --test validation_snapshot -- --nocapture`: 3/3 pass (exit 0)
- `cargo test`: full suite green, 0 failures across all test binaries (116+ tests)

## Requirements Advanced

None.

## Requirements Validated

- R078 — 14-test suite passes: multiplier curve, MIN_ACTION_THRESHOLD_AV clamping, boss spawn wiring, ally exclusion, and boss_scenario_three_slow_hits_show_resistance_curve end-to-end test all green

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

none — all tasks executed according to plan; T03 resumed into a fully-implemented state and required only verification runs

## Known Limitations

TempoResistance hit_count is not reset between encounters — if the boss survives to a second encounter (future meta-loop), it will start with accumulated resistance stacks. Encounter reset behavior is deferred to meta-loop scope (post-M012).

## Follow-ups

none discovered during execution — S07 (Toughness 3 categories + Break Seal) can proceed on the stable AV turn system established here

## Files Created/Modified

None.
