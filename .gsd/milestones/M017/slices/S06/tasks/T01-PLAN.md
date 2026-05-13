---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T01: Add per-unit statuses field to ValidationSnapshot + extend formatter + fixture test

Seam A from S06-RESEARCH. Add a `ValidationStatusSnapshot { kind: StatusEffectKind, duration_remaining: u32 }` type to `src/combat/observability.rs`. Extend `ValidationUnitSnapshot` (around line 120-132) with a `pub statuses: Vec<ValidationStatusSnapshot>` field. Extend the units query at observability.rs:220 with `Option<&StatusBag>`; for each unit, map `StatusBag` instances into `ValidationStatusSnapshot` and sort deterministically by `StatusEffectKind` discriminant (use `kind as u8` or a manual ordering by canonical name). Update `format_validation_snapshot` (around line 310+) to append a per-unit `statuses=[Heated(2),Chilled(1),...]` token (additive, substring-tolerant). Extend the existing `tests/validation_snapshot.rs` fixture: spawn a unit with `StatusBag` pre-loaded via `StatusBag::apply` for all 5 active canon kinds with known durations; assert the snapshot's per-unit `statuses` vector matches a hand-rolled expected vector (sorted, deterministic).

## Inputs

- `.gsd/milestones/M017/slices/S06/S06-RESEARCH.md`
- `src/combat/observability.rs`
- `src/combat/status_effect.rs`
- `tests/validation_snapshot.rs`
- `.gsd/milestones/M017/slices/S02/S02-SUMMARY.md`

## Expected Output

- `src/combat/observability.rs`
- `tests/validation_snapshot.rs`

## Verification

cargo check && cargo test --test validation_snapshot && cargo test

## Observability Impact

Adds ValidationStatusSnapshot type + ValidationUnitSnapshot.statuses field; formatter prints additive statuses=[…] token per unit.
