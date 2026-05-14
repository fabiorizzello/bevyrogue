---
id: T02
parent: S01
milestone: M020
key_files:
  - src/combat/events.rs
  - src/combat/resolution.rs
  - src/combat/turn_system/pipeline.rs
  - src/combat/turn_system/mod.rs
  - tests/unit_died_payload.rs
  - tests/combat_coherence.rs
  - tests/follow_up_triggers.rs
  - tests/event_stream.rs
  - tests/pipeline_dispatch.rs
  - tests/toughness_enemy_only.rs
key_decisions:
  - Added ko_payload() helper in resolution.rs to centralize StatusBag snapshot extraction rather than duplicating inline at both emission sites
  - mod.rs stun-damage site emits empty payload (no StatusBag in scope) with a comment documenting the limitation
  - pipeline.rs match arms use UnitDied { .. } wildcard pattern — payload is not consumed there, behavior unchanged
duration: 
verification_result: passed
completed_at: 2026-05-14T10:29:50.156Z
blocker_discovered: false
---

# T02: Renamed OnKO → UnitDied { status_remaining, heated_remaining } with full payload filled from defender's StatusBag at KO time

**Renamed OnKO → UnitDied { status_remaining, heated_remaining } with full payload filled from defender's StatusBag at KO time**

## What Happened

Renamed the `OnKO` variant in `src/combat/events.rs` to `UnitDied { status_remaining: Vec<StatusEffectKind>, heated_remaining: u32 }`. Added a private `ko_payload(bag: Option<&StatusBag>)` helper in `resolution.rs` that extracts the snapshot fields: `status_remaining` via `bag.iter().map(|inst| inst.kind.clone()).collect()` and `heated_remaining` via `bag.get_dur(&StatusEffectKind::Heated).unwrap_or(0)`. Updated both KO emission sites in `apply_damage_only` (single-target path ~560 and multi-hop path ~780) to call `ko_payload(defender_status)` — `defender_status: Option<&StatusBag>` was already in scope. Updated the stun-damage KO site in `turn_system/mod.rs:488` where no StatusBag is in scope, emitting `status_remaining: vec![], heated_remaining: 0` with a one-line comment. Updated all 4 `OnKO =>` match arms in `pipeline.rs` to `UnitDied { .. } =>` using replace_all (behavior unchanged). Updated inline self-tests in `resolution.rs` (2 sites) and integration tests in `event_stream.rs`, `toughness_enemy_only.rs`, `pipeline_dispatch.rs`, `combat_coherence.rs`, and `follow_up_triggers.rs`. Created `tests/unit_died_payload.rs` with two tests: one asserting that a lethal hit on a defender with Heated(2)+Slowed(1) produces `UnitDied` carrying both kinds and `heated_remaining==2`, and one asserting `UnitDied` is not emitted on a non-lethal hit.

## Verification

cargo test — all tests pass (including 2 new tests in unit_died_payload.rs). cargo check --features windowed — clean compile. rg -n 'CombatEventKind::OnKO' src tests — no matches.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test` | 0 | pass | 45000ms |
| 2 | `cargo check --features windowed` | 0 | pass | 12000ms |
| 3 | `! rg -n 'CombatEventKind::OnKO' src tests` | 0 | pass | 200ms |

## Deviations

none

## Known Issues

None.

## Files Created/Modified

- `src/combat/events.rs`
- `src/combat/resolution.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/turn_system/mod.rs`
- `tests/unit_died_payload.rs`
- `tests/combat_coherence.rs`
- `tests/follow_up_triggers.rs`
- `tests/event_stream.rs`
- `tests/pipeline_dispatch.rs`
- `tests/toughness_enemy_only.rs`
