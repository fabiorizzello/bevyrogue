---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T06: Smoke + grep guard + SUMMARY

Run the headless smoke CLI: `cargo run --bin combat_cli` and confirm exit 0 with no panics. Run grep guards:

1. `rg -En '\b(Burn|Freeze|Shock|DeepFreeze)\b' src/ tests/` — confirm only reserved `Burn`/`Shock` variant declarations remain in `src/combat/status_effect.rs`. Zero `Freeze` or `DeepFreeze` matches anywhere.
2. `rg "StatusEffect\b" src/ tests/` — every match must be either a `StatusEffectKind` enum reference (the enum kept its canonical name) or a comment. The legacy `StatusEffect` struct should not appear.
3. `rg "&'static mut StatusEffect\b|&mut StatusEffect\b|&StatusEffect\b" src/ tests/` — must return 0 matches.
4. `rg "StatusEffect\s*\{" src/ tests/` — must return 0 (no direct struct-literal construction).
5. `ls tests/status_effect_*.rs` — empty (legacy files deleted per T05).

Confirm `cargo check` and full `cargo test` both green (0 failed, 0 ignored).

Produce `.gsd/milestones/M017/slices/S02/S02-SUMMARY.md` via `gsd_complete_slice` describing:

- The `StatusBag` API surface for S03-S05: `apply` / `tick_all` / `cleanse_debuffs` / `has` / `get_dur` / `is_empty` / `iter`.
- The `BuffKind` classification + cleanse hook for M019's `Effect::EmitCleanse`.
- The bootstrap-seed pattern (every unit spawns with `StatusBag::default()`).
- The S01 drift reconciliation (3 legacy files deleted, `status_accuracy.rs` rewritten fresh).
- The two `mod.rs` query sites that needed migration (`ResolveActorsQuery` type alias + `advance_turn_system` inline tuple).
- Deferred work: per-status semantics (S03-S05), source attribution on `StatusInstance` (M020), `BuffKind::DR/Aura/Mark` (M019), stack-aware Heated (post-M017 per D009).

## Inputs

- `src/combat/status_effect.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/turn_system/mod.rs`
- `src/combat/follow_up.rs`
- `tests/status_refresh_max_dur.rs`
- `tests/status_multi_kind_coexist.rs`
- `tests/status_cleanse_policy.rs`
- `tests/status_accuracy.rs`

## Expected Output

- `.gsd/milestones/M017/slices/S02/S02-SUMMARY.md`

## Verification

Smoke CLI exits 0. All 5 grep guards clean. `cargo test` 0 failed / 0 ignored. SUMMARY.md persisted via `gsd_complete_slice`.

## Observability Impact

Captures the public API surface (`StatusBag`, `BuffKind`, `classify_buff_kind`, `cleanse_debuffs`) that S03-S05 and M019 will rely on; documents the bootstrap-seed invariant that S03+ systems may assume.
