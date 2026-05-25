# S05: Sharp Claws and rendering acceptance remediation

**Goal:** Resolve the remaining automated rendering acceptance gaps for M004 by delivering Sharp Claws through the owned VfxAsset/AnimGraph/windowed bridge, enabling HDR bloom-capable VFX rendering, and documenting the D037 rescope that strict custom additive particle material is deferred unless later validation explicitly requires it.
**Demo:** After this: Sharp Claws VFX is either authored and tested in assets/digimon/agumon/vfx.ron or explicitly rescoped, HDR bloom additive rendering criteria are implemented or rescoped, and automated evidence covers the chosen outcome.

## Must-Haves

- `assets/digimon/agumon/vfx.ron` contains a validated `sharp_claws.slash` effect using registered placement verbs and a bloom-capable appearance curve.
- `assets/digimon/agumon/anim_graph.ron` triggers Sharp Claws VFX through `Command::SpawnParticle` rather than a hardcoded render path.
- `src/windowed/render.rs` maps the Sharp Claws particle name to the Agumon-owned effect id and spawns it through the existing VfxAsset data path.
- The windowed camera setup is HDR/bloom configured with Bevy 0.18 components, and automated guards prove the bloom policy is present.
- Automated evidence covers headless asset/schema contracts, the no-hardcoded-VFX guard, windowed compile, and windowed rendering contract tests.
- Strict additive material delivery is not claimed; the D037 deferral is restated in an S05 acceptance artifact and S06 remains responsible for human visual signoff.

## Proof Level

- This slice proves: Contract plus windowed compile proof. S05 must not claim visual UAT: automated tests and `cargo check --features windowed` prove wiring, schema, and compile viability; S06 owns human `cargo winx` signoff or waiver.

## Integration Closure

Consumes S01 typed VfxAsset/eval/load contracts, S02 placement registry and windowed render-dispatch contracts, S03 AnimGraph cue/effect bridge patterns, and S04 validation/boundary documentation. Introduces real Sharp Claws wiring into the existing windowed entry path and HDR/bloom camera configuration. Remaining milestone usability gap after S05 is only S06 human visual signoff/waiver, not automated rendering acceptance.

## Verification

- Keep VFX failure visibility on the existing windowed asset-load/effect-resolution warning seams. Add static/contract tests so future agents can localize missing HDR/bloom configuration, missing Sharp Claws effect ids, unregistered placement verbs, or accidental return to hardcoded VFX-kind paths without launching a window.

## Tasks

- [x] **T01: Enable HDR bloom rendering policy** `est:1h`
  Expected executor skills: bevy, rust-development, rust-testing, tdd, verify-before-complete.
  - Files: `src/windowed/render.rs`, `assets/digimon/agumon/vfx.ron`, `tests/windowed_only/vfx_rendering_acceptance.rs`
  - Verify: cargo check --features windowed
cargo test --features windowed --test windowed_only vfx_rendering_acceptance -- --nocapture

- [x] **T02: Author and trigger Sharp Claws VFX** `est:1.5h`
  Expected executor skills: bevy, rust-development, rust-testing, tdd, verify-before-complete.
  - Files: `assets/digimon/agumon/vfx.ron`, `assets/digimon/agumon/anim_graph.ron`, `src/windowed/render.rs`, `assets/vfx/sharp_claws_slash.png`
  - Verify: cargo check --features windowed
cargo test --test animation vfx_asset_load -- --nocapture

- [x] **T03: Harden Sharp Claws and no-hardcoding contracts** `est:1h`
  Expected executor skills: rust-development, rust-testing, tdd, verify-before-complete.
  - Files: `tests/animation/vfx_asset_load.rs`, `tests/animation/vfx_asset_eval.rs`, `tests/animation/render_no_vfx_kind_guard.rs`, `src/windowed/render.rs`
  - Verify: cargo test --test animation vfx_asset_load -- --nocapture
cargo test --test animation vfx_asset_eval -- --nocapture
cargo test --test animation render_no_vfx_kind_guard -- --nocapture

- [x] **T04: Record rendering acceptance outcome and run final evidence** `est:45m`
  Expected executor skills: write-docs, rust-development, rust-testing, verify-before-complete.
  - Files: `.gsd/milestones/M004/slices/S05/M004-RENDERING-ACCEPTANCE.md`
  - Verify: test -s .gsd/milestones/M004/slices/S05/M004-RENDERING-ACCEPTANCE.md
cargo test --test animation vfx_asset_load -- --nocapture
cargo test --test animation vfx_asset_eval -- --nocapture
cargo test --test animation render_no_vfx_kind_guard -- --nocapture
cargo check --features windowed
cargo test --features windowed --test windowed_only vfx_asset_impact_render -- --nocapture
cargo test --features windowed --test windowed_only vfx_rendering_acceptance -- --nocapture

## Files Likely Touched

- src/windowed/render.rs
- assets/digimon/agumon/vfx.ron
- tests/windowed_only/vfx_rendering_acceptance.rs
- assets/digimon/agumon/anim_graph.ron
- assets/vfx/sharp_claws_slash.png
- tests/animation/vfx_asset_load.rs
- tests/animation/vfx_asset_eval.rs
- tests/animation/render_no_vfx_kind_guard.rs
- .gsd/milestones/M004/slices/S05/M004-RENDERING-ACCEPTANCE.md
