---
estimated_steps: 4
estimated_files: 1
skills_used: []
---

# T03: Spawn, float, fade, and despawn world-space Text2d damage numbers on the canvas

Why: the slice headline ('floating damage number on the canvas over the target') has zero prior art — there is no Text2d / world-space text anywhere in src/. This task adds the new entity lifecycle in the binary, consuming T01's pure hit_damage_amount + damage_number_kinematics fns (which are headless-tested) so only the placement/appearance is K001.

Do (all in src/windowed/render.rs): (1) Add a binary-local component `CanvasDamageNumber { age_ticks: u32, total_ticks: u32 }`. (2) Add a bridge system `spawn_canvas_damage_numbers` reading MessageReader<CombatEvent> (own cursor, MEM065): for each event where `hit_damage_amount(&event.kind)` is Some(amount), resolve the target world XY via the existing find_sprite_xy(&sprites, event.target); if resolved, spawn a Text2d entity showing the amount at (x, y + small +Y offset) with z >= 2.0 (above VFX_PARTICLE_Z = 1.0) and a CanvasDamageNumber initialized to a total lifetime in ticks. Each hit spawns its OWN number (per-hit, not deduped — multi-hit shows multiple). Emit trace!(target: "windowed.agumon_playback", unit_id = ?event.target, amount, "spawned canvas damage number"). (3) Add a per-tick advance system `advance_canvas_damage_numbers` that, per pending animation tick, increments age_ticks, applies damage_number_kinematics(age, total) -> (rise_px, alpha) by setting Transform.translation.y = base_y + rise_px and the Text/TextColor alpha, and despawns the entity when age_ticks >= total_ticks. Register both in RenderPlugin::build: the spawn bridge ordered like drive_hurt_reactions (.after spawn_unit_sprites/.after resolve_action_system/.before advance_agumon_presentation), and the advance system after sample_animation_ticks alongside advance_vfx_particles. Keep everything windowed-gated; no Text2d/2d types may appear outside the windowed binary path.

Failure modes (Q5): a hit for a target with no live sprite resolves find_sprite_xy to None and is skipped (no orphan number). Load (Q6): a multi-hit window spawns one Text2d per hit; volume is low in the single-dummy combat, but the per-tick advance despawns each at end-of-life so entities cannot accumulate unbounded. The white number is sourced from OnHitTaken.amount (simplest honest scope per S03-RESEARCH); kind-coloring from the co-emitted OnDamageDealt is explicitly out of scope.

Done when: cargo build --features windowed compiles, cargo test --features windowed stays green, and cargo clippy --features windowed is clean. Legible placement/float/fade over the target on the pixel canvas is K001 (manual cargo winx). skills_used: bevy-ecs-expert, rust-development.

## Inputs

- `src/windowed/render.rs`
- `src/ui/hit_feedback.rs`

## Expected Output

- `src/windowed/render.rs`

## Verification

cargo build --features windowed

## Observability Impact

Adds trace!(target: "windowed.agumon_playback") on each canvas damage number spawn (target unit_id + amount) — lets a future agent confirm the number bridge fired from logs.
