---
id: T03
parent: S03
milestone: M005
key_files:
  - src/windowed/render.rs
key_decisions:
  - Added a base_y field to CanvasDamageNumber (beyond the plan's age_ticks/total_ticks) so the per-tick advance hard-sets translation.y = base_y + rise_px absolutely — no drift accumulation, mirroring the SpriteRest hit-shake discipline.
  - Used TextFont::default()/embedded default font instead of loading a font asset: bevy/2d transitively enables default_font, so no assets/fonts/ dir is needed (MEM095).
  - Registered advance_canvas_damage_numbers as a standalone system .after(sample_animation_ticks) rather than inside the presentation .chain(), since it touches a disjoint component set (Text2d/CanvasDamageNumber) and needs no ordering against the sprite chain.
duration: 
verification_result: passed
completed_at: 2026-05-26T09:06:57.186Z
blocker_discovered: false
---

# T03: Spawned world-space Text2d damage numbers over struck targets that float, fade, and despawn — driven from OnHitTaken via the headless-tested kinematics projection.

**Spawned world-space Text2d damage numbers over struck targets that float, fade, and despawn — driven from OnHitTaken via the headless-tested kinematics projection.**

## What Happened

Added the floating damage-number lifecycle in the windowed binary (src/windowed/render.rs), consuming T01's pure hit_damage_amount + damage_number_kinematics lib fns so only placement/appearance is K001.

Added a binary-local component `CanvasDamageNumber { age_ticks, total_ticks, base_y }`. The plan's illustrative struct listed only age_ticks/total_ticks; I added `base_y` (the spawn Y captured once) so the per-tick advance can hard-set translation.y = base_y + rise_px absolutely — mirroring the SpriteRest/hit-shake no-drift discipline already established in T02 rather than reverse-computing the base from the previous rise.

`spawn_canvas_damage_numbers` reads MessageReader<CombatEvent> with its own cursor (MEM065): for each event where hit_damage_amount(&event.kind) is Some(amount), it resolves the target's live sprite XY via the existing find_sprite_xy and, if resolved, spawns one Text2d showing the integer amount at (x, y + DAMAGE_NUMBER_SPAWN_OFFSET_Y_PX=40) with DAMAGE_NUMBER_Z=2.0 (above VFX_PARTICLE_Z=1.0) and a CanvasDamageNumber seeded to DAMAGE_NUMBER_TICKS=12 (~1s at 12fps). One number per hit — no dedup, so multi-hit shows multiple. Targets with no live sprite resolve to None and are skipped (debug! log, no orphan number, Q5). Emits trace!(target: "windowed.agumon_playback", unit_id, amount, "spawned canvas damage number").

`advance_canvas_damage_numbers` runs per pending animation tick: ages each number, applies damage_number_kinematics(age, total) -> (rise_px, alpha), sets translation.y = base_y + rise_px and TextColor alpha via with_alpha, and despawns once age_ticks >= total_ticks so numbers cannot accumulate unbounded (Q6).

Registered the spawn bridge ordered exactly like drive_hurt_reactions (.after spawn_unit_sprites/.after resolve_action_system/.before advance_agumon_presentation/.before continue_suspended_timeline_system) and the advance system .after(sample_animation_ticks) (disjoint component set from the presentation chain, so no ordering conflict). All windowed-gated; no Text2d/2d types leak outside the binary.

Key finding (captured as MEM095): Text2d renders with the embedded default font with NO font asset, because `windowed` -> `bevy/2d` -> `default_platform` -> `default_font` (Bevy 0.18.1).

## Verification

cargo build --features windowed compiles clean (exit 0). cargo test --features windowed all green (42 windowed-only tests + all other binaries, 0 failed). cargo clippy --features windowed: my new code (CanvasDamageNumber, the two systems, constants) produces zero warnings; the remaining warnings are all pre-existing (advance_agumon_presentation/advance_vfx_particles/sync_agumon_mode and unrelated modules), confirmed by line numbers. Legible on-canvas placement/float/fade over the target is K001 (manual `cargo winx`) and was NOT run from auto-mode per K001 — requires manual user verification.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build --features windowed` | 0 | pass | 2310ms |
| 2 | `cargo test --features windowed` | 0 | pass (42 windowed tests + all binaries, 0 failed) | 20000ms |
| 3 | `cargo clippy --features windowed` | 0 | pass (no new warnings from T03 code; pre-existing warnings only) | 960ms |

## Deviations

Added a `base_y: f32` field to the `CanvasDamageNumber` component beyond the plan's illustrative `{ age_ticks, total_ticks }`. It is required to set translation.y absolutely each tick without drift; captured once at spawn. No behavioral deviation from the plan's intent.

## Known Issues

Visible appearance (legible placement/float/fade over the target on the pixel canvas) is K001 and was not exercised — it requires a manual `cargo winx` run by the user. The number color is plain white sourced from OnHitTaken.amount; kind-coloring from the co-emitted OnDamageDealt is explicitly out of scope per S03-RESEARCH.

## Files Created/Modified

- `src/windowed/render.rs`
