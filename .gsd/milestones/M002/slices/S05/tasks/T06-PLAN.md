---
estimated_steps: 3
estimated_files: 5
skills_used: []
---

# T06: Twin Core synergy badge + slice verification matrix

Why: Twin Core is the milestone's placeholder ally signal; the demo needs visible proof that the Ultimate resolves into the Twin Core synergy without actually spawning a second unit (M003+ territory). This task also runs the slice verification matrix and records environment limitations explicitly per MEM048/MEM053.

Do: (1) Add a windowed-only `TwinCoreBadgeState { primed_for_frames: u32 }` resource in `src/ui/combat_panel/mod.rs` and an observer system that listens for the Twin Core `BlueprintSignal` projection (already emitted by `src/combat/blueprints/twin_core/mod.rs` on `ult_used`) via `OnKernelTransition::Blueprint(owner = "twin_core", ...)` or the equivalent generic signal seam exposed today — pick the existing path the reactive flash uses in S04 and mirror it. (2) On signal observation, set `primed_for_frames` to a small fixed constant (e.g. 60). A tick system decrements per frame; rendering shows a chip in the combat panel via `src/ui/combat_panel/render.rs` with text/tooltip helpers added to `src/ui/combat_panel/labels.rs`. No `CombatState` mutation. (3) Add a feature-gated test (extend `tests/windowed_preview_cache.rs` or add `tests/windowed_twin_core_badge.rs`) proving the badge: appears on signal, shows the expected label, decrements deterministically, clears at zero, never appears without the signal, and triggers exactly once per Ultimate. (4) Run the full slice verification matrix and capture evidence: `cargo test --test timeline_two_clock_parity --test timeline_cue_barrier_pipeline --test timeline_loop_hop_cue_parity --test agumon_baby_burner_reactive --test agumon_baby_burner_primary --test agumon_sharp_claws_asset --test anim_player_fsm --test anim_graph_asset --test anim_gameplay_command_forbidden --test clip_atlas_parity --test bouncing_fire_off_baseline --test encounter_bootstrap_windowed`, `cargo test --features windowed --test windowed_preview_cache --test windowed_hud_hp_bar --test windowed_target_hurt --test windowed_twin_core_badge`, `cargo test --lib`, `cargo build --no-default-features`, `cargo build --features windowed`. The optional `BEVYROGUE_VALIDATION_WINDOWED=1 BEVYROGUE_VALIDATION_WINDOWED_SOAK_SECS=1 cargo run --features windowed --bin bevyrogue` is environment-limited inside `gsd_exec` (MEM053) — document the limitation explicitly rather than asserting success.

Done-when: the new badge test passes; the verification matrix is green; environment limitation is recorded in the task summary.

## Inputs

- `src/ui/combat_panel/mod.rs`
- `src/ui/combat_panel/render.rs`
- `src/ui/combat_panel/labels.rs`
- `src/windowed/mod.rs`
- `src/combat/blueprints/twin_core/mod.rs`
- `src/combat/blueprints/agumon/mod.rs`
- `src/combat/runtime/registry.rs`
- `src/combat/runtime/post_action.rs`
- `tests/windowed_preview_cache.rs`

## Expected Output

- `src/ui/combat_panel/mod.rs`
- `src/ui/combat_panel/render.rs`
- `src/ui/combat_panel/labels.rs`
- `src/windowed/mod.rs`
- `tests/windowed_twin_core_badge.rs`

## Verification

cargo test --features windowed --test windowed_twin_core_badge --test windowed_preview_cache --test windowed_hud_hp_bar --test windowed_target_hurt

## Observability Impact

Adds `TwinCoreBadgeState` (windowed-only) — a future agent can read it to confirm Twin Core synergy fired without re-running the Ultimate. Slice verification matrix produces durable evidence (test outputs + build pass) and environment limitation is recorded explicitly so a future agent does not mistake the missing live soak for a regression.
