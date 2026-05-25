---
id: S05
parent: M004
milestone: M004
provides:
  - sharp_claws.slash effect in vfx.ron
  - windowed HDR bloom camera policy
  - overbright VFX colors for bloom
  - Sharp Claws AnimGraph on_enter trigger
  - render bridge for sharp_claws_slash
  - no-VFX-kind regression guard with positive boundary assertion
  - M004-RENDERING-ACCEPTANCE.md D037 rescope artifact
requires:
  - slice: S01
    provides: typed VfxAsset/eval/load contracts
  - slice: S02
    provides: placement registry and windowed render-dispatch contracts
  - slice: S03
    provides: AnimGraph cue/effect bridge patterns
  - slice: S04
    provides: validation/boundary documentation
affects:
  []
key_files:
  - assets/digimon/agumon/vfx.ron
  - assets/digimon/agumon/anim_graph.ron
  - src/windowed/render.rs
  - assets/vfx/sharp_claws_slash.png
  - tests/animation/vfx_asset_load.rs
  - tests/animation/vfx_asset_eval.rs
  - tests/animation/render_no_vfx_kind_guard.rs
  - tests/windowed_only/vfx_rendering_acceptance.rs
  - tests/windowed_only/vfx_asset_impact_render.rs
  - .gsd/milestones/M004/slices/S05/M004-RENDERING-ACCEPTANCE.md
key_decisions:
  - Sharp Claws VFX reuses the registered `agumon/baby_flame/static` placement verb — RON-only addition, no verb registration, no core change, no VfxParticleKind branching (D037 compliance).
  - D037 rescope: strict custom additive particle material deferred; Bevy 0.18 2D ColorMaterial has no true additive blending. HDR bloom + overbright colors (all RGB >1.0) deliver the visual intent.
  - Overbright RGB channels (all >1.0) used as headless-testable proxy for HDR-bloom intent, since the bloom camera itself is windowed-only.
  - Positive data-boundary token assertion added to no-VFX-kind guard so a removed bridge fails fast as a data-path regression, not just an absence.
  - K001: no `cargo winx` human signoff performed; S06 owns that boundary. Automated proof and manual UAT are kept strictly separate.
patterns_established:
  - RON-only VFX extension: reuse a registered placement verb and write only vfx.ron — no code change needed for effects that fit existing verbs.
  - Overbright-channel assertion as headless HDR proxy: asserting all RGB > 1.0 on the loaded asset is a pure, windowed-free contract for bloom capability.
  - Positive + negative name-map assertions on render bridge arms to prove exact dispatch, not substring/kind matching.
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-25T18:29:12.623Z
blocker_discovered: false
---

# S05: Sharp Claws and rendering acceptance remediation

**Delivered data-driven Sharp Claws VFX through the owned RON/AnimGraph/render bridge, enabled windowed HDR bloom, and documented D037 rescope; all 7 automated checks green.**

## What Happened

S05 closed the remaining automated rendering acceptance gaps for M004 across four tasks.

**T01 — HDR bloom rendering policy:** Enabled the windowed camera for HDR rendering using Bevy 0.18's `Hdr` component (equivalent to the older `hdr: true` field) alongside `Bloom`, `Tonemapping`, and `DebandDither`. Updated `vfx.ron` VFX color writes to `Color::linear_rgba` so overbright channel values drive HDR bloom rather than being treated as sRGB. Added `vfx_rendering_acceptance` windowed contract tests asserting the camera HDR/bloom policy and overbright VFX data. Windowed compile and acceptance tests passed.

**T02 — Sharp Claws VFX authored and triggered:** Added the `sharp_claws.slash` effect to `assets/digimon/agumon/vfx.ron` — a single target-anchored particle using the already-registered `agumon/baby_flame/static` placement verb (no new verb registration, no core change). Effect: ttl 6 ticks, size 34px, scale pop 0.6→1.0 then hold, overbright pale yellow-white color (3.0, 3.0, 2.2) with alpha 0.95→0.0 to bloom under the HDR camera. Wired the AnimGraph trigger via an `on_enter SpawnParticle(name: "sharp_claws_slash")` on the `sharp_claws_strike` node in `anim_graph.ron`. Extended `src/windowed/render.rs` with the `AGUMON_SHARP_CLAWS_EFFECT_ID` constant, an `on_enter_effect_ids` arm, and a `vfx_texture_handle` arm — all through existing string maps with no new VfxParticleKind-style branching. Generated a baked-orientation 64×64 RGBA claw-streak texture. Repaired four stale test assertions that T01's overbright color change had left red.

**T03 — Hardened contracts:** Added three new headless contract layers: (1) `agumon_vfx_contains_sharp_claws_slash` in `vfx_asset_load` — asserts the real asset contains `sharp_claws.slash`, reuses `agumon/baby_flame/static`, and has the expected spawn params; (2) `sharp_claws_slash_curves_evaluate_deterministically_and_overbright` in `vfx_asset_eval` — drives the pure curve evaluator and asserts scale pop, all-channels-overbright color (headless HDR proxy), alpha fade, and 1000-run bit-identical determinism (R004); (3) `on_enter_sharp_claws_maps_only_to_the_slash_effect` in `render.rs` unit tests — proves the spawn bridge is an exact name map with negative assertions on near-miss names. Extended the no-VFX-kind guard with a positive data-boundary token assertion so a removed bridge fails fast.

**T04 — Acceptance artifact and full evidence:** Authored `.gsd/milestones/M004/slices/S05/M004-RENDERING-ACCEPTANCE.md` separating automated proof from deferred work: delivered items (HDR bloom policy, overbright colors, Sharp Claws fully data-driven), the D037 deferral rationale (Bevy 0.18 2D ColorMaterial has no true additive blending), and the K001 boundary (no `cargo winx` human signoff; S06 owns that). Ran all 7 prescribed commands fresh — all passed.

**Fresh slice verification (all checks green):** Artifact present, `vfx_asset_load` 16 passed, `vfx_asset_eval` 12 passed, `render_no_vfx_kind_guard` 2 passed, `cargo check --features windowed` clean, `vfx_asset_impact_render` 7 passed, `vfx_rendering_acceptance` 2 passed.

## Verification

Fresh slice-level verification via gsd_exec (run ID 9f6e6278):

1. `test -s .gsd/milestones/M004/slices/S05/M004-RENDERING-ACCEPTANCE.md` → exit 0, PASS
2. `cargo test --test animation vfx_asset_load` → exit 0, 16 passed
3. `cargo test --test animation vfx_asset_eval` → exit 0, 12 passed
4. `cargo test --test animation render_no_vfx_kind_guard` → exit 0, 2 passed
5. `cargo check --features windowed` → exit 0, Finished dev profile
6. `cargo test --features windowed --test windowed_only vfx_asset_impact_render` → exit 0, 7 passed
7. `cargo test --features windowed --test windowed_only vfx_rendering_acceptance` → exit 0, 2 passed

All 7 checks green. No game window launched (K001 respected). Visual human signoff deferred to S06.

## Requirements Advanced

None.

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

Strict custom additive particle blending is deferred (D037). Human visual signoff on Sharp Claws and HDR bloom appearance belongs to S06. No `cargo winx` window was launched in this slice (K001).

## Follow-ups

None.

## Files Created/Modified

None.
