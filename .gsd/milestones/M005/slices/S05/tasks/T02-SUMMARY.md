---
id: T02
parent: S05
milestone: M005
key_files:
  - assets/digimon/agumon/sharp_claws_slash.particle.ron
  - assets/digimon/agumon/baby_burner_detonate.particle.ron
  - assets/digimon/agumon/baby_flame_impact.particle.ron
  - tests/windowed_only/enoki_skill_effects_parse.rs
  - tests/windowed_only.rs
key_decisions:
  - Folded baby_burner.flash's central pop into baby_burner_detonate.particle.ron as a brighter initial color_curve core rather than a separate asset, since the seam routes one handle per effect id
  - Enriched baby_flame.impact by raising spawn_amount and linear_speed rather than authoring a separate shard layer, keeping the existing parse test's invariants intact
  - Shared one-shot invariant assertions across both new parse tests via an assert_one_shot_burst helper
duration: 
verification_result: untested
completed_at: 2026-05-26T09:46:45.007Z
blocker_discovered: false
---

# T02: Authored sharp_claws.slash + baby_burner.detonate enoki one-shot assets, enriched baby_flame.impact to fold its radiating shards, and added a parse test pinning all three to the Particle2dEffect schema

**Authored sharp_claws.slash + baby_burner.detonate enoki one-shot assets, enriched baby_flame.impact to fold its radiating shards, and added a parse test pinning all three to the Particle2dEffect schema**

## What Happened

The generalized S05 spawn seam (T01) iterates a per-effect-id handle map keyed by sharp_claws.slash, baby_flame.impact, and baby_burner.detonate, loading each .particle.ron from the paths in src/windowed/render.rs. Two assets did not exist yet; this task authored them and enriched the third. Authored sharp_claws_slash.particle.ron: a quick pale yellow-white slash burst (spawn_rate 0, spawn_amount 18, Circle(4.0), ~0.25s lifetime, outward decelerating scatter, scale_curve popping then shrinking, color_curve overbright (3.0,3.0,2.2) to transparent). Authored baby_burner_detonate.particle.ron: an orange radial shard burst folding the central baby_burner.flash pop (one-shot, spawn_amount 44 and linear_speed 220 wider than impact's 28/150, brighter core (3.4,2.0,0.8) reading as the flash resolving to hot orange (2.2,1.0,0.3) then transparent, scale_curve shrinking). Enriched baby_flame_impact.particle.ron: spawn_amount 28 to 40 and linear_speed 150 to 190 so the burst reads as central flash plus radiating shards, preserving the existing parse test's invariants. All three list all 19 Particle2dEffect fields explicitly with Option fields as Some(..)/None (MEM102, no serde defaults). No numeric gameplay payload added to SpawnParticle (R012). Authored tests/windowed_only/enoki_skill_effects_parse.rs mirroring enoki_impact_effect_parses.rs, asserting one-shot invariants via a shared assert_one_shot_burst helper, and registered the module in tests/windowed_only.rs.</narrative>
<parameter name="verificationEvidence">[{"command": "cargo test --features windowed --test windowed_only", "exitCode": 0, "verdict": "pass", "durationMs": 1138}, {"command": "cargo test --features windowed --test windowed_only enoki_skill_effects_parse", "exitCode": 0, "verdict": "pass", "durationMs": 327}]

## Verification

Ran cargo test --features windowed --test windowed_only: 49 passed, 0 failed, including the pre-existing enoki_impact_effect_parses test which confirms the enriched baby_flame_impact.particle.ron still deserializes and satisfies its one-shot/burst/curve invariants. A focused run of enoki_skill_effects_parse confirmed both new tests pass. All three assets parse into bevy_enoki::Particle2dEffect under the windowed_only harness. No windowed binary executed (K001 respected); tests are windowed-gated but only parse RON.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| — | No verification commands discovered | — | — | — |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `assets/digimon/agumon/sharp_claws_slash.particle.ron`
- `assets/digimon/agumon/baby_burner_detonate.particle.ron`
- `assets/digimon/agumon/baby_flame_impact.particle.ron`
- `tests/windowed_only/enoki_skill_effects_parse.rs`
- `tests/windowed_only.rs`
