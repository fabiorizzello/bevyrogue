# M004 Rendering Acceptance & Rescope (S05)

This artifact is the authoritative record of what S05 delivered for M004's
windowed render-path scope, what its **automated** evidence proves, what
decision **D037** defers, and what **S06** (manual visual signoff) still owns.
A future validator should read this before counting any S05 evidence toward
milestone validation — so that automated proof is not mistaken for human UAT
or for visual-quality acceptance.

## Scope at a glance

| Area | Status | Evidence owner |
|------|--------|----------------|
| Windowed HDR + bloom camera policy | **Delivered** | `src/windowed/render.rs`, `tests/windowed_only/vfx_windowed_contracts.rs` |
| Authored linear VFX color data | **Delivered** | `assets/digimon/agumon/vfx.ron`, `tests/animation/vfx_asset_eval.rs` |
| Sharp Claws VFX (data-driven, RON + AnimGraph + bridge) | **Delivered** | `assets/digimon/agumon/vfx.ron`, `assets/digimon/agumon/anim_graph.ron`, `src/windowed/render.rs`, `tests/animation/vfx_asset_load.rs` |
| No-hardcoded-VFX-kind regression guard | **Delivered** | `tests/animation/render_no_vfx_kind_guard.rs` |
| Strict custom additive particle material | **Deferred (D037)** | See "What D037 defers" |
| Human visual signoff (`cargo winx`) | **Not delivered — S06 / manual-only** | K001, S06 |

## What was delivered (and what proves it)

### 1. HDR + bloom camera policy (windowed)
The windowed camera now spawns with an explicit HDR/bloom post-processing
policy (`Camera2d`, `Hdr`, `Bloom::NATURAL`, `Tonemapping::TonyMcMapface`,
`DebandDither::Enabled`) and VFX particle colors are written via
`Color::linear_rgba`, so authored channels above `1.0` survive into HDR
rendering instead of being clamped as sRGB UI values.

- Bevy 0.18 does not expose the older `Camera { hdr: true }` field; the
  equivalent local contract is the `Hdr` component plus Bloom/Tonemapping/
  DebandDither (see T01 deviation).
- Proof: `tests/windowed_only/vfx_windowed_contracts.rs` fails fast if the
  camera loses HDR/bloom/tonemapping/dithering, or if the render path
  stops writing VFX colors with `Color::linear_rgba`. This is a headless test
  harness; it does **not** launch the game window (K001 respected).

### 2. Authored linear VFX color data
Agumon's effect color curves in `assets/digimon/agumon/vfx.ron` remain owned
authored data. Automated proof only asserts that the pure evaluator preserves
those values deterministically; whether the resulting effect looks good is still
a human-only judgment.

- Proof: `tests/animation/vfx_asset_eval.rs` drives the pure curve evaluator
  on the real asset and asserts authored scale/color values plus deterministic,
  bit-identical repeated evaluation (R004).

### 3. Sharp Claws VFX — fully data-driven
`sharp_claws.slash` was authored entirely through the owned data seam:
- a single target-anchored particle in `vfx.ron` (ttl 6, size 34px, scale pop
  0.6→1.0 by 0.3 life then hold, overbright pale yellow-white `(3.0,3.0,2.2)`
  with alpha fade 0.95→0.0), reusing the already-registered
  `agumon/baby_flame/static` placement verb — **no** new placement verb and
  **no** `register_agumon_ext`/core change;
- an `on_enter` `SpawnParticle` trigger on the `sharp_claws_strike` node in
  `anim_graph.ron`, via the existing node-entry bridge;
- string→effect/texture map arms in `src/windowed/render.rs`
  (`AGUMON_SHARP_CLAWS_EFFECT_ID`, `on_enter_effect_ids`, `vfx_texture_handle`,
  `VfxVisuals.sharp_claws_slash`) — **no** new `VfxParticleKind`-style branching.
- Claw orientation is baked into `assets/vfx/sharp_claws_slash.png` because
  windowed particle rendering has no per-particle rotation.
- Proof: `tests/animation/vfx_asset_load.rs` asserts the real RON contains a
  bounded single-particle `sharp_claws.slash` on a known placement verb, and
  the binary unit test maps `sharp_claws_slash` to exactly the slash effect id
  (near-miss names do not resolve — an exact map, not a substring match).

### 4. No-hardcoding regression guard
`tests/animation/render_no_vfx_kind_guard.rs` forbids reintroduction of
hardcoded VFX-kind identifiers **and** positively asserts the
`on_enter_effect_ids` data boundary still exists, so a future agent can
localize a removed bridge as a data-path regression rather than a silent
absence.

This supports the local M004 constraint labels: **R002** (headless-first — all
the above contract tests run without `windowed`), **R004** (determinism — the
curve evaluator is seed-free and bit-stable), and **R005** (dependency gating —
HDR/bloom/render code is `windowed`-only).

## What D037 defers

**D037** (`.gsd/DECISIONS.md`) deferred **strict custom additive particle
material** to a future isolated rendering refactor, unless milestone validation
later explicitly requires it. Bevy 0.18's built-in 2D sprite/`ColorMaterial`
path does not expose true additive blending; honest additive delivery would
require a custom `Material2d` plus particle mesh/material conversion, which is
higher risk than the S05 acceptance gap.

S05 therefore delivers technical windowed render-path prerequisites — HDR/Bloom
camera wiring, linear color writes, and authored VFX data exercised through the
existing seam — but does **not** implement strict additive blending and does
**not** claim automated visual acceptance. No test in this slice asserts
additive-material delivery or human-perceived quality, and this artifact makes
no such claim.

## What S06 still owns (do not claim here)

- **K001 manual visual signoff** for Baby Flame, Baby Burner, and Sharp Claws
  in the real windowed build (`cargo winx`). Auto-mode cannot run the windowed
  binary (K001), so visual quality is **manual-only** and remains pending S06.
- This artifact deliberately makes **no** claim of `cargo winx` human signoff
  and **no** claim of strict additive material delivery.

## Verification commands

The full S05 automated evidence set (run after all S05 code/doc changes):

```bash
test -s .gsd/milestones/M004/slices/S05/M004-RENDERING-ACCEPTANCE.md
cargo test --test animation vfx_asset_load -- --nocapture
cargo test --test animation vfx_asset_eval -- --nocapture
cargo test --test animation render_no_vfx_kind_guard -- --nocapture
cargo check --features windowed
cargo test --features windowed --test windowed_only vfx_asset_impact_render -- --nocapture
cargo test --features windowed --test windowed_only vfx_windowed_contracts -- --nocapture
```

These are compile and headless test-harness commands only; none launches the
game window (K001). Fresh results are recorded in the T04 task summary's
Verification Evidence table.
