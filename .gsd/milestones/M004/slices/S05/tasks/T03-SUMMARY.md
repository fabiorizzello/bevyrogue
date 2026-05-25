---
id: T03
parent: S05
milestone: M004
key_files:
  - tests/animation/vfx_asset_load.rs
  - tests/animation/vfx_asset_eval.rs
  - tests/animation/render_no_vfx_kind_guard.rs
  - src/windowed/render.rs
key_decisions:
  - Asserted Sharp Claws RGB channels are all >1.0 (overbright) as the headless proxy for HDR-bloom intent, since the curve evaluator is pure and the bloom camera itself is windowed-only.
  - Added a positive data-boundary token assertion (on_enter_effect_ids must exist) to the no-VFX-kind guard rather than only forbidding identifiers, so a removed bridge is localized as a data-path regression, not just an absence.
  - Used exact near-miss negative names to pin that the spawn bridge is an exact map, not a substring/string-kind match.
duration: 
verification_result: passed
completed_at: 2026-05-25T18:25:31.009Z
blocker_discovered: false
---

# T03: Added headless contract + curve-eval + render-bridge tests proving Sharp Claws is data-driven and no-VFX-kind regression-guarded

**Added headless contract + curve-eval + render-bridge tests proving Sharp Claws is data-driven and no-VFX-kind regression-guarded**

## What Happened

Extended the headless test surface so Sharp Claws is proven data-driven without launching a window. (1) tests/animation/vfx_asset_load.rs gains agumon_vfx_contains_sharp_claws_slash: the real assets/digimon/agumon/vfx.ron must contain sharp_claws.slash, which reuses the registered agumon/baby_flame/static placement verb (asserted to be in KNOWN_VERBS), has a bounded single-particle spawn plan (count 1, spread 0, ttl 6, asserted 0<ttl<=12), TargetCenter anchor, size 34.0, the sharp_claws_slash texture key, and no on_expire (terminal). Added PlacementAnchor to the imports. (2) tests/animation/vfx_asset_eval.rs gains sharp_claws_slash_curves_evaluate_deterministically_and_overbright: loads the real asset and drives the pure evaluator — scale pops 0.6->1.0 by 0.3 life then holds; color holds the overbright pale yellow-white core (all RGB >1.0, asserted, so the HDR+bloom camera blooms it) while alpha fades 0.95->0.0; midpoint alpha is the linear interpolant (0.475); and 1000 repeated evals are bit-identical (R004). (3) src/windowed/render.rs unit tests gain on_enter_sharp_claws_maps_only_to_the_slash_effect: on_enter_effect_ids(sharp_claws_slash) maps exactly to AGUMON_SHARP_CLAWS_EFFECT_ID == sharp_claws.slash, and near-miss names (sharp_claws, slash, sharp_claws_strike, baby_flame_charge, empty) do NOT resolve to it — proving an exact name map, not a substring/string-kind match (Q7 negatives). (4) tests/animation/render_no_vfx_kind_guard.rs keeps the forbidden-identifier guard green and adds render_rs_keeps_the_data_driven_effect_id_boundary: asserts on_enter_effect_ids still exists and that Sharp Claws routes through the owned effect id, so a future agent can localize a data-path regression directly.

## Verification

Ran the three prescribed headless commands plus the windowed binary test harness (no game window launched, K001 respected). All green. vfx_asset_load: 16 passed (incl. new Sharp Claws contract test). vfx_asset_eval: 12 passed (incl. new deterministic overbright eval test). render_no_vfx_kind_guard: 2 passed (forbidden-identifier guard + new positive data-boundary assertion). Binary on_enter tests: 2 passed (incl. new exact-name-map negative test). Malformed-asset behavior (Q5) stays covered by the existing validate_effects rejection tests, which remain green.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test animation vfx_asset_load` | 0 | pass | 560ms |
| 2 | `cargo test --test animation vfx_asset_eval` | 0 | pass | 120ms |
| 3 | `cargo test --test animation render_no_vfx_kind_guard` | 0 | pass | 120ms |
| 4 | `cargo test --features windowed --bin bevyrogue on_enter` | 0 | pass | 1480ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `tests/animation/vfx_asset_load.rs`
- `tests/animation/vfx_asset_eval.rs`
- `tests/animation/render_no_vfx_kind_guard.rs`
- `src/windowed/render.rs`
