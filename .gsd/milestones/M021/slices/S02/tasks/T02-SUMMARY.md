---
id: T02
parent: S02
milestone: M021
key_files:
  - src/combat/api/registry.rs
  - src/combat/api/skill_ctx.rs
  - src/combat/api/timeline.rs
key_decisions:
  - SelectorCtx<'a>/CueCtx<'a> use the default S=() in the Fn type aliases — concrete S type is supplied by the runner in T03+, keeping registry.rs import-free from higher-level context types.
  - world: &'a bevy::prelude::World chosen as the simplest F7 promotion — direct ECS read access without thread-local indirection; full query API arrives when the runner is wired in T03.
  - FormulaExt/TickExt/AiUtilityExt intentionally left as fn() with explicit S05/S07 comments to stay within S02 scope.
duration: 
verification_result: passed
completed_at: 2026-05-15T07:43:37.176Z
blocker_discovered: false
---

# T02: Promoted ExtPoint::Fn signatures for Hook/Selector/Predicate/Cue to real for<'a> fn(...) shapes and extended SkillCtx<'a> with registries, world, and cast_hit_set borrows.

**Promoted ExtPoint::Fn signatures for Hook/Selector/Predicate/Cue to real for<'a> fn(...) shapes and extended SkillCtx<'a> with registries, world, and cast_hit_set borrows.**

## What Happened

Edited registry.rs to add imports from super::timeline (BeatEvent, SelectorCtx, CueCtx) and super::skill_ctx (SkillCtx), then replaced the four fn() placeholders: HookExt::Fn = for<'a> fn(&BeatEvent, &mut SkillCtx<'a>), SelectorExt::Fn = for<'a> fn(&SelectorCtx<'a>) -> Vec<UnitId>, PredicateExt::Fn = for<'a> fn(&BeatEvent, &SkillCtx<'a>) -> bool, CueExt::Fn = for<'a> fn(&CueCtx<'a>) -> &'static str. FormulaExt, TickExt, AiUtilityExt kept as fn() with explicit comments noting S05/S07 refinement. Edited skill_ctx.rs to add three new public fields: registries (&'a ExtRegistries), world (&'a bevy::prelude::World, F7 promotion replacing spike thread-locals), and cast_hit_set (&'a mut HashSet<UnitId> for NoRepeat selector / chain-bolt pattern). Updated SkillCtx::new signature accordingly. No existing call sites for SkillCtx::new existed in src/ (intent_applier builds no SkillCtx). Updated the populated_regs() helper in timeline.rs inline tests to register properly-typed function pointers (noop_hook, noop_selector, noop_pred) instead of || {} closures which would no longer match the promoted Fn types. Circular module references (registry ↔ skill_ctx, registry ↔ timeline) are fine within a single Rust crate — no structural issue.

## Verification

cargo check (headless) — 0 errors, only pre-existing warnings. cargo check --features windowed — 0 errors. cargo test --lib combat::api — 14/14 passed (4 registry, 6 rng, 4 timeline). cargo test --test intent_applier_canary --test cast_id_propagation — 5/5 passed.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check` | 0 | pass | 4200ms |
| 2 | `cargo check --features windowed` | 0 | pass | 3100ms |
| 3 | `cargo test --lib combat::api` | 0 | pass — 14/14 | 1180ms |
| 4 | `cargo test --test intent_applier_canary --test cast_id_propagation` | 0 | pass — 5/5 | 22880ms |

## Deviations

populated_regs() in timeline.rs inline tests updated as a necessary side-effect of the type changes (not listed in expected output files, but required for cargo test to pass).

## Known Issues

none

## Files Created/Modified

- `src/combat/api/registry.rs`
- `src/combat/api/skill_ctx.rs`
- `src/combat/api/timeline.rs`
