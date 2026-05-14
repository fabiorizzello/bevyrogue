---
id: S01
parent: M020
milestone: M020
provides:
  - CombatEventKind::UltimateUsed emitted once per cast
  - CombatEventKind::UnitDied with status_remaining and heated_remaining payload
requires:
  []
affects:
  - S02
key_files:
  - src/combat/events.rs
  - src/combat/turn_system/pipeline.rs
  - src/combat/resolution.rs
  - src/combat/turn_system/mod.rs
  - tests/ultimate_event.rs
  - tests/unit_died_payload.rs
  - tests/combat_coherence.rs
  - tests/follow_up_triggers.rs
  - tests/event_stream.rs
  - tests/pipeline_dispatch.rs
  - tests/toughness_enemy_only.rs
key_decisions:
  - Emit UltimateUsed in all 4 hoist blocks symmetrically with UltGain — every path that does UltEffect::Reset now emits the event once per cast
  - ko_payload() helper in resolution.rs centralizes StatusBag snapshot extraction rather than duplicating inline at both KO emission sites
  - mod.rs stun-damage site emits UnitDied with empty payload (no StatusBag in scope) — documented with a one-line comment
  - pipeline.rs match arms use UnitDied { .. } wildcard — payload not consumed there, behavior unchanged
patterns_established:
  - UltimateUsed follows the same source/target = attacker_id pattern as UltGain for symmetry
  - ko_payload() helper pattern: centralize bag snapshot at emission site rather than threading bag all the way through callers
observability_surfaces:
  - CombatEventKind::UltimateUsed is now a dedicated signal in the JSONL event stream (jsonl_logger.rs) — one entry per ultimate cast, queryable by unit_id
  - CombatEventKind::UnitDied now carries status_remaining and heated_remaining in the JSONL stream — post-mortem state is observable without replaying the full game log
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-14T10:38:51.770Z
blocker_discovered: false
---

# S01: Nuovi eventi reactive bus: UltimateUsed + UnitDied payload

**Added CombatEventKind::UltimateUsed emitted once per cast in all 4 pipeline hoist blocks, and renamed OnKO → UnitDied { status_remaining, heated_remaining } with full StatusBag snapshot at KO time; 673 tests green.**

## What Happened

**T01 — UltimateUsed variant and emit:** Added `CombatEventKind::UltimateUsed { unit_id: UnitId }` to `events.rs` (near `UltGain`). Emitted exactly once per cast inside the four resource-hoist blocks of `pipeline.rs` (single-target ~561, Blast/AllEnemies ~1077, AllAllies ~1357, PerHop ~1861), gated by `matches!(inflight.action.ult_effect, UltEffect::Reset)`. New integration test `tests/ultimate_event.rs` (3 tests) verifies: (a) exactly one `UltimateUsed` event with correct `unit_id` per cast via `ActionIntent::Ultimate`; (b) no `UltimateUsed` on Basic or non-Reset Skill. `cargo check` headless and windowed both clean.

**T02 — OnKO → UnitDied with payload:** Renamed `OnKO` to `UnitDied { status_remaining: Vec<StatusEffectKind>, heated_remaining: u32 }` in `events.rs`. Added `ko_payload()` helper in `resolution.rs` to centralize `StatusBag` snapshot extraction at both `apply_damage_only` KO emission sites (~559 and ~780), avoiding duplication. `mod.rs` stun-damage path (no `StatusBag` in scope) emits with `status_remaining: vec![], heated_remaining: 0` and a one-line comment documenting the gap. All four `pipeline.rs` match arms updated to `UnitDied { .. }` wildcard (behavior unchanged). Updated JSON string expectations in `tests/combat_coherence.rs`, `tests/follow_up_triggers.rs`, `tests/event_stream.rs`, `tests/pipeline_dispatch.rs`, `tests/toughness_enemy_only.rs`. New `tests/unit_died_payload.rs` (2 tests): seeds a defender with `Heated(dur=2)` + `Slowed(dur=1)` in `StatusBag`, inflicts fatal damage via `apply_effects`, and asserts `UnitDied` carries both kinds in `status_remaining` and `heated_remaining == 2`. Zero residual `CombatEventKind::OnKO` occurrences confirmed by `rg`.

## Verification

- `cargo test`: 673 tests pass across all integration test suites, including the 3 new tests in `ultimate_event.rs` and 2 new tests in `unit_died_payload.rs`. Zero failures.
- `cargo check` (headless): Finished cleanly; warnings only (no new warnings introduced).
- `cargo check --features windowed`: Finished cleanly; warnings only (no new warnings introduced).
- `rg -n 'CombatEventKind::OnKO' src tests`: exit 1 — zero matches. Rename is complete throughout the codebase.

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

none

## Known Limitations

The `mod.rs` stun-damage KO path emits `UnitDied` with `status_remaining: vec![], heated_remaining: 0` because no `StatusBag` is in scope at that call site. This is a known gap documented with a comment; downstream listeners relying on the payload for post-KO effects will not receive status snapshot data from stun-triggered kills.

## Follow-ups

none — S02 (shim removal) proceeds as planned

## Files Created/Modified

- `src/combat/events.rs` — Added UltimateUsed { unit_id } variant; renamed OnKO → UnitDied { status_remaining, heated_remaining }
- `src/combat/turn_system/pipeline.rs` — Added UltimateUsed emit in all 4 hoist blocks; updated OnKO match arms to UnitDied { .. }
- `src/combat/resolution.rs` — Added ko_payload() helper; updated both KO emit sites to UnitDied with StatusBag snapshot
- `src/combat/turn_system/mod.rs` — Updated stun-damage KO emit to UnitDied with empty payload + comment
- `tests/ultimate_event.rs` — New: 3 tests verifying UltimateUsed event emission per cast
- `tests/unit_died_payload.rs` — New: 2 tests verifying UnitDied carries status_remaining and heated_remaining
- `tests/combat_coherence.rs` — Updated JSON string expectations from OnKO to UnitDied
- `tests/follow_up_triggers.rs` — Updated JSON string expectations from OnKO to UnitDied
- `tests/event_stream.rs` — Updated JSON string expectations from OnKO to UnitDied
- `tests/pipeline_dispatch.rs` — Updated JSON string expectations from OnKO to UnitDied
- `tests/toughness_enemy_only.rs` — Updated JSON string expectations from OnKO to UnitDied
