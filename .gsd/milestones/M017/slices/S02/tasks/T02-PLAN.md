---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T02: Bootstrap-seed StatusBag + migrate apply pipeline

Edit `src/combat/bootstrap.rs:160`: add `StatusBag::default(),` to the unit spawn bundle, adjacent to `RoundFlags::default()`. This eliminates the "create-on-first-apply" branch — every unit always has a bag, matching the seeding pattern already used for `RoundFlags`, `BasicStreak`, `Energy`, etc. Edit `src/combat/turn_system/pipeline.rs:21` import: replace `StatusEffect` with `StatusBag`. Edit the apply site at `src/combat/turn_system/pipeline.rs:721-753`: replace `commands.entity(target_entity).insert(StatusEffect { kind, duration_remaining })` (line 731) with a query-driven `bag.apply(kind, duration_remaining)` call against `&mut StatusBag`. Preserve the accuracy gate at lines 725-729 — `roll_pct(threshold)` stays in front of `apply`, so resisted re-apply still fires `OnStatusResisted` and never calls `apply`. Continue emitting `OnStatusApplied { target, kind }` on both insert and refresh — emit AFTER `apply` returns, unconditional within the gate-passed branch (refresh is still an apply per canon). Pick the simplest borrow shape that compiles: prefer adding `Query<&mut StatusBag>` to the apply system if it doesn't conflict; if it does, use `commands.entity(target).queue(move |entity_mut| { ... })` to fetch the bag from inside the deferred closure.

## Inputs

- `.gsd/milestones/M017/M017-CONTEXT.md`
- `.gsd/milestones/M017/M017-ROADMAP.md`
- `.gsd/milestones/M017/slices/S02/S02-RESEARCH.md`
- `src/combat/bootstrap.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/status_effect.rs` (post-T01)

## Expected Output

- `src/combat/bootstrap.rs`
- `src/combat/turn_system/pipeline.rs`

## Verification

`cargo check` clean for pipeline + bootstrap paths. Manual diff read: `OnStatusApplied` fires on refresh; `OnStatusResisted` still gated by `roll_pct`; bootstrap inserts `StatusBag::default()` for every spawned unit (`rg "StatusBag::default" src/combat/bootstrap.rs` returns ≥1).

## Observability Impact

Bootstrap-seed means S03+ semantic systems never have to branch on "bag exists?" — every unit ships with one. The apply path keeps the `OnStatusApplied` / `OnStatusResisted` event surface unchanged in shape.
