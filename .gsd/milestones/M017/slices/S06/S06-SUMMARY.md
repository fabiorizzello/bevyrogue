---
id: S06
parent: M017
milestone: M017
provides:
  - Canon JSONL event stream verified clean of legacy status names (Freeze/DeepFreeze) and reserved variants (Burn/Shock) when anchored on "kind":"…" payload
  - ValidationSnapshot exposes per-unit statuses field for deterministic snapshot diffing
  - Integration test fixture proving observability surfaces emit §H.1 canon vocabulary end-to-end
requires:
  []
affects:
  []
key_files:
  - (none)
key_decisions:
  - Sort ValidationStatusSnapshot by explicit canonical ordinal rather than repr(u8) cast — enum lacks #[repr(u8)], manual match is safer and future-proof for reserved variants
  - Anchor legacy/reserved leak check on "kind":"Burn" not bare "Burn" — Heated DoT emits damage_tag:"Fire" which would false-positive on a naive substring search
  - Drain Messages<CombatEvent> cursor every frame inside the drive loop — Bevy 0.18 double-buffer clears oldest events each update(); batch drain after all frames loses early-frame events
  - Capture ValidationSnapshot between round 1 and round 2 so statuses are still active (dur=1) and the per-unit assertion is non-trivial
  - Additive formatter token: existing exact-match tests updated with statuses=[] rather than converting to contains-only checks, preserving full contract coverage
patterns_established:
  - JSONL canon-leak guard pattern: assert "kind":"<Name>" substring presence per canon status + zero matches for "kind":"<Legacy>" — anchor on the payload key to avoid false positives from other fields like damage_tag
  - Per-frame event drain pattern for Bevy 0.18 integration tests: initialize cursor with get_cursor_current() before the drive loop, drain each frame, accumulate into Vec<String>
  - Deterministic ValidationSnapshot diffing: statuses sorted by canonical ordinal, captured at a known point in the scenario where durations are predictable
observability_surfaces:
  - ValidationSnapshot.statuses_per_unit: Vec<ValidationStatusSnapshot> sorted by canonical ordinal, printed as statuses=[Kind(dur),...] token in format_validation_snapshot output
  - JSONL CombatEvent stream: OnStatusApplied, OnStatusTick, OnStatusExpired events carry StatusEffectKind serialized as bare canonical variant name (Heated/Chilled/Paralyzed/Slowed/Blessed)
drill_down_paths:
  - .gsd/milestones/M017/slices/S06/tasks/T01-SUMMARY.md
  - .gsd/milestones/M017/slices/S06/tasks/T02-SUMMARY.md
duration: ""
verification_result: passed
completed_at: 2026-05-13T11:05:44.584Z
blocker_discovered: false
---

# S06: Observability — canon JSONL log + ValidationSnapshot

**Added per-unit statuses field to ValidationSnapshot and verified the JSONL event stream emits only §H.1 canon status names with zero legacy/reserved leakage.**

## What Happened

S06 delivered the observability closure for M017's §H.1 status taxonomy by adding structured per-unit status data to ValidationSnapshot and proving the JSONL event stream is clean of legacy vocabulary.

**T01** extended `src/combat/observability.rs` with a new `ValidationStatusSnapshot { kind: StatusEffectKind, duration_remaining: u32 }` type and a `pub statuses: Vec<ValidationStatusSnapshot>` field on `ValidationUnitSnapshot`. The units query was updated to read `Option<&StatusBag>` per unit; statuses are sorted deterministically by a hand-written canonical ordinal (safer than `repr(u8)` cast since the enum lacks `#[repr(u8)]`). The `format_validation_snapshot` formatter was extended to print a `statuses=[Heated(2),Chilled(1),...]` token per unit (additive, substring-tolerant). The existing `tests/validation_snapshot.rs` fixture was extended with a deterministic scenario covering all 5 active canon kinds with known durations; all 6 fixture tests pass.

**T02** added `tests/status_observability_canon.rs`: a minimal headless Bevy App spawning 5 distinct units, one per canon status (Heated/Chilled/Paralyzed/Slowed/Blessed), with `CombatRng::from_seed(0)` for determinism. The test drives `advance_turn_system` + `resolve_action_system` + `apply_turn_advance_system` for multiple cycles, draining `Messages<CombatEvent>` into a `Vec<String>` via `serde_json::to_string` each frame (not batched after all frames to avoid Bevy 0.18 double-buffer loss). Assertions verify: (a) each of the 5 canon kind names appears as `"kind":"<Name>"` substring; (b) zero matches for `"kind":"Freeze"`, `"kind":"DeepFreeze"`, `"kind":"Burn"`, `"kind":"Shock"` — anchored on the `"kind":"…"` payload to avoid false-positives from `damage_tag:"Fire"` emitted by Heated DoT; (c) ValidationSnapshot captured between rounds shows per-unit statuses matching a hand-rolled expected vector.

Key insight: legacy/reserved check must anchor on `"kind":"Burn"` not bare `"Burn"` — `damage_tag:"Fire"` from Heated DoT would produce a false positive on a naive string search. The cursor must be initialized with `get_cursor_current()` and drained each frame — Bevy 0.18 double-buffer clears oldest events on each `update()`.

## Verification

cargo check: clean (exit 0, warnings only — no errors). cargo test --test validation_snapshot: 6/6 passed. cargo test --test status_observability_canon: 1/1 passed. cargo test (full suite): all tests passed, 0 failed — no regressions on S01–S05 tests.

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

None.

## Follow-ups

None.

## Files Created/Modified

- `src/combat/observability.rs` — Added ValidationStatusSnapshot type; extended ValidationUnitSnapshot with statuses field; updated units query to read StatusBag; updated format_validation_snapshot to print statuses token
- `tests/validation_snapshot.rs` — Extended fixture with deterministic 5-canon-status scenario; updated existing exact-match tests to include statuses=[] token
- `tests/status_observability_canon.rs` — New integration test: 5 canon statuses on distinct units, JSONL stream assertions for canon presence and zero legacy/reserved leakage, ValidationSnapshot per-unit statuses assertion
