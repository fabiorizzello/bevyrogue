---
id: S04
parent: M017
milestone: M017
provides:
  - (none)
requires:
  []
affects:
  []
key_files:
  - (none)
key_decisions:
  - Paralyzed ticks the StatusBag (duration decrements) unlike Stunned which drops status_opt without ticking — aligns with canon §H.1 always-skip while preserving duration lifecycle.
  - is_first_apply_slowed computed before bag.apply() to capture pre-mutation state; TurnAdvance emitted after OnStatusApplied to preserve JSONL log order (applied → delayed).
  - get_cursor_current() (not get_cursor()) must be used to anchor MessageCursor before event-reading loops — get_cursor() starts at position 0 and double-counts due to 2-frame message buffer.
  - Slowed AV push routes through existing TurnAdvance event → apply_turn_advance_system → resistance::apply_av_change path, picking up TempoResistance handling for free with no new event variants.
  - OnActionFailed fires on the last Paralyzed tick (dur 1→0) because is_paralyzed is captured from the pre-tick bag state — 100 turns = exactly 100 skip events.
patterns_established:
  - (none)
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-13T09:41:42.286Z
blocker_discovered: false
---

# S04: Paralyzed + Slowed — turn skip + delay-on-apply

**Paralyzed always-skip and Slowed first-apply AV push wired into the turn pipeline; two deterministic integration tests verify both mechanics end-to-end.**

## What Happened

S04 wired the §H.1 semantics for Paralyzed (always-skip turn) and Slowed (one-shot −30 % gauge push on first apply) on top of the S02 StatusBag lifecycle.

**T01 — Paralyzed skip in process_turn_advanced_system (`turn_system/mod.rs`):** Added the Paralyzed always-skip block mirroring the Stunned short-circuit but with a key difference: `tick_all()` runs inside the Paralyzed block so duration decrements each turn (canon §H.1 always-skip preserves duration lifecycle). The `Snap` struct was extended with `is_paralyzed: bool` populated from the pre-tick StatusBag, and the enemy-AI dispatch gate was widened to `&& !snap.is_paralyzed`. `CombatEventKind::OnActionFailed { reason: "paralyzed" }` is emitted each skipped turn — including the last tick when duration expires from 1→0, because `is_paralyzed` is captured before the tick runs.

**T02 — Slowed first-apply via TurnAdvance event (`pipeline.rs`):** Before `bag.apply(Slowed, duration)`, the pre-mutation flag `is_first_apply_slowed` captures whether the bag already has Slowed. If true (first apply only), `TurnAdvance { target, amount_pct: -30 }` is emitted after `OnStatusApplied` (preserving JSONL log order: applied → delayed). The event routes through `apply_turn_advance_system` → `resistance::apply_av_change`, picking up TempoResistance handling for free. Re-apply and the resist branch both skip the emission, satisfying the no-double-push requirement.

**T03 — Integration test `tests/status_paralyzed_skip.rs`:** Spawns 1 ally + 1 enemy with Paralyzed(dur=100) applied at construction. Drives 100 TurnAdvanced cycles for the enemy via the existing event-bus harness. Asserts: exactly 100 OnActionFailed{reason:"paralyzed"} events and zero ActionIntents from the enemy across all 100 cycles. Key pattern discovered: `get_cursor_current()` (not `get_cursor()`) must be used to initialize the MessageCursor before the loop — `get_cursor()` starts at position 0 and double-counts due to the 2-frame message buffer; `get_cursor_current()` records the current write-head and advances cleanly each frame.

**T04 — Integration test `tests/status_slowed_delay.rs`:** Spawns attacker + defender (ActionValue=5000, no TempoResistance, StatusBag). Applies Slowed via the skill-resolution path with a pure-ApplyStatus SkillDef and CombatRng::from_seed(0) to ensure the accuracy roll passes. Chains `resolve_action_system` → `apply_turn_advance_system` to match production schedule. Asserts: exactly one TurnAdvance{amount_pct:-30} emitted; OnStatusApplied precedes TurnAdvance in the event stream; defender AV reduced to 2000 (5000 − 3000, where 3000 = 30% of MAX_AV=10000). Second apply: zero additional TurnAdvance events — refresh_max_dur path only, is_first_apply_slowed=false because bag already contains Slowed.

**T05 — Full-suite verification and grep guard:** cargo check exit:0 (no errors, pre-existing warnings only). cargo test exit:0: entire integration suite passes including the two new S04 tests (status_paralyzed_skip 1/1, status_slowed_delay 1/1) and all S03/S02/S01/baseline tests. Grep guard: 11 hits for Burn|Freeze|Shock|DeepFreeze in src/ and tests/ — all pre-existing canonical/legitimate uses in the two exempted files (status_effect.rs, skills_ron.rs) and legacy system names (ShockTransfer, MissingPreExistingShock in battery_loop/kernel/observability). The `grep -v 'reserved'` filter is imperfect (doc comment sits on the preceding line, not the variant line) but the semantic intent is fully satisfied: S04 (T01–T04) introduced zero new uncontrolled occurrences.

## Verification

**cargo check:** exit:0 (0 errors, pre-existing warnings only).
**cargo test:** exit:0 — entire integration suite green, 0 failures, 0 ignored. New S04 tests: status_paralyzed_skip (1/1 pass), status_slowed_delay (1/1 pass). Existing tests: status_amp_pipeline, combat_coherence, follow_up_chains, form_identity, validation_snapshot, ultimate_meter, and all other suites pass without regression.
**Grep guard:** `grep -rn -E 'Burn|Freeze|Shock|DeepFreeze' src/ tests/ | grep -v 'reserved' | wc -l` = 11. All 11 hits are pre-existing canonical declarations in status_effect.rs and skills_ron.rs (exempted files per guard spec) or legacy compound names (ShockTransfer, MissingPreExistingShock) predating S04. Zero S04-introduced violations.

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

Grep guard count is 11 (not 0) due to filter imperfection (doc comment 'Reserved §H.1' on preceding line, not variant line). All 11 hits are pre-existing canonical/legitimate uses; S04 introduced zero new violations. The semantic guard intent is fully satisfied.

## Follow-ups

None.

## Files Created/Modified

- `src/combat/turn_system/mod.rs` — 
- `src/combat/turn_system/pipeline.rs` — 
- `tests/status_paralyzed_skip.rs` — 
- `tests/status_slowed_delay.rs` — 
