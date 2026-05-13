---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T03: Migrate tick + expiration + both mod.rs query sites

Edit `src/combat/turn_system/mod.rs`. Update the import at line 6: replace `StatusEffect` with `StatusBag`. **Migrate BOTH query sites** (this is the gap the previous plan missed): (a) `ResolveActorsQuery` type alias at lines 53-71 — change line 67 from `Option<&'static mut StatusEffect>` to `Option<&'static mut StatusBag>`; (b) `advance_turn_system` inline query tuple at lines 358-378 — change line 369 from `Option<&mut StatusEffect>` to `Option<&mut StatusBag>`. Do not skip either — they are independent queries on different systems, not a single declaration. Rewrite the tick path at lines 465-509: for each unit's bag, call `let expired = bag.tick_all();` and iterate `expired` to emit one `OnStatusExpired { unit, kind }` per expired kind. The per-kind match arm at lines 479-485 (covering all 7 `StatusEffectKind` variants) stays a no-op placeholder for S03-S05 — preserve totality. Delete the `commands.entity(snap.entity).remove::<StatusEffect>()` call at line 500 — the bag persists empty after `tick_all` drains expired instances; removal forces re-insert on next apply (wasteful churn, breaks the bootstrap-seed invariant). Do not change event payload shapes.

## Inputs

- `.gsd/milestones/M017/slices/S02/S02-RESEARCH.md`
- `src/combat/status_effect.rs` (post-T01)
- `src/combat/turn_system/mod.rs`

## Expected Output

- `src/combat/turn_system/mod.rs`

## Verification

`cargo check` clean for mod.rs. `rg "&'static mut StatusEffect\b|&mut StatusEffect\b" src/combat/turn_system/mod.rs` returns 0 (both query sites migrated). `rg "remove::<StatusEffect>" src/` returns 0. Tick system emits exactly one `OnStatusExpired` per expired kind (verified later by T05 tests).

## Observability Impact

Per-instance `OnStatusExpired` emission instead of per-component removal; tick match arm preserved over all 7 enum variants so S03-S05 can plug per-kind effect arms without re-discovering the seam.
