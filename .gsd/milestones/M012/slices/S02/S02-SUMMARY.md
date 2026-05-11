---
id: S02
parent: M012
milestone: M012
provides:
  - Enemy-only toughness affordance truth
  - Hidden ally/zero-max toughness surfaces
  - Stable unsupported-shape rejection reasons
  - Preserved target-shape metadata for later legality query work
requires:
  []
affects:
  - S03
  - S04
  - S06
  - S07
key_files:
  - src/combat/toughness.rs
  - src/combat/resolution.rs
  - src/combat/turn_system/pipeline.rs
  - src/combat/state.rs
  - src/headless.rs
  - src/combat/observability.rs
  - src/ui/combat_panel.rs
  - src/windowed.rs
  - tests/toughness_enemy_only.rs
  - tests/bootstrap_spawn_composition.rs
  - tests/validation_snapshot.rs
  - tests/target_shape_truthfulness.rs
  - tests/follow_up_triggers.rs
  - tests/combat_coherence.rs
key_decisions:
  - Keep Toughness attached internally on allies, but gate visible/applicable break semantics with team-aware helpers.
  - Carry TargetShape on ResolvedAction so legality can be decided after DSL resolution but before mutation.
  - Reject non-single shapes in step_declaration with a stable UnimplementedTargetShape:<Shape> failure and no lifecycle side effects.
patterns_established:
  - Use shared helper functions for both runtime behavior and display-surface truthfulness.
  - Prefer explicit pre-mutation rejection over silent degradation when a shape is not yet executable.
  - Treat feature-gated windowed compile checks as a required regression gate whenever combat query signatures change.
observability_surfaces:
  - Headless roster output
  - Validation snapshot formatting
  - Combat event failure reasons (`OnActionFailed` with `UnimplementedTargetShape`)
  - Windowed combat panel compile path
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-04-30T21:32:43.650Z
blocker_discovered: false
---

# S02: Enemy-only Toughness and TargetShape truthfulness

**S02 made combat truthfulness consistent across engine, headless/validation surfaces, and windowed compile: ally toughness no longer looks like an enemy break bar, zero-max bars stay hidden, and unsupported target shapes are rejected before mutation.**

## What Happened

S02 closed two UI-blocking truth gaps in the combat layer. First, toughness exposure is now team-aware: allies may still carry internal toughness/weakness data for damage math, but the runtime and display surfaces only expose break affordances for enemies with a real positive bar. HP damage, status, revive, and weakness-driven damage continue to resolve for ally targets without consuming toughness, emitting OnBreak, or applying stun-from-break side effects.

Second, target-shape truthfulness is now preserved through resolution. `ResolvedAction` carries the originating `TargetShape`, and `step_declaration` rejects non-single shapes (`Row`, `AllEnemies`, `SelfOnly`) before any lifecycle mutation with a stable `UnimplementedTargetShape:<Shape>` failure reason. That keeps the engine honest about current capabilities instead of silently treating AoE-like data as a single-target action. Canonical follow-up fixtures that previously relied on row-shaped stand-ins were updated so regression coverage reflects the new contract.

The slice also made the player-facing and diagnostic surfaces truthful: headless roster output, validation snapshots, and the windowed combat panel now all hide ally toughness and zero-max enemy bars while still showing positive enemy break bars when they exist. A minor windowed compatibility path had to be refreshed after the optional toughness query shape changed, but the feature-gated build now compiles cleanly. Overall, the slice established reusable helper-based semantics that S03/S04/S06 can consume directly rather than hardcoding per-skill UI behavior.

## Verification

Fresh verification in this workspace passed:
- `cargo test-dev --test toughness_enemy_only --test follow_up_triggers --test combat_coherence --test bootstrap_spawn_composition --test validation_snapshot --test target_shape_truthfulness` ✅
  - `toughness_enemy_only`: 2/2
  - `follow_up_triggers`: 6/6
  - `combat_coherence`: 3/3
  - `bootstrap_spawn_composition`: 1/1
  - `validation_snapshot`: 3/3
  - `target_shape_truthfulness`: 3/3
- `cargo check --features "dev windowed"` ✅

Observed contract behavior during verification:
- ally toughness is hidden/N/A in headless and validation surfaces
- zero-max enemy toughness is hidden
- positive enemy toughness remains visible
- `Row` and `AllEnemies` are rejected before mutation with `UnimplementedTargetShape`
- `Single` still executes normally

## Requirements Advanced

- R085 — Established truthful enemy-only toughness exposure and explicit target-shape deferral/rejection so UI-readiness work can rely on engine truth instead of hardcoded assumptions.
- R084 — Preserved resolved target-shape metadata and shared the engine-side legality vocabulary that later pure query work will consume.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

None.

## Follow-ups

None.

## Files Created/Modified

None.
