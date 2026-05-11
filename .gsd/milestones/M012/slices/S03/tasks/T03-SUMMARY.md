---
id: T03
parent: S03
milestone: M012
key_files:
  - src/combat/resolution.rs
  - src/combat/resolution_tests.rs
key_decisions:
  - Use `SkillDef.targeting.shape` as the sole source of truth for `ResolvedAction.target_shape`.
  - Treat effect-shape mismatch as a validation boundary, not a runtime inference path.
duration: 
verification_result: passed
completed_at: 2026-04-30T22:00:42.475Z
blocker_discovered: false
---

# T03: Resolved target shape from SkillDef.targeting metadata instead of effect inference.

**Resolved target shape from SkillDef.targeting metadata instead of effect inference.**

## What Happened

Updated combat resolution so `ResolvedAction.target_shape` is copied directly from `SkillDef.targeting.shape`, removing the old effect-scanning fallback. Refined the resolution unit tests to prove metadata wins over a mismatched `Effect::Damage` target shape and that revive/no-damage skills still resolve from explicit targeting metadata. The S02 rejection path remains intact: Row and AllEnemies skills still fail before mutation with the stable `UnimplementedTargetShape:<Shape>` reason, and the canonical skills-book validation still passes.

## Verification

Fresh verification after the final code edit passed: the slice regression suite confirmed metadata-driven target-shape resolution, revive semantics, and pre-mutation unsupported-shape rejection; the canonical skills-book validation also passed.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test-dev --test target_shape_truthfulness --test revive_semantics --test patamon_revive && cargo test-dev skills_ron` | 0 | ✅ pass | 2900ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/combat/resolution.rs`
- `src/combat/resolution_tests.rs`
