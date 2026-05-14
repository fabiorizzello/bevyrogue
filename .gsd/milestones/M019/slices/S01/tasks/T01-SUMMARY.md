---
id: T01
parent: S01
milestone: M019
key_files:
  - src/combat/buffs.rs
  - src/combat/mod.rs
  - src/combat/bootstrap.rs
key_decisions:
  - DrBag instances accumulate by unclamped summation — no merging or capping at the bag level; policy is caller's responsibility
  - tick_all returns usize (count dropped) mirroring StatusBag::tick_all shape
  - DrBag::apply kept as public seam for future M021 Intent::ApplyDR work
duration: 
verification_result: passed
completed_at: 2026-05-14T08:13:11.227Z
blocker_discovered: false
---

# T01: DrBag component + sum_dr helper + bootstrap insert already fully implemented in prior commit (2c09b85)

**DrBag component + sum_dr helper + bootstrap insert already fully implemented in prior commit (2c09b85)**

## What Happened

On inspection the task was already complete from a prior commit. `src/combat/buffs.rs` exists with `DrInstance { value: f32, duration: u32 }`, `DrBag(Vec<DrInstance>)` (Component + Default + Debug + Clone), `sum_dr(Option<&DrBag>) -> f32` (unclamped sum), `DrBag::tick_all() -> usize` (decrement + drop expired, returns count dropped), and `DrBag::apply(value, duration)` public seam. `src/combat/mod.rs` declares `pub mod buffs` with a doc comment. `src/combat/bootstrap.rs` imports `DrBag` and inserts `DrBag::default()` at the unit spawn site (line 164), alongside `StatusBag::default()`. All three buffs unit tests pass (`sum_dr_none_is_zero`, `sum_dr_sums_unclamped`, `tick_all_drops_expired`). Full 189-test lib suite green, `cargo check` clean.

## Verification

cargo check: clean (no errors). cargo test --lib: 189 passed including combat::buffs::tests::* (3 tests). cargo test bootstrap_spawn_composition and calculate_damage filters produced 0 matches (test names differ in codebase) but all lib tests pass.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check` | 0 | pass | 3970ms |
| 2 | `cargo test --lib` | 0 | pass — 189 tests, 0 failed | 4290ms |

## Deviations

none — work was already present from prior commit 2c09b85

## Known Issues

None.

## Files Created/Modified

- `src/combat/buffs.rs`
- `src/combat/mod.rs`
- `src/combat/bootstrap.rs`
