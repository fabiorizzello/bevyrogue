---
estimated_steps: 1
estimated_files: 3
skills_used: []
---

# T01: Author baby_flame charge, ember, projectile .particle.ron enoki assets

Why: enoki can only render an effect id that has a loaded Particle2dEffect asset. The three Baby Flame sequence effects that were deferred to the quad path by D040 (charge buildup, ember swirl, traveling projectile) now need enoki assets so D043 (enoki is sole renderer) can be satisfied. impact_flash and baby_burner.flash are NOT authored — they are already folded into baby_flame_impact.particle.ron and baby_burner_detonate.particle.ron respectively (their on_expire chains die with the quad system, by design). Do: author three new files under assets/digimon/agumon/ — baby_flame_charge.particle.ron (a tight charge orb building at the mouth), baby_flame_ember.particle.ron (a small ember swirl, ~7 particles), baby_flame_projectile.particle.ron (a continuous/short-burst emitter that reads as a traveling flame core; the cross-screen travel is driven by the ProjectileFlight system in T03, not by the asset). Each file MUST enumerate ALL 19 Particle2dEffect fields explicitly (Option fields as Some(..)/None) because Particle2dEffect has no serde defaults — a missing field fails the RON parse at asset load (MEM098). Mirror the structure and HDR/overbright color conventions of the existing baby_flame_impact.particle.ron / sharp_claws_slash.particle.ron. Done-when: all three files parse — verified by the enoki parse-contract test pattern (the enoki_impact_effect_parses / enoki_skill_effects_parse suites load .particle.ron via the real loader) and the assets exist on disk.

## Inputs

- `assets/digimon/agumon/baby_flame_impact.particle.ron`
- `assets/digimon/agumon/baby_burner_detonate.particle.ron`
- `assets/digimon/agumon/sharp_claws_slash.particle.ron`
- `assets/digimon/agumon/vfx.ron`
- `tests/windowed_only/enoki_impact_effect_parses.rs`
- `tests/windowed_only/enoki_skill_effects_parse.rs`

## Expected Output

- `assets/digimon/agumon/baby_flame_charge.particle.ron`
- `assets/digimon/agumon/baby_flame_ember.particle.ron`
- `assets/digimon/agumon/baby_flame_projectile.particle.ron`

## Verification

cargo test --features windowed --test windowed_only enoki
