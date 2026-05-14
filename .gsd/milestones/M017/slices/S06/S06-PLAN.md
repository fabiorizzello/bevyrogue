# S06: Observability — canon JSONL log + ValidationSnapshot

**Goal:** Verify the JSONL event stream and ValidationSnapshot expose the §H.1 canon status vocabulary (Heated/Chilled/Paralyzed/Slowed/Blessed) with zero leakage of the legacy names (Freeze/DeepFreeze) and zero runtime emission of reserved variants (Burn/Shock). Adds per-unit `statuses` field on ValidationSnapshot for deterministic snapshot diffing.
**Demo:** Scripted scenario CLI: applica Heated + Chilled + Paralyzed + Slowed + Blessed su units diversi → JSONL log analizzato via grep test, zero match su vocabolario legacy. ValidationSnapshot.statuses_per_unit deterministico in test fixture.

## Must-Haves

- `cargo check` clean.
- `cargo test --test validation_snapshot` green with the extended fixture asserting per-unit `statuses` ordering deterministic.
- New `cargo test --test status_observability_canon` green: 5 canon statuses applied on distinct units; serialized `CombatEvent` stream contains each canon `"kind":"<Name>"` at least once AND zero substring matches for `"kind":"Freeze"`, `"kind":"DeepFreeze"`, `"kind":"Burn"`, `"kind":"Shock"`.
- Full `cargo test` suite green — no regressions on S01–S05 tests.
- ValidationSnapshot exposes a per-unit `statuses: Vec<ValidationStatusSnapshot>` sorted deterministically by kind discriminant.

## Proof Level

- This slice proves: integration tests (in-process Bevy app, no subprocess); deterministic via StatusBag::apply direct seeding and CombatRng::from_seed where any roll is involved.

## Integration Closure

No new CombatEvent variants. No RON schema changes. No M020 source-attribution work. ValidationSnapshot is the snapshot surface; `combat_cli` consumes `format_validation_snapshot` output (substring-tolerant assertions only, confirmed safe).

## Verification

- Adds `ValidationStatusSnapshot { kind, duration_remaining }` and `ValidationUnitSnapshot.statuses` field. Updates `format_validation_snapshot` to print a `statuses=[…]` token per unit (additive, substring-tolerant — CLI surface tests use assert_contains/not_contains). JSONL log unchanged — already canon by construction (Serialize derive emits bare canonical variant names).

## Tasks

- [x] **T01: Add per-unit statuses field to ValidationSnapshot + extend formatter + fixture test** `est:M`
  Seam A from S06-RESEARCH. Add a `ValidationStatusSnapshot { kind: StatusEffectKind, duration_remaining: u32 }` type to `src/combat/observability.rs`. Extend `ValidationUnitSnapshot` (around line 120-132) with a `pub statuses: Vec<ValidationStatusSnapshot>` field. Extend the units query at observability.rs:220 with `Option<&StatusBag>`; for each unit, map `StatusBag` instances into `ValidationStatusSnapshot` and sort deterministically by `StatusEffectKind` discriminant (use `kind as u8` or a manual ordering by canonical name). Update `format_validation_snapshot` (around line 310+) to append a per-unit `statuses=[Heated(2),Chilled(1),...]` token (additive, substring-tolerant). Extend the existing `tests/validation_snapshot.rs` fixture: spawn a unit with `StatusBag` pre-loaded via `StatusBag::apply` for all 5 active canon kinds with known durations; assert the snapshot's per-unit `statuses` vector matches a hand-rolled expected vector (sorted, deterministic).
  - Files: `src/combat/observability.rs`, `tests/validation_snapshot.rs`
  - Verify: cargo check && cargo test --test validation_snapshot && cargo test

- [x] **T02: Scripted scenario integration test: 5 canon statuses on JSONL stream, zero legacy/reserved leakage** `est:M`
  Seam B from S06-RESEARCH. New file `tests/status_observability_canon.rs`. Build a minimal headless Bevy App (mirror harness pattern of `tests/status_paralyzed_skip.rs` / `tests/status_slowed_delay.rs`); critically register `add_message::<ActionValueUpdated>()` and seed `CombatRng::from_seed(0)` to satisfy headless-determinism. Spawn 5 distinct units, apply one canon status each via `StatusBag::apply` directly (Heated/Chilled/Paralyzed/Slowed/Blessed) with known durations. Drive `advance_turn_system` + `resolve_action_system` + `apply_turn_advance_system` for enough cycles to emit OnStatusApplied (already at apply-time), OnStatusTick (each turn), Heated DoT OnDamageDealt{damage_tag:Fire,amount:4}, Slowed first-apply TurnAdvance{amount_pct:-30}, Paralyzed OnActionFailed{reason:"paralyzed"}, and at least one OnStatusExpired. Use `events.get_cursor_current()` (NOT `get_cursor()`) to initialize the MessageCursor before draining the CombatEvent stream into a `Vec<String>` via `serde_json::to_string(&ev).unwrap()`. Assertions: (a) each of the 5 canon kind names appears at least once as substring `"kind":"<Name>"` in the joined stream; (b) zero substring matches for `"kind":"Freeze"`, `"kind":"DeepFreeze"`, `"kind":"Burn"`, `"kind":"Shock"` — anchor on `"kind":"…"` payload (NOT raw `Fire`/`Ice` substring — `damage_tag:"Fire"` is a legitimate false-positive trap); (c) capture a ValidationSnapshot at scenario end and assert per-unit `statuses` matches the expected hand-rolled vector for each of the 5 units.
  - Files: `tests/status_observability_canon.rs`
  - Verify: cargo check && cargo test --test status_observability_canon && cargo test

## Files Likely Touched

- src/combat/observability.rs
- tests/validation_snapshot.rs
- tests/status_observability_canon.rs
