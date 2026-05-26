# S04 RESEARCH — bevy_enoki integration spike (one effect)

> Milestone M005 · Slice S04 · `risk:high` · `depends:[]` — the deepest-research, highest-risk slice. It adopts an unfamiliar GPU 2D particle dependency (`bevy_enoki`), windowed-gated, proving the integration on ONE Agumon effect before the full migration (S05).

## Summary

S04 is a **spike**: wire `bevy_enoki` as a windowed-gated GPU particle backend and prove ONE Agumon impact effect renders from a `.particle.ron` asset, while `cargo test` (headless) stays green and a static dep-gating test proves `bevy_enoki` (and its transitive render crates: `bevy_render`, `bevy_sprite_render`, `bevy_core_pipeline`, `bevy_camera`, `bevy_mesh`, `bevy_shader`) are ABSENT from the headless build. The central risk is **dependency-graph leakage**: `bevy_enoki 0.6` hard-depends on the entire Bevy render stack, so if it (or its feature) is reachable from the default build, R002/R005 are violated and the headless build balloons. The integration must hang the enoki spawn off the *existing* `spawn_effect_by_id` seam in `src/windowed/render.rs` WITHOUT moving particle lifetime into the kernel/FSM cue/barrier timeline (D031/D032 stay untouched — enoki only RENDERS). The good news: the project's gating architecture already isolates ALL windowed code (the entire `src/windowed/` module is `#[cfg(feature = "windowed")]`-gated at the `mod` declaration in `src/main.rs`), so the leak surface is small and well-understood.

## Recommendation

End-to-end approach, in risk-retiring order:

1. **Add the dep + prove no leak FIRST (before any effect authoring).** Declare `bevy_enoki = { version = "0.6", optional = true }` in `[dependencies]` and add `"dep:bevy_enoki"` to the existing `windowed` feature list in `Cargo.toml`. Author a static dep-gating test asserting `bevy_enoki` is absent from the headless dependency graph. Verify: headless `cargo test`/`cargo build` green and never compile `bevy_enoki`; `cargo build --features windowed` green. This single step retires the integration risk.
2. **Wire `EnokiPlugin` windowed-gated** in `RenderPlugin::build` (`src/windowed/render.rs`) — the same place `RonAssetPlugin::<VfxAsset>` is added today. Enoki brings its own asset loader for `Particle2dEffect`, so no `RonAssetPlugin` registration is needed for `.particle.ron`.
3. **Author ONE `.particle.ron`** (recommend `baby_flame.impact` — a single-shot burst at `TargetCenter`, the cleanest one-shot mapping) under `assets/`, and at the `spawn_effect_by_id` seam, for that ONE effect id, spawn an enoki `(ParticleSpawner, ParticleEffectHandle, OneShot, Transform)` at the resolved world anchor instead of the quad. Keep every OTHER effect id on the existing quad path (the spike touches one effect only).
4. **Fail loudly on load failure** — mirror the existing `diagnose_agumon_vfx_load` one-shot `warn!` pattern: if the `.particle.ron` handle reports `LoadState::Failed`, emit a contextual warning naming the effect; never silently spawn nothing.

The enoki spawn is presentation-only and stateless w.r.t. the kernel — it is fire-and-forget (`OneShot`), so it cannot leak lifetime into the kernel timeline. The FSM cue/barrier (`barrier.request_release` / `fire_kernel_cue`) stays exactly as-is; the spawn just hangs off the same `spawn_effect_by_id` call site that the quad spawn used.

## Implementation Landscape

