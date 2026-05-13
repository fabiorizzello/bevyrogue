---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T04: Add chain_bolt fixture skill + RON round-trip test for Bounce(N) inside Effect::Damage

Add an Implemented fixture skill `chain_bolt` to `assets/data/skills.ron`. Targeting: `shape: Bounce(3)`, side: Enemy, life: Alive. Effects: `Damage { target: Bounce(3), base_damage: 18, damage_tag: ... }` (target must equal targeting.shape exactly — copy-paste pitfall; see resolution.rs:301). Include a `ToughnessHit`-style toughness damage value matching project conventions (cross-reference existing `nova_burst` Blast fixture). Add a focused RON round-trip unit test in `src/data/skills_ron.rs` `#[cfg(test)] mod tests` mirroring `effect_roundtrip_damage_struct_variant` (line ~461) but for `Effect::Damage { target: Bounce(3), ... }` — proves tuple-inside-struct-variant deserialization is correct. Also add a load-and-validate test confirming `chain_bolt` survives the full `SkillBook::load` pipeline.

## Inputs

- ``assets/data/skills.ron` — current Blast fixture `nova_burst` and AllEnemies fixture `dark_flood` (S02 T05) as template`
- ``src/data/skills_ron.rs` (T01/T02 outputs) — Bounce(u8) variant, validate_skill_def gate widened`
- ``src/data/skills_ron.rs` — existing `effect_roundtrip_damage_struct_variant` test (line ~461)`

## Expected Output

- ``assets/data/skills.ron` — new `chain_bolt` SkillDef entry with Bounce(3) targeting and matching Effect::Damage target`
- ``src/data/skills_ron.rs` — new round-trip test for Bounce(3) inside Effect::Damage; new chain_bolt load-and-validate test`

## Verification

cargo test --lib skills_ron::tests::chain_bolt_roundtrip && cargo test --test data_loaders
