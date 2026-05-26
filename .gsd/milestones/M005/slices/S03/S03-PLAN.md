# S03: Hit feedback flash shake and canvas damage numbers

**Goal:** Add three transient on-hit presentation effects to the windowed view, driven from the CombatEvent bus (OnHitTaken) with no combat-state mutation (R010): a color flash and a positional shake of the struck AgumonSprite, plus a floating damage number rendered in world space (Text2d) on the pixel canvas over the target. The testable projection logic (arming, decay, flash tint, shake offset, damage-number kinematics, hit-amount extraction) lives in a windowed-gated lib module so it has deterministic headless tests; the application to Sprite.color / Transform / Text2d lives in the binary render.rs and is K001 (manual cargo winx) for visible appearance.
**Demo:** In cargo winx, each hit flashes and shakes the struck sprite and shows a floating damage number on the canvas over the target.

## Must-Haves

- Pure lib projection (HitFlashState/HitShakeState arming + decay, flash_tint, shake_offset, damage_number_kinematics, hit_damage_amount) is headless-tested with synthetic OnHitTaken events, mirroring the windowed_target_hurt test shape.
- cargo test --features windowed is green including the new tests; headless cargo test stays green (new lib module is windowed-gated, no render/text dep leaks into headless — R002/R005/R016).
- cargo build --features windowed compiles the binary with flash tint applied to the struck AgumonSprite's Sprite.color, a decaying shake offset applied to its Transform (restored to rest), and Text2d damage numbers spawned at the target's world XY that float and fade.
- cargo clippy --features windowed is clean; cargo fmt --check passes.
- K001 (manual cargo winx, user sign-off): each hit visibly flashes and shakes the struck sprite and shows a legible floating damage number on the canvas over the target — cannot be auto-verified.

## Proof Level

- This slice proves: contract + integration. Real runtime required: no for the lib projection (headless App + synthetic events); yes (--features windowed build) for the visible wiring. Human/UAT required: yes — visible flash/shake/number appearance and placement are K001-only (manual cargo winx sign-off), exactly like S01/S02. The headless-testable part is the event→state arming/decay and the pure kinematics/tint/offset/amount functions.

## Integration Closure

Upstream consumed: CombatEventKind::OnHitTaken (src/combat/observability/events.rs) as the per-hit defender value source; find_sprite_xy and advance_agumon_presentation in src/windowed/render.rs for target world-XY resolution and the per-animation-tick decay/apply loop; the spawn_unit_sprites rest position for shake. New wiring in RenderPlugin::build: observe_hit_feedback (lib) registered before advance_agumon_presentation, plus a Text2d spawn bridge and a per-tick canvas-number advance/despawn system. Remaining before the milestone is end-to-end usable: S04/S05 (bevy_enoki VFX migration) — independent of this slice.

## Verification

- New trace!(target: "windowed.agumon_playback") lines fire when flash/shake arm for a struck unit and when a canvas damage number is spawned (carrying unit_id and amount), mirroring drive_hurt_reactions' trace seam — a future agent can confirm the bridges fired from logs without running the windowed binary. No new metrics or status surfaces; no secrets/PII (only internal UnitId and integer damage amounts).

## Tasks

- [x] **T01: Add windowed-gated lib hit-feedback projection (flash/shake state + kinematics) with headless tests** `est:2h`
  Why: src/windowed/render.rs is part of the BINARY crate (declared via src/main.rs `mod windowed`), so tests/ cannot reach its private items. The only honest headless-testable seam for flash/shake/damage-number logic is LIB code, exactly like TargetHurtState/observe_target_hurt/tick_target_hurt_state in src/ui/combat_panel/mod.rs (windowed-gated lib, tested with a bare App in tests/windowed_only/windowed_target_hurt.rs). This task owns ALL testable projection logic so T02/T03 only apply it to ECS visuals.
  - Files: `src/ui/hit_feedback.rs`, `src/ui/mod.rs`, `tests/windowed_only/windowed_hit_feedback.rs`, `tests/windowed_only.rs`
  - Verify: cargo test --features windowed

- [x] **T02: Apply flash tint + shake offset to the struck AgumonSprite in render.rs** `est:1.5h`
  Why: T01 produced the testable lib projection (HitFlashState/HitShakeState + flash_tint/shake_offset). This task wires it into the binary so the struck sprite visibly flashes and shakes — the K001 half. It follows the drive_hurt_reactions registration pattern S01 established.
  - Files: `src/windowed/render.rs`
  - Verify: cargo build --features windowed

- [x] **T03: Spawn, float, fade, and despawn world-space Text2d damage numbers on the canvas** `est:1.5h`
  Why: the slice headline ('floating damage number on the canvas over the target') has zero prior art — there is no Text2d / world-space text anywhere in src/. This task adds the new entity lifecycle in the binary, consuming T01's pure hit_damage_amount + damage_number_kinematics fns (which are headless-tested) so only the placement/appearance is K001.
  - Files: `src/windowed/render.rs`
  - Verify: cargo build --features windowed

## Files Likely Touched

- src/ui/hit_feedback.rs
- src/ui/mod.rs
- tests/windowed_only/windowed_hit_feedback.rs
- tests/windowed_only.rs
- src/windowed/render.rs
