---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T06: Smoke + grep guard + SUMMARY

Run the headless smoke CLI: `cargo run --bin combat_cli` and confirm exit 0 with no panics. Re-run the S01 grep guard `grep -rEn '\b(Burn|Freeze|Shock|DeepFreeze)\b' src/ tests/` and confirm only the reserved Burn/Shock variant declarations remain (no new legacy references introduced by S02). Confirm `cargo check` and full `cargo test` both green (0 failed, 0 ignored). Produce `.gsd/milestones/M017/slices/S02/S02-SUMMARY.md` via `gsd_complete_slice` describing the migration, the `StatusBag` API surface for S03-S05, and the cleanse hook for M019.

## Inputs

- `src/combat/status_effect.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/turn_system/mod.rs`
- `src/combat/follow_up.rs`
- `tests/status_refresh_max_dur.rs`
- `tests/status_multi_kind_coexist.rs`
- `tests/status_cleanse_policy.rs`

## Expected Output

- `.gsd/milestones/M017/slices/S02/S02-SUMMARY.md`

## Verification

Smoke CLI exits 0. Grep guard clean. `cargo test` 0 failed / 0 ignored. SUMMARY.md persisted via `gsd_complete_slice`.

## Observability Impact

Captures the public API surface (StatusBag, BuffKind, cleanse_debuffs) that downstream slices and M019 will rely on.
