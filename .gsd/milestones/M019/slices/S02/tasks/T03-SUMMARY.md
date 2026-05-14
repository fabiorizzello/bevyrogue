---
id: T03
parent: S02
milestone: M019
key_files:
  - src/combat/turn_system/pipeline.rs
  - tests/heal_effect.rs
key_decisions:
  - AllAllies branch uses actors.get_mut(def_entity) instead of get_many_mut — no attacker borrow needed for heal, avoids entity-collision when caster is in target list
  - AllAllies branch inserted before offensive self-skip guard so caster is included in the heal fan-out
  - KO filtering handled inside apply_heal_only (no pre-loop filter in pipeline) — double-guard is harmless since resolve_targets already excludes KO units from target_ids
duration: 
verification_result: passed
completed_at: 2026-05-14T08:48:57.921Z
blocker_discovered: false
---

# T03: Wired AllAllies heal fan-out in pipeline.rs and added tests/heal_effect.rs with 5 passing cases

**Wired AllAllies heal fan-out in pipeline.rs and added tests/heal_effect.rs with 5 passing cases**

## What Happened

Extended the MULTI-TARGET PATH block in pipeline.rs to include TargetShape::AllAllies alongside Blast and AllEnemies. Added `apply_heal_only` to the resolution imports. In the per-target loop (Phase 2), inserted an AllAllies branch before the offensive self-skip guard: it calls `actors.get_mut(def_entity)` to get only the defender unit (no attacker borrow needed), calls `apply_heal_only`, and emits OnHealed events per target. KO units are silently no-oped inside `apply_heal_only`. The caster is not skipped (unlike offensive shapes), so they self-heal if alive. Phase 1 (attacker stun/KO guard + hoisted SP/Ult resource consumption) and Phase 3 (post-loop attacker resource effects + once-per-cast events) are shared with the existing offensive paths. Created tests/heal_effect.rs using the apply_heal_only direct-call pattern (no Bevy world): 5 cases covering single heal on damaged ally, full-HP zero-amount emission, KO no-op with sp_ok=true, AllAllies fan-out with slot-ordered events and KO-slot skipped, and the hp_max cap edge case.

## Verification

cargo test --test heal_effect: 5/5 pass. cargo test (full suite): 73 test binaries, 0 failures, 0 regressions across dr_pipeline and all other integration tests. cargo check: green (warnings are pre-existing).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test heal_effect` | 0 | 5/5 pass | 3330ms |
| 2 | `cargo test` | 0 | 73 test binaries, 0 FAILED | 45000ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `src/combat/turn_system/pipeline.rs`
- `tests/heal_effect.rs`
