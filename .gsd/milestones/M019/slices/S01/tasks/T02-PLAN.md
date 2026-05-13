---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T02: Integrate DR into calculate_damage formula + DamageBreakdown

Extend `calculate_damage` in `src/combat/damage.rs` to accept a new parameter `defender_dr: Option<&DrBag>` (placed after `defender_status`). Compute `dr_sum = sum_dr(defender_dr)`, `dr_mod = (1.0 - dr_sum).max(0.0)`, multiply it into the raw formula, then apply `final_damage = round(raw).max(0)`. Add `dr_pct: i32` (integer percent, i.e. `(dr_sum * 100.0).round() as i32`) to `DamageBreakdown` and populate it. Update every internal/unit-test call site in `damage.rs` to pass `None` for the new parameter so the existing 18-row multiplicative matrix continues to pass byte-for-byte. Do NOT modify `resolution.rs` in this task (T03 owns those call sites).

## Inputs

- `src/combat/buffs.rs`
- `src/combat/damage.rs`
- `.gsd/milestones/M019/slices/S01/S01-RESEARCH.md`

## Expected Output

- `src/combat/damage.rs`

## Verification

cargo test --lib calculate_damage && cargo check
