---
id: T01
parent: S01
milestone: M020
key_files:
  - src/combat/events.rs
  - src/combat/turn_system/pipeline.rs
  - tests/ultimate_event.rs
key_decisions:
  - Emit UltimateUsed in all 4 hoist blocks (not just single-target) for symmetry with UltGain pattern — all paths that do UltEffect::Reset now emit the event once per cast
  - Used replace_all=true for single-target and Blast/AllEnemies blocks since they had identical surrounding code; AllAllies and PerHop were distinct enough for surgical edits
duration: 
verification_result: passed
completed_at: 2026-05-14T10:23:52.553Z
blocker_discovered: false
---

# T01: Added CombatEventKind::UltimateUsed { unit_id } to events.rs and emitted it once per cast in all 4 pipeline hoist blocks; 3 integration tests pass.

**Added CombatEventKind::UltimateUsed { unit_id } to events.rs and emitted it once per cast in all 4 pipeline hoist blocks; 3 integration tests pass.**

## What Happened

Added the `UltimateUsed { unit_id: UnitId }` variant to `CombatEventKind` in `src/combat/events.rs`, adjacent to `UltGain`. Added emit calls in all 4 resource-hoist blocks of `pipeline.rs`, gated by `matches!(inflight.action.ult_effect, UltEffect::Reset)`: (1) single-target path (~line 640), (2) Blast/AllEnemies path (~line 1165) — both replaced via `replace_all` since they shared identical surrounding code, (3) AllAllies/self-target path (~line 1453), (4) PerHop path (~line 1898). The emit mirrors the `UltGain` pattern: source=attacker_id, target=attacker_id. Wrote `tests/ultimate_event.rs` using the `MessageCursor`/`Messages` API pattern (same as `target_shape_aoe_all_order.rs`) with 3 tests: one asserts exactly one `UltimateUsed` event with correct `unit_id` on Ultimate cast with full meter, two negative tests confirm no `UltimateUsed` on Basic and Skill intents.

## Verification

Ran `cargo test --test ultimate_event` (3/3 pass), `cargo check` (clean, warnings only), `cargo check --features windowed` (clean, warnings only).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test ultimate_event` | 0 | 3 tests passed: ultimate_used_emitted_once_on_ult_cast, no_ultimate_used_on_basic_attack, no_ultimate_used_on_skill_cast | 4200ms |
| 2 | `cargo check` | 0 | pass (warnings only, no errors) | 3450ms |
| 3 | `cargo check --features windowed` | 0 | pass (warnings only, no errors) | 5040ms |

## Deviations

none

## Known Issues

None.

## Files Created/Modified

- `src/combat/events.rs`
- `src/combat/turn_system/pipeline.rs`
- `tests/ultimate_event.rs`
