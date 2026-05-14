---
estimated_steps: 1
estimated_files: 6
skills_used: []
---

# T02: Thread attacker_dmg_mult through apply_effects and apply Blessed ×1.15

Add an `attacker_dmg_mult: f32` parameter to `calculate_damage` in `src/combat/damage.rs` (folded into the final product as another factor; default 1.0 from call sites without Blessed context). In `src/combat/resolution.rs::apply_effects` accept an `attacker_statuses: Option<&StatusBag>` and compute `1.15 if has Blessed else 1.0`, passing it to every `calculate_damage` call. Update the two `apply_effects` call sites in `src/combat/turn_system/pipeline.rs` (~280, ~576) to fetch the attacker StatusBag from the existing tuple and pass it through. Insert `None` at all other call sites in `tests/resolution_tests.rs` mechanically — they pass no buff context today. Add `tests/status_blessed_offensive.rs`: spawn attacker with/without Blessed, fire a Basic, assert the dmg event shows `round(base*tag*tri*break*1.15)` vs `round(base*tag*tri*break)`.

## Inputs

- `src/combat/damage.rs`
- `src/combat/resolution.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/status_effect.rs`
- `.gsd/milestones/M017/slices/S05/S05-RESEARCH.md`

## Expected Output

- `src/combat/damage.rs`
- `src/combat/resolution.rs`
- `src/combat/turn_system/pipeline.rs`
- `tests/resolution_tests.rs`
- `tests/damage_tests.rs`
- `tests/status_blessed_offensive.rs`

## Verification

cargo check && cargo test --test status_blessed_offensive && cargo test --test damage_tests && cargo test --test resolution_tests
