---
id: T04
parent: S07
milestone: M021
key_files:
  - src/combat/api/applier.rs
  - tests/passive_reactive_canon.rs
key_decisions:
  - D001
duration: 
verification_result: passed
completed_at: 2026-05-16T11:54:24.822Z
blocker_discovered: false
---

# T04: Made Tentomon's reactive block and generic passive mitigation emit a shared BlockReactionTriggered event deterministically.

**Made Tentomon's reactive block and generic passive mitigation emit a shared BlockReactionTriggered event deterministically.**

## What Happened

Refined the incoming-damage pipeline so one-shot passive mitigation modifiers are surfaced as BlockReactionTriggered, while Tentomon's armed block still reduces pre-DR damage through the same shared path. I also fixed the canon passive integration test to compare remaining HP correctly, which let the deterministic seed search find a valid block proc and confirm the baseline/miss cases still behave as expected. The Dorumon predator listener coverage stayed intact throughout.

## Verification

Ran `cargo test --test passive_reactive_canon` and `cargo test --test block_reaction_pipeline`; both passed after the HP comparison fix and the shared passive-mitigation event emission update. The passive canon now proves Dorumon listener wiring, deterministic Tentomon block behavior, and the no-proc guard path, while the generic block pipeline verifies the shared pre-damage mitigation event surface.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test passive_reactive_canon` | 0 | ✅ pass | 259ms |
| 2 | `cargo test --test block_reaction_pipeline` | 0 | ✅ pass | 271ms |

## Deviations

Adjusted the canon test's HP inequality to compare remaining HP correctly; also broadened `BlockReactionTriggered` emission to cover any consumed passive mitigation modifier, not only Tentomon-specific procs.

## Known Issues

None.

## Files Created/Modified

- `src/combat/api/applier.rs`
- `tests/passive_reactive_canon.rs`
