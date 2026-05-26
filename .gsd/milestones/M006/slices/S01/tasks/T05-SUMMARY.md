---
id: T05
parent: S01
milestone: M006
key_files:
  - tests/windowed_only/enoki_impact_render.rs
key_decisions:
  - Asserted the quad system's deletion via code-shaped tokens (fn advance_vfx_particles, VfxParticle { literal, the for-loop) rather than bare 'advance_vfx_particles'/'VfxParticle' substrings, because those names survive intentionally in render.rs historical comments — a bare !contains would false-fail.
  - Renamed enoki_handle_map_is_keyed_by_all_three_contact_burst_ids to ..._all_six_agumon_ids and added the three Baby Flame sequence ids (charge, ember, projectile) to the assertion set, matching M006/S01's enoki-as-sole-renderer end state.
duration: 
verification_result: passed
completed_at: 2026-05-26T10:57:47.496Z
blocker_discovered: false
---

# T05: Inverted the windowed source-contract tests to the enoki-only state: pinned the quad system as deleted, broadened the handle-map assertion to all six Agumon ids, and added a T03 lifecycle-layer contract.

**Inverted the windowed source-contract tests to the enoki-only state: pinned the quad system as deleted, broadened the handle-map assertion to all six Agumon ids, and added a T03 lifecycle-layer contract.**

## What Happened

The pre-T04 contract in tests/windowed_only/enoki_impact_render.rs pinned the quad spawn loop's continued existence (`for i in 0..count`). T04 deleted that loop, so this task inverts the contract to the enoki-only end state.

Changes to enoki_impact_render.rs:
1. Added a new test `quad_vfx_system_is_fully_deleted_from_render_src` asserting the quad system's defining code symbols are gone from render.rs: `fn advance_vfx_particles`, the `VfxParticle {` component bundle literal, and the `for i in 0..count` spawn loop. I verified via grep that `advance_vfx_particles`/`VfxParticle` survive only inside historical comments (lines 401, 452, 987, 1032, 1332, 1605), so the assertions match code-shaped tokens (`fn ...`, struct literal) rather than bare substrings that the comments would falsely trip. The existing `!block.contains("for i in 0..count")` + `return 0` assertions in the one-shot routing test were kept.
2. Renamed `enoki_handle_map_is_keyed_by_all_three_contact_burst_ids` to `enoki_handle_map_is_keyed_by_all_six_agumon_ids` and broadened it to assert all six ids are inserted in load_agumon_enoki_vfx: CHARGE, EMBER, PROJECTILE, IMPACT, DETONATE, SHARP_CLAWS (matching the six const names at render.rs:444-457 and the six handles.insert calls at 506-544).
3. Added `enoki_lifecycle_layer_is_present` pinning the T03 layer: `ChargeEmberEnokiMarker`, `ProjectileFlight`, and `fn advance_enoki_projectiles`.
4. Updated the module-header doc comment to describe the six-id map and the lifecycle layer.

The `kernel_and_fsm_control_flow_remains_untouched` test (fire_kernel_cue / request_release, D031/D032) was already present and was left unchanged. vfx_asset_impact_render.rs and render_no_vfx_kind_guard.rs were not touched — they exercise lib symbols / on_enter_effect_ids that survive, and they pass unchanged.

## Verification

Ran the full verification set. `cargo test --features windowed --test windowed_only`: 54 passed / 0 failed (includes the new quad_vfx_system_is_fully_deleted_from_render_src, the renamed six-id map test, and enoki_lifecycle_layer_is_present — the harness compiled the renamed test, proving the rename is consistent). `cargo test --test dependency_gating`: 2 passed / 0 failed — bevy_enoki_absent_from_headless_graph still proves no enoki/windowed leak into the headless graph. `cargo build` (headless): Finished clean. Full headless `cargo test`: all harnesses green (51 passed in the lib-symbol harness shown; overall exit 0 under pipefail).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --features windowed --test windowed_only` | 0 | pass | 1064ms |
| 2 | `cargo test --test dependency_gating` | 0 | pass | 5327ms |
| 3 | `cargo build (headless)` | 0 | pass | 5327ms |
| 4 | `cargo test (full headless)` | 0 | pass | 5327ms |

## Deviations

Plan said the existing `for i in 0..count` assertion should be removed/replaced. It was already present (added by T04) as a negative `!block.contains(...)` plus a `return 0` assertion, which already expresses the quad-loop-gone contract. I kept those and added a dedicated quad_vfx_system_is_fully_deleted_from_render_src test that broadens coverage to the function and component-literal symbols, rather than re-deleting the satisfactory existing assertions.

## Known Issues

none

## Files Created/Modified

- `tests/windowed_only/enoki_impact_render.rs`
