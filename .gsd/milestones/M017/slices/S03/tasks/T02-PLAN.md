---
estimated_steps: 1
estimated_files: 4
skills_used: []
---

# T02: Wire status_amp_pct into calculate_damage + apply_effects plumb

Extend `DamageBreakdown` with `pub status_amp_pct: i32`. Change `calculate_damage` signature to take `defender_status: Option<&StatusBag>` and apply `× status_amp_pct/100` as a fourth multiplicative factor after tag_mod, tri_mod, break_mod; default 100 when bag is None. Update `apply_effects` (resolution.rs:185) to accept `defender_status: Option<&StatusBag>` and forward to `calculate_damage` at the call site `:281-285`. Update both call sites of `apply_effects` in `src/combat/turn_system/pipeline.rs:280` and `:576` to pass the defender's `&StatusBag` (already queried at `:67, :369`). Update all `apply_effects` callers in `src/combat/resolution_tests.rs` to pass `None` (regression-safe default). Update `DamageBreakdown` destructuring at `resolution.rs:281-285` to include `status_amp_pct` (unused by the OnDamageDealt event for now — kept in the breakdown for log/snapshot symmetry). Skills: api-design, design-an-interface, verify-before-complete.

## Inputs

- `src/combat/damage.rs`
- `src/combat/resolution.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/resolution_tests.rs`
- `src/combat/status_effect.rs`

## Expected Output

- `src/combat/damage.rs`
- `src/combat/resolution.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/resolution_tests.rs`

## Verification

cargo check && cargo test combat::damage_tests && cargo test combat::resolution_tests
