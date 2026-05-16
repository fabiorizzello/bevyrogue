---
id: T02
parent: S07
milestone: M021
key_files:
  - src/combat/modifiers.rs
  - src/combat/api/intent.rs
  - src/combat/api/applier.rs
  - src/combat/events.rs
  - src/combat/plugin.rs
  - src/combat/mod.rs
  - tests/block_reaction_pipeline.rs
key_decisions:
  - Treat IncomingDamage as an observational pre-damage seam only; armed state must already exist before the hit resolves.
  - Use a canonical ordered modifier fold (Intrinsic→Status→Buff→Passive) so layered modifiers stay deterministic and replayable.
duration: 
verification_result: passed
completed_at: 2026-05-16T10:49:34.615Z
blocker_discovered: false
---

# T02: Added a deterministic pre-damage modifier ledger so Block Reaction can halve incoming hits before DR and emit incoming/post-mitigation events.

**Added a deterministic pre-damage modifier ledger so Block Reaction can halve incoming hits before DR and emit incoming/post-mitigation events.**

## What Happened

Introduced a shared combat modifier layer with canonical Intrinsic→Status→Buff→Passive ordering and a target-scoped DamageModifierLedger for one-shot armed modifiers. Extended the intent surface with ApplyDamageModifier, added IncomingDamage and BlockReactionTriggered to CombatEventKind, and rewired intent_applier so damage resolution now emits the pre-damage seam, drains armed modifiers before calculate_damage runs, and emits the post-mitigation trigger after the hit is committed. Registered the new ledger in CombatPlugin and added a focused integration test covering the unchanged baseline when no modifier is armed, the armed 50% Block Reaction path against 30% DR, and deterministic replay with a fixed seed.

## Verification

Executed `cargo test --test block_reaction_pipeline`; the test passed and confirmed the baseline no-op path, the armed reduction path, the IncomingDamage → OnDamageDealt → BlockReactionTriggered ordering, and identical replay results for the same seed.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test block_reaction_pipeline` | 0 | ✅ pass | 9730ms |

## Deviations

Added CombatPlugin wiring for the new DamageModifierLedger resource and a dedicated modifier-ledger module to keep the applier path generic; the main resolution pipeline was left untouched because this task scoped the intent-applier damage route.

## Known Issues

None.

## Files Created/Modified

- `src/combat/modifiers.rs`
- `src/combat/api/intent.rs`
- `src/combat/api/applier.rs`
- `src/combat/events.rs`
- `src/combat/plugin.rs`
- `src/combat/mod.rs`
- `tests/block_reaction_pipeline.rs`