### Files / purpose
- **`Cargo.toml`** — declares `bevy` with `default-features = false` (headless minimal: `bevy_asset`, `bevy_log`, `bevy_state`, `file_watcher`, `std`, executors). The `windowed` feature today = `["dep:bevy_egui", "bevy/2d", "bevy/tonemapping_luts", "bevy/dynamic_linking"]`. `bevy_egui` is the ONLY existing optional render dep, gated via `dep:bevy_egui`. **This is the exact mechanism to copy for `bevy_enoki`.** No winit/wgpu appear as direct deps — they arrive transitively via `bevy/2d`, which is itself only enabled by `windowed`.
- **`src/main.rs`** — `#[cfg(not(feature = "windowed"))] mod headless;` vs `#[cfg(feature = "windowed")] mod windowed;`. The ENTIRE windowed module (incl. `render.rs`) is excluded from the headless build at the `mod` boundary. So any `use bevy_enoki::...` inside `src/windowed/` is automatically headless-absent — the only requirement is that the Cargo dep itself is `optional` and gated.
- **`src/windowed/mod.rs`** — `register()` adds `DefaultPlugins`, `RenderPlugin`, `UiPlugin`. `EnokiPlugin` should be added inside `RenderPlugin::build` (next to `RonAssetPlugin::<VfxAsset>`).
- **`src/windowed/render.rs`** (1888 lines) — owns `setup_camera` (HDR Camera2d + Bloom::NATURAL + Tonemapping::TonyMcMapface + DebandDither), the `VfxVisuals` image handles, the `AgumonVfx` handle resource, `diagnose_agumon_vfx_load` (one-shot load-failure warn), and the VFX engine: **`spawn_effect_by_id`** (the seam) + **`advance_vfx_particles`** (per-tick quad driver).
- **`assets/digimon/agumon/vfx.ron`** — the M004 `VfxAsset`: a map of namespaced effect id → `EffectDef { placement{verb, params, anchor}, appearance{count, ttl_ticks, size_px, texture, scale[], color[], rotation}, on_expire }`. This is the data path enoki replaces (per effect). enoki's `.particle.ron` is a DIFFERENT, separate schema (`Particle2dEffect`); it does NOT reuse `VfxAsset`.
- **`tests/windowed_only/*.rs`** — all `#![cfg(feature = "windowed")]`, aggregated by `tests/windowed_only.rs` (R003 single-binary convention). Pattern for static contract tests: `include_str!("../../src/windowed/render.rs")` and assert on source text (see `vfx_windowed_contracts.rs`, which checks `setup_camera` wires HDR/Bloom/Tonemapping). The dep-gating test should live OUTSIDE this folder (it must run on the headless default build, NOT under `cfg(feature="windowed")`).

### The `spawn_effect_by_id` seam (investigated thoroughly)
Signature (`src/windowed/render.rs:1035`):
```rust
fn spawn_effect_by_id(
    commands: &mut Commands,
    asset: &VfxAsset,
    effect_id: &str,
    visuals: Option<&VfxVisuals>,
    caster_xy: [f32; 2],
    target_xy: [f32; 2],
    source_unit: UnitId,
    source_flip_x: bool,
    source_scale: f32,
) -> u32  // count of particles spawned (0 = effect id absent)
```
It resolves the `EffectDef`, computes the world anchor via `anchor_base_xy(effect.placement.anchor, caster_xy, target_xy, flip_x, scale)` (→ `Mouth`/`CasterCenter`/`TargetCenter`), then spawns N quad entities `(Sprite, Transform, VfxParticle, VfxParticleTarget, VfxParticleSource)`. **The enoki spawn replaces the quad-spawn body for the ONE chosen effect id**: compute the same `base` anchor `[x,y]`, then spawn the enoki bundle at `Transform::from_xyz(base[0], base[1], VFX_PARTICLE_Z)`. The `anchor_base_xy` placement math is REUSED for positioning; enoki then owns motion/emission/color over lifetime internally (the `EffectDef` scale/color curves + `advance_vfx_particles` are NOT used for the enoki effect).

**Call sites of `spawn_effect_by_id`** (all in `render.rs`, all on the windowed FSM cue/barrier path — confirm none move):
- `advance_agumon_presentation`: on node-enter (`on_enter_effect_ids` → charge/ember/projectile/impact/sharp_claws), and on barrier RELEASE (`request_release` returns `Released`/`DuplicateRelease`) it spawns `AGUMON_PROJECTILE_EFFECT_ID`. This is the D031/D032 impact-frame release seam — the spawn hangs off `barrier.request_release(...)` / `fire_kernel_cue()`. **enoki must NOT alter this control flow; it only changes what gets spawned for one effect id.**
- `spawn_detonate_particles`: Baby Burner detonate from a `CombatEvent` flash trigger.

