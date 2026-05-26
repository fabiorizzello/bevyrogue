---
estimated_steps: 9
estimated_files: 1
skills_used: []
---

# T02: FadeOut component + advance_death_fade system (off-field exit)

Why: Per success criteria and research recommendation 2, a KO'd unit must fade off the field AFTER its death node completes — not instant-despawn on `UnitDied` read (that would break the M002 post-KO overshoot observability). The fade reuses the established `advance_vfx_particles` alpha/despawn-on-`PendingAnimationTicks` pattern.

Do: All edits in `src/windowed/render.rs` (depends on T01's `DeathExiting` marker and the modified `advance.exited` branch).
1. Add `#[derive(Component, Debug, Clone, Copy, PartialEq)] struct FadeOut { remaining_ticks: u32, total_ticks: u32 }` and a `const DEATH_FADE_TICKS: u32 = <short value, e.g. 8>;` (a few animation ticks at the 12fps clock).
2. In the `if advance.exited` branch of `advance_agumon_presentation` (render.rs:870), when the `DeathExiting` marker is present, instead of leaving the sprite frozen, `commands.entity(entity).insert(FadeOut { remaining_ticks: DEATH_FADE_TICKS, total_ticks: DEATH_FADE_TICKS })`. (Insert once — if `FadeOut` is already present, do not re-insert; the death node exits once, but guard defensively.) This requires `Entity` in the p0 query tuple (add it in T01 or here).
3. Add a pure helper `fn fade_alpha(remaining_ticks: u32, total_ticks: u32) -> f32` returning `(remaining_ticks as f32 / total_ticks.max(1) as f32).clamp(0.0, 1.0)` and unit-test it in `#[cfg(test)] mod tests` (full=1.0, half≈0.5, zero=0.0, total=0 saturates to 1.0 without divide-by-zero) following the `decrement_vfx_ttl_saturates_at_zero` pattern.
4. Add `fn advance_death_fade(mut commands: Commands, pending_ticks: Res<PendingAnimationTicks>, mut faders: Query<(Entity, &mut FadeOut, &mut Sprite)>)`: for each of `pending_ticks.0` ticks, decrement `remaining_ticks` (saturating), set `sprite.color` alpha via `fade_alpha(...)` (read the current color's RGB and rebuild with the faded alpha, e.g. via `Color::linear_rgba` like `advance_vfx_particles` does), and `commands.entity(entity).despawn()` when `remaining_ticks` reaches 0. Emit a `trace!(target: "windowed.agumon_playback", ...)` on despawn (unit/entity faded off field).
5. Register `advance_death_fade` in `RenderPlugin::build` ordered with the presentation chain — e.g. add it to the `(advance_vfx_particles, advance_agumon_presentation).chain()` group as `.after(advance_agumon_presentation)` (or chain it after) so the fade observes the marker/FadeOut set in the same frame's exited branch on the following tick, on the same `PendingAnimationTicks` clock.

Done-when: `cargo build --features windowed` green; `cargo test --features windowed` green incl. the new `fade_alpha` tests; `cargo build` (headless) green; no lib file touched. The death-marked sprite, on death-node exit, fades alpha to 0 over `DEATH_FADE_TICKS` and despawns. Skills: rust-skills, tdd, bevy-ecs-expert.

Failure modes (Q5): an entity despawned by another path mid-fade → the `Query` simply no longer yields it (no panic); `total_ticks == 0` → `fade_alpha` saturates to 1.0 (guarded by `.max(1)`). Determinism (R004): fade runs on the wall-clock-derived fixed tick and writes only `Sprite.color`/despawn — strictly downstream of presentation, never feeding the kernel.

## Inputs

- `src/windowed/render.rs`
- `assets/digimon/agumon/stance.ron`

## Expected Output

- `src/windowed/render.rs`

## Verification

cargo build --features windowed && cargo test --features windowed 2>&1 | tail -5 && cargo build 2>&1 | tail -3

## Observability Impact

Adds trace! on fade completion/despawn so the off-field exit is confirmable from logs without the windowed binary.
