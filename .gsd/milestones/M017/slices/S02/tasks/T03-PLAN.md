---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T03: Migrate tick + expiration to StatusBag

Rewrite the tick path in `src/combat/turn_system/mod.rs:465-509`: switch the query from `Option<&'static mut StatusEffect>` to `Option<&'static mut StatusBag>`. For each unit's bag, call `bag.tick_all()` and iterate the returned expired kinds, emitting one `OnStatusExpired { unit, kind }` event per expired instance. The per-kind match arm stays empty (S03-S05 will hook DoT/amp/skip/delay/dealt-buff here). Leave the bag component in place even if empty after expiry (cheap, avoids re-insert churn on next apply). Do not change event payload shapes.

## Inputs

- `src/combat/status_effect.rs`
- `src/combat/turn_system/mod.rs`
- `.gsd/milestones/M017/slices/S02/S02-RESEARCH.md`

## Expected Output

- `src/combat/turn_system/mod.rs`

## Verification

`cargo check` clean. The tick system emits exactly one `OnStatusExpired` per expired instance (verified later by T05 tests).

## Observability Impact

Per-instance `OnStatusExpired` emission instead of per-component removal.
