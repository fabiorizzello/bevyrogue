---
id: T02
parent: S02
milestone: M012
key_files:
  - src/combat/toughness.rs
  - src/headless.rs
  - src/combat/observability.rs
  - src/ui/combat_panel.rs
  - src/windowed.rs
  - tests/bootstrap_spawn_composition.rs
  - tests/validation_snapshot.rs
key_decisions:
  - Centralize toughness visibility with the existing team-aware affordance helper and a shared optional display view so ally and zero-max enemy toughness stays hidden everywhere.
  - Treat enemy toughness as a required component for snapshot diagnostics even when the visible bar is hidden, so missing real enemy break bars still fail loudly.
duration: 
verification_result: passed
completed_at: 2026-04-30T21:19:28.761Z
blocker_discovered: false
---

# T02: Hid ally and zero-bar toughness from headless, validation, and windowed combat surfaces

**Hid ally and zero-bar toughness from headless, validation, and windowed combat surfaces**

## What Happened

I updated the combat display surfaces so toughness is only surfaced as a break affordance for enemies with a real positive bar. Headless roster logging now accepts optional toughness and prints allies and hidden/zero-max cases as `N/A`; validation snapshots now store toughness as optional display data, format ally and zero-max enemy toughness as hidden, and still fail when an enemy that should expose a bar is missing its Toughness component; and the windowed combat panel now queries `Option<&Toughness>` so allies stay visible without rendering an enemy-style break bar. I also extended the bootstrap and validation tests to pin the helper contract for allies, zero-max enemies, and positive-bar enemies. The feature-gated compile surfaced a stale `TurnOrder::advance` call in `src/windowed.rs`, so I replaced it with a preview-based compatibility path to keep the windowed build green without changing the toughness contract.

## Verification

Fresh verification passed after the last code change: `cargo test-dev --test bootstrap_spawn_composition --test validation_snapshot --test toughness_enemy_only` passed with 1/1, 3/3, and 2/2 tests green respectively, and `cargo check --features "dev windowed"` completed successfully. The rerun also confirmed the windowed combat panel compiles with the new optional toughness query shape.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test-dev --test bootstrap_spawn_composition --test validation_snapshot --test toughness_enemy_only` | 0 | ✅ pass | 3700ms |
| 2 | `cargo check --features "dev windowed"` | 0 | ✅ pass | 2900ms |

## Deviations

The feature-gated compile exposed an unrelated stale `TurnOrder::advance` call in `src/windowed.rs`; I patched that compatibility path so the required windowed build gate could pass.

## Known Issues

None.

## Files Created/Modified

- `src/combat/toughness.rs`
- `src/headless.rs`
- `src/combat/observability.rs`
- `src/ui/combat_panel.rs`
- `src/windowed.rs`
- `tests/bootstrap_spawn_composition.rs`
- `tests/validation_snapshot.rs`
