---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T03: Wire AllAllies multi-target fan-out in pipeline.rs and add tests/heal_effect.rs

In `src/combat/turn_system/pipeline.rs` (around line 175), extend the multi-target block currently handling `Blast | AllEnemies` to also handle `AllAllies`. Hoist SP/Ult/streak resource consumption once (the existing pattern already does this); per-target dispatch chooses `apply_damage_only` for offensive shapes and `apply_heal_only` for AllAllies. Keep the existing damage paths unchanged. Then add `tests/heal_effect.rs` using the apply_effects direct-call pattern established in tests/dr_pipeline.rs (no Bevy world): (1) Single heal on damaged ally — amount = floor(hp_max * pct / 100), capped to hp_max - hp_current, OnHealed { amount, hp_after } emitted; (2) Single heal at full HP — amount = 0, event still emitted with amount=0; (3) Single heal on KO target — no state change, no event, sp_ok still true (no SP consumed); (4) AllAllies heal with 1 KO + 2 alive damaged — KO untouched and no event; both alive receive heal with OnHealed events ordered by slot_index ascending; (5) Cap test — ally at hp_max-3 with 50% heal → healed exactly 3, hp_after == hp_max. Naming is functional per CLAUDE.md (no s##_ prefix). Tests must be deterministic: no RNG, no wall-clock. If a RON fixture is needed, prefer a test-only fixture under tests/fixtures/ over editing assets/data/skills.ron to keep baseline JSONL trace identity neutral.

## Inputs

- `src/combat/turn_system/pipeline.rs`
- `src/combat/resolution.rs`
- `src/combat/state.rs`
- `src/combat/events.rs`
- `src/data/skills_ron.rs`
- `tests/dr_pipeline.rs`
- `.gsd/milestones/M019/slices/S02/S02-RESEARCH.md`

## Expected Output

- `src/combat/turn_system/pipeline.rs`
- `tests/heal_effect.rs`

## Verification

cargo test --test heal_effect — all 5 cases pass. cargo test (full suite) — green, no regression in dr_pipeline.rs or other integration tests. cargo check — green.

## Observability Impact

AllAllies fan-out emits one OnHealed event per alive ally (slot-ordered) and zero events for KO allies — confirmed by test case 4.
