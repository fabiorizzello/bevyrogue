---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T02: Scripted scenario integration test: 5 canon statuses on JSONL stream, zero legacy/reserved leakage

Seam B from S06-RESEARCH. New file `tests/status_observability_canon.rs`. Build a minimal headless Bevy App (mirror harness pattern of `tests/status_paralyzed_skip.rs` / `tests/status_slowed_delay.rs`); critically register `add_message::<ActionValueUpdated>()` and seed `CombatRng::from_seed(0)` to satisfy headless-determinism. Spawn 5 distinct units, apply one canon status each via `StatusBag::apply` directly (Heated/Chilled/Paralyzed/Slowed/Blessed) with known durations. Drive `advance_turn_system` + `resolve_action_system` + `apply_turn_advance_system` for enough cycles to emit OnStatusApplied (already at apply-time), OnStatusTick (each turn), Heated DoT OnDamageDealt{damage_tag:Fire,amount:4}, Slowed first-apply TurnAdvance{amount_pct:-30}, Paralyzed OnActionFailed{reason:"paralyzed"}, and at least one OnStatusExpired. Use `events.get_cursor_current()` (NOT `get_cursor()`) to initialize the MessageCursor before draining the CombatEvent stream into a `Vec<String>` via `serde_json::to_string(&ev).unwrap()`. Assertions: (a) each of the 5 canon kind names appears at least once as substring `"kind":"<Name>"` in the joined stream; (b) zero substring matches for `"kind":"Freeze"`, `"kind":"DeepFreeze"`, `"kind":"Burn"`, `"kind":"Shock"` — anchor on `"kind":"…"` payload (NOT raw `Fire`/`Ice` substring — `damage_tag:"Fire"` is a legitimate false-positive trap); (c) capture a ValidationSnapshot at scenario end and assert per-unit `statuses` matches the expected hand-rolled vector for each of the 5 units.

## Inputs

- `.gsd/milestones/M017/slices/S06/S06-RESEARCH.md`
- `src/combat/status_effect.rs`
- `src/combat/events.rs`
- `src/combat/jsonl_logger.rs`
- `src/combat/observability.rs`
- `src/combat/turn_system/mod.rs`
- `src/combat/turn_system/pipeline.rs`
- `tests/status_paralyzed_skip.rs`
- `tests/status_slowed_delay.rs`
- `tests/status_amp_pipeline.rs`
- `.gsd/milestones/M017/slices/S04/S04-SUMMARY.md`

## Expected Output

- `tests/status_observability_canon.rs`

## Verification

cargo check && cargo test --test status_observability_canon && cargo test

## Observability Impact

Regression guard on JSONL event stream — asserts canon vocabulary only, no legacy/reserved variant leakage. Anchored on `"kind":"…"` payload to avoid `damage_tag:"Fire"` false-positives.
