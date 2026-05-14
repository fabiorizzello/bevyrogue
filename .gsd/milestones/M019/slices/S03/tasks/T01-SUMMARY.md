---
id: T01
parent: S03
milestone: M019
key_files:
  - src/data/skills_ron.rs
  - src/combat/events.rs
key_decisions:
  - Added LegalityReasonCode::MixedEffectKinds (new variant) for the mixed Heal+Cleanse rejection — no existing code fit semantically
  - No changes needed to resolution.rs or follow_up.rs: all Effect match arms use wildcard arms
duration: 
verification_result: passed
completed_at: 2026-05-14T09:03:12.234Z
blocker_discovered: false
---

# T01: Added Effect::Cleanse variant, CombatEventKind::OnCleansed event, LegalityReasonCode::MixedEffectKinds, and ally-side + mixed-effect validators; cargo check --tests clean.

**Added Effect::Cleanse variant, CombatEventKind::OnCleansed event, LegalityReasonCode::MixedEffectKinds, and ally-side + mixed-effect validators; cargo check --tests clean.**

## What Happened

Read all four target files to understand the current data model before making changes. All Effect match arms in resolution.rs and follow_up.rs use find_map/wildcard patterns (`_ => None` or `_ => {}`), so no exhaustiveness fallout was triggered by the new variant.

Changes made:

1. **src/data/skills_ron.rs** — Added `LegalityReasonCode::MixedEffectKinds` (with doc comment) to the enum. Added `Effect::Cleanse { count: Option<u8>, target: TargetShape }` after `Effect::Heal` with a doc comment. Added two validator blocks after the existing Heal validator: (a) Cleanse ally-side target enforcement (mirrors Heal validator, rejects Bounce/AllEnemies/Blast with WrongSide); (b) mixed Heal+Cleanse rejection using MixedEffectKinds.

2. **src/combat/events.rs** — Added `CombatEventKind::OnCleansed { kinds: Vec<StatusEffectKind> }` after `OnHealed`, with doc comment matching the slice spec (empty kinds for no-op, mirrors OnHealed amount=0).

No changes were needed in resolution.rs or follow_up.rs — all existing Effect match arms use wildcard arms that tolerate the new variant without modification. The new event variant flows through serde derive automatically for JSONL logging.

## Verification

cargo check --tests returned 0 errors (only pre-existing warnings). cargo test --test validation_snapshot passed all 6 tests. No existing tests regressed. No Effect::Cleanse behavior is wired yet as intended.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check --tests` | 0 | pass — 0 errors, warnings only | 4360ms |
| 2 | `cargo test --test validation_snapshot` | 0 | pass — 6/6 tests ok | 6030ms |

## Deviations

None — all locked decisions followed exactly. count field is first in Cleanse struct as specified.

## Known Issues

None.

## Files Created/Modified

- `src/data/skills_ron.rs`
- `src/combat/events.rs`
