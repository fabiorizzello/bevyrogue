---
estimated_steps: 4
estimated_files: 1
skills_used: []
---

# T02: Apply flash tint + shake offset to the struck AgumonSprite in render.rs

Why: T01 produced the testable lib projection (HitFlashState/HitShakeState + flash_tint/shake_offset). This task wires it into the binary so the struck sprite visibly flashes and shakes — the K001 half. It follows the drive_hurt_reactions registration pattern S01 established.

Do (all in src/windowed/render.rs): (1) Register `bevyrogue::ui::hit_feedback::observe_hit_feedback` plus `HitFlashState`/`HitShakeState` resources in RenderPlugin::build, ordered `.after(spawn_unit_sprites).after(resolve_action_system).before(advance_agumon_presentation)` like drive_hurt_reactions (init the resources via .init_resource). (2) In spawn_unit_sprites, also insert a binary-local `SpriteRest { xy: Vec2 }` component capturing the spawn (x, 0.0) so shake can restore to the exact rest position without accumulating drift (research warns hardcoding ±200 goes stale; capture at spawn). (3) In advance_agumon_presentation: decay both states ONCE per frame by pending_ticks.0 (call decay_by); change the p0 query's `&Transform` to `&mut Transform` and add `&SpriteRest`; each tick set `transform.translation = (rest.xy + shake_offset(remaining, SHAKE_TICKS)) extended with the existing z`, and when shake remaining is 0 hard-set translation back to rest.xy (never accumulate). (4) Write `render_sprite.color = flash_tint(flash_remaining, FLASH_TICKS)` every tick (flash is the SOLE color writer for AgumonSprite; flash_tint returns WHITE at remaining 0, so steady state stays white) — but GUARD: skip the color write when `death_exiting.is_some() || fade_out.is_some()` so it never fights advance_death_fade's alpha lerp. (5) Add a trace!(target: "windowed.agumon_playback", unit_id, "flash+shake armed") when a unit is freshly armed. Keep these as pure overlays: do NOT touch sprite.mode, sprite.player, or the barrier — they must not perturb the kernel-barrier release logic (D031/D032).

Failure modes (Q5): a dropped/duplicated OnHitTaken simply fails to arm or re-arms to full (idempotent, no stuck state); a hit on a UnitId with no live sprite is a no-op (the resource entry just decays unused). Accept that the brief shake perturbs VFX mouth/caster anchors while active (sprite_positions is read from the shaken Transform) — documented and acceptable for a sub-second shake per S03-RESEARCH.

Done when: cargo build --features windowed compiles, cargo test --features windowed stays green (no regression), and cargo clippy --features windowed is clean. Visible flash + shake is K001 (manual cargo winx). skills_used: bevy-ecs-expert, rust-development.

## Inputs

- `src/windowed/render.rs`
- `src/ui/hit_feedback.rs`

## Expected Output

- `src/windowed/render.rs`

## Verification

cargo build --features windowed

## Observability Impact

Adds trace!(target: "windowed.agumon_playback") when flash/shake arm for a struck unit_id — confirms the bridge fired from logs without running the windowed binary.
