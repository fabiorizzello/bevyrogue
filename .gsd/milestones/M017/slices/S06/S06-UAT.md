# S06: Observability — canon JSONL log + ValidationSnapshot — UAT

**Milestone:** M017
**Written:** 2026-05-13T11:05:44.584Z

# UAT: S06 — Observability canon JSONL log + ValidationSnapshot

## UAT Type
Automated integration test (in-process headless Bevy App, deterministic via seeded RNG and fixed durations). No subprocess, no CLI invocation.

## Preconditions
- Clean `cargo check` (no errors)
- All S01–S05 tests passing
- `tests/status_observability_canon.rs` present
- `tests/validation_snapshot.rs` extended with 5-canon-status fixture

## Steps

1. Run `cargo test --test validation_snapshot`
   - **Expected:** 6/6 tests pass. Snapshot fixture for a unit with all 5 canon statuses (Heated(2), Chilled(2), Paralyzed(2), Slowed(2), Blessed(2)) asserts `statuses` vector matches hand-rolled expected in canonical order.

2. Run `cargo test --test status_observability_canon`
   - **Expected:** 1/1 test passes.
   - JSONL stream contains `"kind":"Heated"`, `"kind":"Chilled"`, `"kind":"Paralyzed"`, `"kind":"Slowed"`, `"kind":"Blessed"` at least once each.
   - JSONL stream contains zero matches for `"kind":"Freeze"`, `"kind":"DeepFreeze"`, `"kind":"Burn"`, `"kind":"Shock"`.
   - ValidationSnapshot captured mid-scenario shows per-unit statuses matching expected hand-rolled vectors.

3. Run `cargo test` (full suite)
   - **Expected:** All tests pass, 0 failed. No regressions on S01–S05 tests (status_amp_pipeline, status_paralyzed_skip, status_slowed_delay, status_blessed_*, status_policy_refresh, status_cleanse_policy, combat_coherence, follow_up_chains, form_identity).

## Edge Cases
- **False-positive trap:** `damage_tag:"Fire"` is emitted by Heated DoT. Legacy leak assertions are anchored on `"kind":"Burn"` not bare `"Burn"` — this is verified by the test passing while Heated DoT events are present in the stream.
- **Event loss trap:** Bevy 0.18 double-buffer: cursor initialized with `get_cursor_current()` and drained each frame. Test verifies events from all frames (including frame 1 where statuses are applied) are captured.
- **Snapshot timing:** ValidationSnapshot captured between round 1 and round 2 — statuses have dur=1 remaining, making the per-unit assertion non-trivial (not empty).

## Not Proven By This UAT
- Subprocess JSONL file output format (only in-process serialization tested)
- Status vocabulary in windowed UI / egui rendering
- JSONL log rotation or persistence across sessions
- Stack-aware status (Heated × N) — deferred to §H.5 post-M017