For the spike, choose an effect whose spawn site is simplest and most clearly "impact": `baby_flame.impact` (TargetCenter, single burst) is the recommended target.

### Cargo feature-gating mechanism (what to add)
- `[dependencies]`: `bevy_enoki = { version = "0.6", optional = true }`
- `windowed = [..., "dep:bevy_enoki"]` (append to the existing list).
- No change to the headless `bevy` feature set. `bevy_enoki` pulls `bevy_render`/`bevy_sprite_render`/`bevy_core_pipeline`/`bevy_camera`/`bevy_mesh`/`bevy_shader` (`^0.18`) — all acceptable because they only enter under `windowed` (which already pulls `bevy/2d`).

### What exists vs missing
- EXISTS: full windowed gating architecture; the `dep:bevy_egui` optional-dep pattern; HDR Camera2d + Bloom (enoki renders fine into it); the `spawn_effect_by_id` seam + `anchor_base_xy` placement; the `diagnose_agumon_vfx_load` loud-load-failure pattern to copy; the `include_str!`-based static contract test pattern.
- MISSING: the `bevy_enoki` dep declaration; `EnokiPlugin` registration; any `.particle.ron` asset; the enoki spawn branch in `spawn_effect_by_id`; the static dep-gating/leak test (no such test exists today — gating is currently enforced by Cargo alone).

## Don't Hand-Roll (what bevy_enoki provides)

bevy_enoki is a real GPU-instanced 2D particle engine. Do NOT reinvent any of this in the quad path:
- **Emission over time** — `spawn_rate` + `spawn_amount` (the current quad path can only do one-shot spawns).
- **Per-particle lifetime + physics** — `lifetime`, `linear_speed`/`linear_acceleration`, `angular_speed`/`angular_acceleration`, `gravity_speed`/`gravity_direction`, `linear_damp`/`angular_damp`.
- **Color-over-lifetime + scale-over-lifetime curves** — `color_curve` / `scale_curve` as `MultiCurve` with per-point easing (`BounceOut`, `SineInOut`, …). Replaces the project's `eval_color`/`eval_scale` for enoki effects.
- **Emission shapes** — `emission_shape: Point | Circle`.
- **Textured / sprite-sheet particles + additive-capable materials** — `SpriteParticle2dMaterial` (texture + sprite-sheet animation via frame params); `Particle2dMaterial` trait for custom fragment shaders (this is the path to TRUE additive blending that M004/D037 deferred — see MEM084/MEM081). Default material = white quads.
- **Its own RON asset loader + hot reload** for `.particle.ron` → `Particle2dEffect`. No `RonAssetPlugin` registration needed.
- **One-shot lifecycle management** — `OneShot` deactivates/despawns the spawner after the first burst, so impact bursts self-clean without manual TTL bookkeeping.

## API / Version Notes

