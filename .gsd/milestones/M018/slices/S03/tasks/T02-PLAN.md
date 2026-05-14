---
estimated_steps: 1
estimated_files: 4
skills_used: []
---

# T02: Extend TargetShape::Bounce schema to struct form { hops, selector, repeat } + damage curve in Effect::Damage

Migrate `TargetShape::Bounce(u8)` to struct variant `TargetShape::Bounce { hops: u8, selector: BounceSelector, repeat: RepeatPolicy }` in src/data/skills_ron.rs. Update the three validation gates (validate_skill_def, resolution::target_shape_is_executable_now, action_query::target_status_for_unit) to match the new struct form — keep N>=1 enforcement; reject hops==0 with UnimplementedTargetShape. In `Effect::Damage`, add an optional `per_hop` field: enum `DamageCurve { Constant, Falloff { pct: u16 }, PerHop(Vec<i32>) }` defaulting to Constant via serde default. validate_skill_def enforces: if curve is PerHop(v), v.len() == hops; if Falloff, pct <= 100. The base_damage field is unchanged and feeds the curve (Falloff applies to base; PerHop overrides base per index). Update the existing chain_bolt fixture and all tests asserting on old Bounce(u8) literal. Add a RON round-trip unit test for the struct form mirroring effect_roundtrip_damage_struct_variant, plus a positive validator test for PerHop and Falloff variants and a negative test for PerHop length mismatch.

## Inputs

- `BounceSelector + RepeatPolicy from T01`
- `existing three-gate allowlist sites from commits d4dc202/9bf931d`

## Expected Output

- `TargetShape::Bounce struct variant + DamageCurve enum in DSL`
- `three validation gates updated`
- `RON round-trip + curve validator tests`
- `chain_bolt fixture migrated to new shape`

## Verification

cargo test --lib skills_ron::tests && cargo test --lib resolution::tests && cargo check
