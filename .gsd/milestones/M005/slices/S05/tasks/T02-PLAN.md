---
estimated_steps: 1
estimated_files: 5
skills_used: []
---

# T02: Author Sharp Claws + Baby Burner enoki one-shots, enrich Baby Flame impact, with parse tests

Why: the generalized seam needs a parseable Particle2dEffect asset per routed id, and the demo requires all three skills' contact bursts to render through enoki. Do: author two new .particle.ron files mirroring the structure of baby_flame_impact.particle.ron, with ALL 19 Particle2dEffect fields explicit (MEM102 — enoki's loader has no serde defaults; Option fields as Some(..)/None): (a) assets/digimon/agumon/sharp_claws_slash.particle.ron — a quick pale yellow-white slash burst (one-shot spawn_rate 0, modest spawn_amount, short ~0.25s lifetime, small Circle emission, outward scatter, color_curve from overbright pale yellow-white to transparent, scale_curve popping in then shrinking), echoing the vfx.ron sharp_claws.slash intent (overbright (3.0,3.0,2.2) tint, target-anchored); (b) assets/digimon/agumon/baby_burner_detonate.particle.ron — an orange radial shard burst folding the central baby_burner.flash pop (one-shot, larger spawn_amount + wider linear_speed than impact, hot orange color_curve (2.2,1.0,0.3)->transparent with a brighter initial core, scale_curve shrinking). Also enrich assets/digimon/agumon/baby_flame_impact.particle.ron to fold the radiating impact_flash shards (raise spawn_amount and/or outward linear_speed/spread so the single enoki burst reads as the central flash PLUS the radiating shards) while preserving the one-shot/burst/curve invariants the existing parse test asserts. Then author tests/windowed_only/enoki_skill_effects_parse.rs (mirroring enoki_impact_effect_parses.rs): ron::from_str::<Particle2dEffect>(include_str!(..)) each of the two new assets and assert one-shot invariants (spawn_rate 0.0, spawn_amount > 0, positive lifetime.0, color_curve.is_some(), scale_curve.is_some()). Register the new module in tests/windowed_only.rs (add `#[path = "windowed_only/enoki_skill_effects_parse.rs"] mod enoki_skill_effects_parse;`). Done when: all three assets parse into Particle2dEffect under the windowed_only harness. Constraint: do not add numeric gameplay payload anywhere on the SpawnParticle command surface — these are presentation-only render assets (R012).

## Inputs

- `assets/digimon/agumon/baby_flame_impact.particle.ron`
- `tests/windowed_only/enoki_impact_effect_parses.rs`
- `tests/windowed_only.rs`

## Expected Output

- `assets/digimon/agumon/sharp_claws_slash.particle.ron`
- `assets/digimon/agumon/baby_burner_detonate.particle.ron`
- `assets/digimon/agumon/baby_flame_impact.particle.ron`
- `tests/windowed_only/enoki_skill_effects_parse.rs`
- `tests/windowed_only.rs`

## Verification

cargo test --features windowed --test windowed_only
