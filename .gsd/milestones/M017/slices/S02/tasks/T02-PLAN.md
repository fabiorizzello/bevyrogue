---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T02: Migrate apply pipeline to StatusBag

Rewrite the status apply site in `src/combat/turn_system/pipeline.rs:721-753`: replace `commands.entity(target_entity).insert(StatusEffect { kind, duration_remaining })` with a path that ensures a `StatusBag` exists on the target and then calls `bag.apply(kind, duration_remaining)`. Use Bevy 0.18 `EntityCommands::entry::<StatusBag>().or_default()` if the API matches in this codebase; if not, fall back to a two-step `Query<Option<&mut StatusBag>>` read at the system level plus a conditional `commands.insert(StatusBag::default())` for first-application units (verify the exact shape against `Cargo.toml` Bevy version before committing — research notes this as a Bevy-version risk). Preserve the accuracy gate at lines 725-729 exactly: the `roll_pct(threshold)` check stays in front of `apply`, so a resisted re-apply still fires `OnStatusResisted` and leaves the existing duration. Continue emitting `OnStatusApplied { target, kind }` on both insert and refresh (refresh is still an apply per canon). If `bootstrap.rs` spawns units with a `StatusEffect` component in a bundle, swap to empty `StatusBag::default()` (or omit and let apply-time `entry().or_default()` handle it). Do not change event payload shapes.

## Inputs

- `src/combat/status_effect.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/bootstrap.rs`
- `.gsd/milestones/M017/slices/S02/S02-RESEARCH.md`

## Expected Output

- `src/combat/turn_system/pipeline.rs`
- `src/combat/bootstrap.rs`

## Verification

`cargo check` compiles cleanly for the apply path. Manual read: `OnStatusApplied` still fires on refresh; `OnStatusResisted` still gated by `roll_pct`.

## Observability Impact

Preserves `OnStatusApplied` on insert+refresh and `OnStatusResisted` on failed accuracy roll.
