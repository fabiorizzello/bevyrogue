---
id: T04
parent: S05
milestone: M012
key_files:
  - src/combat/turn_system/pipeline.rs
key_decisions:
  - Only pure self-resource / self-advance Form Identity effects retarget to self; hidden combat follow-ups keep their external enemy target.
  - The runtime pipeline was split into explicit same-entity and attacker-vs-defender paths to avoid Bevy query aliasing and restore compilation.
  - Energy grants continue to flow through `RoundEnergyTracker` first, then `Energy::gain_capped`, so event amounts stay truthful.
duration: 
verification_result: passed
completed_at: 2026-05-01T08:31:56.033Z
blocker_discovered: false
---

# T04: Fix Form Identity self-targeting and DORUgamon follow-up targeting in the combat pipeline

**Fix Form Identity self-targeting and DORUgamon follow-up targeting in the combat pipeline**

## What Happened

I repaired the broken combat action pipeline in `src/combat/turn_system/pipeline.rs` by rebuilding `step_app` around two explicit execution paths: a same-entity hidden/self-resource path and the standard attacker-vs-defender path. The rewrite restores the scoped Bevy component bindings that had broken compilation, keeps energy grants authoritative through `RoundEnergyTracker` and `Energy::gain_capped`, and preserves event emission and KO/stun handling in the live pipeline. I also narrowed Form Identity `SelfOnly` retargeting so only pure self-resource or self-advance effects retarget to the source, while hidden combat follow-ups like DORUgamon's toughness hit keep the externally selected enemy target. After the fix, the targeted test suite compiles and passes, including the canonical Greymon/Kyubimon energy/self-advance cases, DORUgamon and Angemon Form Identity behavior, and the resource-cap assertions.

## Verification

Ran `cargo test-dev --test form_identity --test resource_caps`; the project compiled cleanly and all targeted tests passed. Verified behavior includes Greymon Form Identity energy grants, Kyubimon self-advance, DORUgamon enemy-targeted toughness follow-up, Angemon attribute-triggered follow-up, and the round energy cap checks.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test-dev --test form_identity --test resource_caps` | 0 | ✅ pass | 172ms |

## Deviations

No test files were changed; the existing `form_identity.rs` and `resource_caps.rs` coverage already pinned the repaired behaviors, so the work stayed focused on the runtime fix.

## Known Issues

None.

## Files Created/Modified

- `src/combat/turn_system/pipeline.rs`
