---
id: T01
parent: S01
milestone: M006
key_files:
  - assets/digimon/agumon/baby_flame_charge.particle.ron
  - assets/digimon/agumon/baby_flame_ember.particle.ron
  - assets/digimon/agumon/baby_flame_projectile.particle.ron
  - tests/windowed_only/enoki_skill_effects_parse.rs
key_decisions:
  - Authored charge, ember, and projectile as continuous emitters (spawn_rate > 0) rather than one-shot bursts, since the charge orb must sustain through the buildup window, the ember must swirl, and the projectile must trail — the T03 lifecycle/flight systems drive despawn and travel.
  - Used a single enoki Attractor at the origin on the ember asset to recreate the quad path's converge_inward behavior instead of approximating with directional velocity.
  - Set relative_positioning Some(true) for charge/ember (orb/swirl follow the moving mouth anchor) and Some(false) for projectile (world-space particles leave a trail as the spawner moves).
duration: 
verification_result: passed
completed_at: 2026-05-26T10:37:24.870Z
blocker_discovered: false
---

# T01: Authored baby_flame charge, ember, and projectile enoki .particle.ron assets so the deferred Baby Flame sequence effects can render through enoki.

**Authored baby_flame charge, ember, and projectile enoki .particle.ron assets so the deferred Baby Flame sequence effects can render through enoki.**

## What Happened

Created three new bevy_enoki Particle2dEffect assets under assets/digimon/agumon/ for the Baby Flame sequence effects that D040 had previously deferred to the quad path:

- baby_flame_charge.particle.ron — a tight, dense CONTINUOUS emitter (spawn_rate 60, Circle(6.0)) that reads as one coherent glowing orb building at the mouth. Strong linear_damp keeps the shimmer near the core; scale_curve pops then shrinks; relative_positioning Some(true) so the orb tracks the moving mouth anchor.
- baby_flame_ember.particle.ron — a ~7-alive ember swirl (spawn_rate 10 * lifetime ~0.7) seeded from a Circle(58.0) ring matching the quad BABY_FLAME_EMBER_RADIUS_PX. A single Attractor at the origin (strength 220, min_distance 8) pulls the radially-seeded sparks inward (converge), and angular_speed gives each spark a slow spin so the ring reads as a swirl.
- baby_flame_projectile.particle.ron — a dense short-life CONTINUOUS emitter (spawn_rate 80, Circle(4.0)) reading as a comet core. relative_positioning Some(false) so emitted particles stay in world space as the T03 ProjectileFlight system moves the spawner, leaving a trail. The cross-screen travel is driven by that system, not the asset.

All three files enumerate all 19 Particle2dEffect fields explicitly (Option fields as Some(..)/None) per MEM098 since the loader's RON deserialize has no field defaults, mirroring the HDR/overbright color conventions of the existing baby_flame_impact / sharp_claws_slash assets. impact_flash and baby_burner.flash were intentionally NOT authored — they remain folded into baby_flame_impact / baby_burner_detonate by design.

Added three parse-contract tests to tests/windowed_only/enoki_skill_effects_parse.rs. Because the new assets are continuous emitters (spawn_rate > 0) rather than one-shot bursts, they can't reuse assert_one_shot_burst; added an assert_continuous_emitter helper that pins spawn_rate > 0, spawn_amount > 0, positive lifetime, and present color/scale curves. The ember test additionally asserts its attractor survived the parse.

## Verification

Ran the slice verification command `cargo test --features windowed --test windowed_only enoki`: 10 passed, 0 failed. This includes the three new tests (baby_flame_charge/ember/projectile_parses_into_enoki_schema) which load each new .particle.ron via the same ron deserialize path bevy_enoki's ParticleEffectLoader uses, proving all three parse into Particle2dEffect. The pre-existing impact/sharp_claws/baby_burner parse tests and the enoki render/dep-gating tests still pass. All three assets exist on disk under assets/digimon/agumon/.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --features windowed --test windowed_only enoki` | 0 | pass | 1050ms |

## Deviations

Added an assert_continuous_emitter helper and three tests to enoki_skill_effects_parse.rs. The task plan listed only the three RON files as expected output, but the done-when requires parse verification via the enoki parse-contract test pattern; the existing tests only covered the one-shot bursts, so continuous-emitter coverage was added to actually exercise the new assets through the real loader path.

## Known Issues

VFX visual quality (how the orb/swirl/trail actually look on screen) is K001 manual and deferred — not verified in auto-mode. The assets are tuned from the quad-path conventions but a human should confirm the visual read in `cargo run --features windowed`.

## Files Created/Modified

- `assets/digimon/agumon/baby_flame_charge.particle.ron`
- `assets/digimon/agumon/baby_flame_ember.particle.ron`
- `assets/digimon/agumon/baby_flame_projectile.particle.ron`
- `tests/windowed_only/enoki_skill_effects_parse.rs`
