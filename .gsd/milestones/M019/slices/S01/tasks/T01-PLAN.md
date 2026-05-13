---
estimated_steps: 1
estimated_files: 3
skills_used: []
---

# T01: Create DrBag component + sum_dr helper + bootstrap insert

Introduce a new `src/combat/buffs.rs` module owning `DrInstance { value: f32, duration: u32 }`, `DrBag(Vec<DrInstance>)` (derive `Component`, `Default`, `Debug`, `Clone`), a pure `sum_dr(bag: Option<&DrBag>) -> f32` helper (unclamped), and a `DrBag::tick_all() -> usize` method that decrements every instance's duration and drops zero entries (returning the count dropped, mirroring `StatusBag::tick_all`'s shape). Also expose `DrBag::apply(value: f32, duration: u32)` so future M021 `Intent::ApplyDR` work has a public seam. Re-export the module from `src/combat/mod.rs`. Insert `DrBag::default()` at the same spawn site as `StatusBag::default()` in `src/combat/bootstrap.rs:162` (and any sibling unit-spawn paths the grep surfaces, e.g. `pipeline.rs:1717` fresh-bag construction — only if it constructs full units, not a partial reset). No formula changes yet; existing tests must still pass.

## Inputs

- `.gsd/milestones/M019/slices/S01/S01-RESEARCH.md`
- `src/combat/status_effect.rs`
- `src/combat/bootstrap.rs`

## Expected Output

- `src/combat/buffs.rs`
- `src/combat/mod.rs`
- `src/combat/bootstrap.rs`

## Verification

cargo check && cargo test --lib calculate_damage && cargo test bootstrap_spawn_composition