- **Version:** `bevy_enoki 0.6.0` (published 2026-01-25, latest). Confirmed via crates.io deps API: every `bevy_*` dependency is pinned `^0.18` — so **0.6 is the correct, Bevy-0.18-compatible release**. Project is on `bevy = "=0.18.1"` → match confirmed. Aligns with MEM091 / decision D-enoki ("bevy_enoki 0.6, Bevy 0.18-compatible").
- **Transitive deps (the leak surface to gate):** `bevy_render`, `bevy_sprite_render`, `bevy_core_pipeline`, `bevy_camera`, `bevy_mesh`, `bevy_shader`, `bevy_image`, plus `rand ^0.9.2`, `ron ^0.12` (note: enoki bundles its own `ron 0.12` for its loader — this is independent of the project's `ron 0.8` for `VfxAsset`; no conflict, both coexist).
- **Plugin:** `EnokiPlugin` — `app.add_plugins(EnokiPlugin)` inside `RenderPlugin::build`.
- **Spawn API** (prelude exports `ParticleSpawner`, `ParticleEffectHandle`, `OneShot`, `Particle2dEffect`, `SpriteParticle2dMaterial`):
```rust
use bevy_enoki::prelude::*;
commands.spawn((
    ParticleSpawner::default(),                 // white-quad material; or ParticleSpawner(material_handle)
    ParticleEffectHandle(asset_server.load("vfx/baby_flame_impact.particle.ron")),
    OneShot,                                     // deactivate/despawn spawner after first burst
    Transform::from_xyz(base[0], base[1], VFX_PARTICLE_Z),
));
```
  - `OneShot` is documented as "deactivate or delete the spawner after first burst." Exact form (unit tag vs enum w/ Deactivate/Despawn) should be confirmed against docs.rs/source at implementation time — treat as a one-shot tag for planning. For an impact burst, despawn-after-burst is the desired semantics so spawners don't accumulate.
  - For a textured additive particle, build a `SpriteParticle2dMaterial` handle from an image and pass it to `ParticleSpawner`. For the minimal spike, default (untextured) material is sufficient to prove the path; texture is a polish follow-up.
- **`.particle.ron` schema sketch** (verbatim shape from enoki's `example/assets/base.particle.ron`; the planner can hand this to an executor and tune):
```ron
(
    spawn_rate: 0.1,
    spawn_amount: 10,
    emission_shape: Point,                 // or Circle
    lifetime: (1.0, 0.5),                  // (value, randomness)
    direction: Some(((0, 1), 0.1)),        // (dir_vec, randomness)
    linear_speed: Some((100, 1)),
    linear_acceleration: Some((0, 0)),
    angular_speed: Some((0, 0)),
    angular_acceleration: Some((0, 0)),
    gravity_speed: Some((500, 0.5)),
    gravity_direction: Some(((0, -1), 0)),
    scale: Some((100., 0)),
    linear_damp: Some((20, 0.8)),
    angular_damp: Some((10, 0)),
    scale_curve: Some(MultiCurve(points: [
        (10, 0, None),
        (30, 1.0, Some(BounceOut)),
    ])),
    color_curve: Some(MultiCurve(points: [
        (LinearRgba(red: 1.0, green: 1.0, blue: 1.0, alpha: 1.0), 0, None),
        (LinearRgba(red: 1.0, green: 1.0, blue: 0.5, alpha: 0.0), 1.0, Some(SineInOut)),
    ])),
)
```
  - For an impact burst: small `lifetime`, `emission_shape: Circle`, high `spawn_amount`, outward `direction`/`linear_speed`, alpha fading to 0 in `color_curve`. Overbright RGB (>1.0) for HDR bloom is achievable since `color_curve` uses `LinearRgba` (consistent with the M004/S05 HDR-bloom intent, MEM084).

## Natural Seams (task decomposition)

- **(a) Add dep + gating + leak test** — `Cargo.toml`: `bevy_enoki` optional + `dep:bevy_enoki` on `windowed`; author the static dep-gating test (see First Proof). PROVE: headless build never compiles enoki; windowed build does. **This is the whole point of the spike — do it first, alone, before anything visual.**
- **(b) Wire `EnokiPlugin` + asset loading, windowed-gated** — add `app.add_plugins(EnokiPlugin)` in `RenderPlugin::build`; add an `AgumonEnokiImpact`-style handle resource (or load on demand) for the `.particle.ron`; add a one-shot load-failure `warn!` mirroring `diagnose_agumon_vfx_load`. PROVE: `cargo build --features windowed` + `cargo test --features windowed` green.
- **(c) Author one `.particle.ron` + spawn at the seam** — create `assets/.../baby_flame_impact.particle.ron`; in `spawn_effect_by_id`, branch on the chosen effect id to spawn the enoki `(ParticleSpawner, ParticleEffectHandle, OneShot, Transform)` at `anchor_base_xy(...)` instead of the quad loop. PROVE: windowed builds/tests green; K001 manual look check is the only visual proof (auto-mode cannot run it).
- **(d) Retire/bypass the old quad path for that ONE effect** — ensure the chosen effect id no longer also spawns quads (early-return after the enoki spawn for that id), and that its `advance_vfx_particles` quad-driver branch is bypassed. Leave ALL other effects on the quad path (S05 migrates the rest).

## First Proof (highest-risk unblocker)

**Dep compiles windowed-gated AND a static dep-gating test proves no headless leak — before authoring any effect.** This retires the integration risk that defines the slice. Concrete options for the leak test (planner picks one):

1. **`cargo metadata`/`cargo tree` graph assertion** (strongest, true static graph check). A test under the default (headless) build runs `cargo tree -e normal --no-default-features --features dev` (or parses `cargo metadata`) and asserts `bevy_enoki` (and ideally `wgpu`/`winit`) do NOT appear in the resolved graph for the headless feature set, while they DO appear under `--features windowed`. Pin it as a `#[test]` in a headless integration test (e.g. `tests/dep_gating.rs`) so it runs in the normal `cargo test`. Caveat: invoking cargo-from-cargo needs care in sandboxed CI — guard/skip gracefully if `CARGO` is unavailable.
2. **Cfg/compile-presence assertion** (cheap, complements #1). A headless test asserting `cfg!(feature = "windowed") == false` in the default build, plus the structural guarantee that any `use bevy_enoki` lives only inside the `#[cfg(feature="windowed")]`-gated `src/windowed/` module — i.e. the leak is impossible by construction because the dep is `optional` + only `dep:bevy_enoki`-referenced by `windowed`. A `Cargo.toml` static-text contract test (`include_str!("../../Cargo.toml")`) can assert `bevy_enoki` is declared `optional = true` and that the `windowed` feature lists `dep:bevy_enoki`, and that it is NOT in `default`/`dev`.

Recommend #1 as the authoritative leak proof (matches "static test proves bevy_enoki symbols are ABSENT from the headless build"), with #2 as a fast guard. The acceptance bar (M005-CONTEXT) is "a static dep-gating test proves bevy_enoki stays behind `#[cfg(feature = "windowed")]` (R005)."

## Verification (exact commands)

- `cargo test` — headless default build; MUST pass and MUST NOT compile `bevy_enoki` (the dep-gating test asserts absence). This is the nextest `agent` profile target too: `cargo nextest run --profile agent`.
- Dep-gating test specifically: `cargo test --test dep_gating` (name per the new test file) on the headless build — green = no leak.
- `cargo test --features windowed` — windowed contract + windowed_only suite green (incl. any new static contract for the enoki spawn / EnokiPlugin wiring).
- `cargo build --features windowed` — full windowed render stack + enoki compiles green.
- **K001 (manual only, NOT auto-mode):** `cargo winx` (== `cargo run --features windowed`) — user confirms the one effect renders through enoki and looks better than the quad. Auto-mode MUST NOT run this; close any visual artifact as PENDING/WAIVED per MEM087/MEM089, never PASS.

## Risks / Watch-outs

- **Transitive dep leak (the headline risk).** `bevy_enoki` hard-deps the full render stack (`bevy_render`, `bevy_sprite_render`, `bevy_core_pipeline`, `bevy_camera`, `bevy_mesh`, `bevy_shader`). If the dep is not `optional`, or is referenced outside the `windowed` feature, or if a `use bevy_enoki` accidentally lands in lib/headless code, R002/R005 break and the headless build bloats. Gate strictly via `dep:bevy_enoki` on `windowed` only; keep ALL enoki usage inside `src/windowed/`. The leak test is the guard.
- **Version mismatch.** Use `bevy_enoki = "0.6"` (or pin `=0.6.0` to mirror the project's `=0.18.1` bevy pin convention). Confirmed `^0.18` bevy deps. If a future enoki patch drifts, re-verify the bevy req.
- **`ron` version coexistence.** enoki bundles `ron 0.12`; project uses `ron 0.8` for `VfxAsset`. They are independent (enoki parses its own assets). No action needed, but the two `ron` versions WILL both appear in the windowed `Cargo.lock` — expected, not a leak.
- **Camera / render-layer coexistence.** enoki renders 2D into the existing HDR `Camera2d` (the setup with Bloom/Tonemapping/DebandDither). It needs a 2D camera, which exists. Watch z-ordering vs sprites/quads (use `VFX_PARTICLE_Z`) and that bloom doesn't blow out the effect unexpectedly. No render-layer split is required for the spike; if particles draw behind/in front incorrectly, tune the spawner Transform z. Confirm enoki's default material participates in the HDR pipeline (it targets the 2D core pipeline, which is HDR-capable here).
- **Particle lifetime must NOT leak into the kernel timeline.** The enoki spawn is fire-and-forget (`OneShot`); the FSM cue/barrier release (D031/D032 — `barrier.request_release` / `fire_kernel_cue`) must stay byte-for-byte in control of GAMEPLAY timing. Do not gate any kernel release on particle completion. enoki only renders.
- **Load failure must be loud (R013).** A missing/bad `.particle.ron` must produce a contextual one-shot `warn!` naming the effect (copy `diagnose_agumon_vfx_load`), never a silent no-op. enoki's asset is loaded via the Bevy `AssetServer`, so `LoadState::Failed` is observable the same way `VfxAsset` is.
- **K001 look check is unautomatable.** "Renders through enoki and looks better" is manual-only. Auto-mode proves only the four headless/build gates; the visual verdict is the user's.
- **Spike scope discipline.** Touch ONE effect. Leave the quad path intact for the others. The full migration (delete vs keep the quad path) is S05 and explicitly a sketch until this spike's findings land.

## Sources

- crates.io — bevy_enoki versions: latest **0.6.0**, published 2026-01-25 (`https://crates.io/crates/bevy_enoki`).
- crates.io deps API — `bevy_enoki 0.6.0` dependencies, all `bevy_*` pinned `^0.18` (`bevy_render`, `bevy_sprite_render`, `bevy_core_pipeline`, `bevy_camera`, `bevy_mesh`, `bevy_shader`, `bevy_image`, …), `rand ^0.9.2`, `ron ^0.12` (`https://crates.io/api/v1/crates/bevy_enoki/0.6.0/dependencies`).
- GitHub README — `EnokiPlugin`, `ParticleSpawner` + `ParticleEffectHandle(server.load("*.particle.ron"))`, `OneShot`, `Particle2dEffect` schema fields (`https://github.com/Lommix/bevy_enoki/blob/master/README.md`).
- docs.rs 0.6.0 — prelude exports: `EnokiPlugin`, `ParticleSpawner`, `ParticleEffectHandle`, `OneShot`, `Particle2dEffect`, `SpriteParticle2dMaterial`, `Particle2dMaterial` trait (`https://docs.rs/bevy_enoki/0.6.0/bevy_enoki/`).
- enoki `example/assets/base.particle.ron` — full RON schema example (`https://raw.githubusercontent.com/Lommix/bevy_enoki/master/example/assets/base.particle.ron`).
- Repo: `Cargo.toml` (windowed feature + `dep:bevy_egui` optional-dep pattern), `src/main.rs` (mod-level cfg gating), `src/windowed/render.rs` (`spawn_effect_by_id:1035`, `setup_camera`, `diagnose_agumon_vfx_load`, FSM cue/barrier call sites), `assets/digimon/agumon/vfx.ron`, `tests/windowed_only/vfx_windowed_contracts.rs` (static `include_str!` contract pattern), `.gsd/milestones/M005/M005-CONTEXT.md`, `.gsd/REQUIREMENTS.md` (R002/R005), MEM091/MEM084/MEM081/MEM087/MEM089/MEM070.
