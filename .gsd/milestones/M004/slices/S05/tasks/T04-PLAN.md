---
estimated_steps: 5
estimated_files: 1
skills_used: []
---

# T04: Record rendering acceptance outcome and run final evidence

Expected executor skills: write-docs, rust-development, rust-testing, verify-before-complete.

Why: S05 includes a scope decision: HDR/bloom is delivered, strict custom additive material is deferred by D037, and S06 remains responsible for human visual signoff. That outcome must be explicit so milestone validation does not overcount automated evidence as manual UAT or true additive material delivery.

Do: Write a concise S05 rendering acceptance artifact under the S05 slice directory that states what was delivered, what automated evidence proves, what D037 defers, and what S06 still owns. The artifact should be reader-oriented for a future validator: no claim of `cargo winx` human signoff, no claim of strict additive material unless the executor actually implemented and tested it, and direct references to the verification commands/tests. Then run the complete S05 automated evidence set after all code/doc changes.

Done when: The S05 acceptance artifact exists and the full verification suite passes with fresh output after the final change.

Quality gates: Q3 no auth/data exposure. Q4 validates local M004 R002/R004/R005 support and the D037 rescope. Q5 dependency failure mode is test/build failure; do not mark complete until failures are fixed or explicitly escalated. Q6 full verification may be slower under windowed features but should remain local-only. Q7 negative proof is covered by T03 guards and T01 bloom-policy assertions.

## Inputs

- `.gsd/milestones/M004/slices/S04/M004-VALIDATION-SCOPE.md`
- `.gsd/milestones/M004/slices/S04/M004-BOUNDARY-MAP.md`
- `assets/digimon/agumon/vfx.ron`
- `assets/digimon/agumon/anim_graph.ron`
- `src/windowed/render.rs`
- `tests/animation/vfx_asset_load.rs`
- `tests/animation/vfx_asset_eval.rs`
- `tests/animation/render_no_vfx_kind_guard.rs`
- `tests/windowed_only/vfx_asset_impact_render.rs`
- `tests/windowed_only/vfx_rendering_acceptance.rs`

## Expected Output

- `.gsd/milestones/M004/slices/S05/M004-RENDERING-ACCEPTANCE.md`

## Verification

test -s .gsd/milestones/M004/slices/S05/M004-RENDERING-ACCEPTANCE.md
cargo test --test animation vfx_asset_load -- --nocapture
cargo test --test animation vfx_asset_eval -- --nocapture
cargo test --test animation render_no_vfx_kind_guard -- --nocapture
cargo check --features windowed
cargo test --features windowed --test windowed_only vfx_asset_impact_render -- --nocapture
cargo test --features windowed --test windowed_only vfx_rendering_acceptance -- --nocapture

## Observability Impact

Creates a validator-facing acceptance/rescope artifact so later agents can distinguish delivered automated rendering proof from deferred additive material and S06 manual UAT.
