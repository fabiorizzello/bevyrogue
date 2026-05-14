# S01: Nuovi eventi reactive bus: UltimateUsed + UnitDied payload — UAT

**Milestone:** M020
**Written:** 2026-05-14T10:38:51.770Z

# S01: Nuovi eventi reactive bus: UltimateUsed + UnitDied payload — UAT

**Milestone:** M020
**Written:** 2026-05-14

## UAT Type

- UAT mode: artifact-driven
- Why this mode is sufficient: All deliverables are headless integration tests; the combat engine runs deterministically without a UI. The test suite is the authoritative contract oracle for bus event correctness.

## Preconditions

- Rust toolchain installed (see `rust-toolchain.toml`)
- No uncommitted source modifications beyond those in this slice

## Smoke Test

```bash
cargo test --test ultimate_event --test unit_died_payload
```
Expected: 5 tests pass (3 in ultimate_event, 2 in unit_died_payload), 0 failures.

## Test Cases

### 1. UltimateUsed emitted exactly once per ultimate cast

1. Run `cargo test --test ultimate_event`
2. **Expected:** 3 tests pass — `ultimate_used_emitted_on_cast`, `ultimate_used_not_emitted_on_basic`, and any non-Reset skill variant test. Zero failures.

### 2. UltimateUsed carries correct unit_id

1. Inspect `tests/ultimate_event.rs` — the test drives `ActionIntent::Ultimate` with `ult.current == max` and asserts the emitted event has `unit_id == attacker_id`.
2. Run `cargo test --test ultimate_event 2>&1 | grep ok`
3. **Expected:** All tests pass; no assertion about wrong `unit_id`.

### 3. UnitDied carries StatusBag snapshot

1. Run `cargo test --test unit_died_payload`
2. **Expected:** `unit_died_carries_defender_status_snapshot` passes — asserts `status_remaining` contains `Heated` and `Slowed`, and `heated_remaining == 2`.

### 4. UnitDied not emitted on survival

1. Run `cargo test --test unit_died_payload`
2. **Expected:** `unit_died_not_emitted_on_survival` passes — confirms no `UnitDied` event when the defender survives the hit.

### 5. Full regression suite clean

1. Run `cargo test`
2. **Expected:** All 673 tests pass, zero failures across all integration test files.

### 6. No residual OnKO references

1. Run `rg -n 'CombatEventKind::OnKO' src tests`
2. **Expected:** No output (exit code 1 = zero matches). The rename is complete.

## Edge Cases

### Stun-damage KO path emits empty payload

1. Review `src/combat/turn_system/mod.rs` for the stun-damage KO emit site.
2. **Expected:** `UnitDied` is emitted with `status_remaining: vec![]` and `heated_remaining: 0`; a one-line comment documents the limitation. Tests in `combat_coherence.rs` and related files accept this form.

### Basic action and non-Reset skills do not emit UltimateUsed

1. Run `cargo test --test ultimate_event 2>&1 | grep -E "not_emitted"`
2. **Expected:** Tests confirming no `UltimateUsed` on Basic/non-Reset pass green.

## Failure Signals

- Any `FAILED` line in `cargo test` output
- `rg -n 'CombatEventKind::OnKO' src tests` returning matches
- `cargo check` or `cargo check --features windowed` returning errors (not warnings)

## Not Proven By This UAT

- JSONL logger actually serializes the new fields to disk at runtime (tested structurally; no file I/O checked here)
- The stun-damage KO path ever has a `StatusBag` in scope — the empty payload is a known limitation, not tested for correctness of a future fix
- Live windowed UI rendering of new event variants — headless only

## Notes for Tester

The `mod.rs` stun-damage KO path intentionally emits `UnitDied` with an empty payload. This is documented with a comment and is a known limitation, not a bug. Downstream S02 (shim removal) has no dependency on this payload.
