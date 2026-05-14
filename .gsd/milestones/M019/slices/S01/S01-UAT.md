# S01: DR pipeline — BuffKind::DR multiplicative damage reduction primitive — UAT

**Milestone:** M019
**Written:** 2026-05-14T08:28:14.395Z

# S01: DR pipeline — UAT

**Milestone:** M019
**Written:** 2026-05-14

## UAT Type

- UAT mode: artifact-driven
- Why this mode is sufficient: All observable behaviour is exercised by deterministic integration tests that cover every stated success criterion. No runtime server or human interaction is needed for a pure combat-formula primitive.

## Preconditions

Rust toolchain present. `cargo test` succeeds from `/home/fabio/dev/bevyrogue`.

## Smoke Test

```
cargo test --test dr_pipeline
```
Expected: `test result: ok. 6 passed; 0 failed`.

## Test Cases

### 1. Single DR reduces damage proportionally

```
cargo test --test dr_pipeline dr_single_30pct_reduces_damage
```
1. A defender with one `DrEntry { amount: 0.30 }` in their `DrBag` is hit.
2. **Expected:** Final HP damage = raw_damage × 0.70 (rounded). Test asserts the exact value.

### 2. Stacked DR sums unclamped

```
cargo test --test dr_pipeline dr_stacked_sums_unclamped
```
1. Two DR entries (0.30 + 0.40 = 0.70 total) on the defender.
2. **Expected:** Final damage = raw × 0.30. No error if sum exceeds 1.0.

### 3. DR combined with type resistance stacks multiplicatively

```
cargo test --test dr_pipeline dr_combined_with_resist_stacks_multiplicatively
```
1. Defender has 30% DR and a type-resistance multiplier applied.
2. **Expected:** Damage = raw × resist_factor × 0.70 (multiplicative stack confirmed).

### 4. DR applies when toughness already broken

```
cargo test --test dr_pipeline dr_applies_when_toughness_already_broken
```
1. Set `toughness.broken = true` before the hit. Defender has 30% DR.
2. **Expected:** HP damage is still DR-reduced regardless of Break state.

### 5. 100% DR clamps damage to 0 and emits CombatEvent::Damage

```
cargo test --test dr_pipeline dr_100pct_clamps_to_zero_and_event_emitted
```
1. Defender has `DrEntry { amount: 1.0 }`.
2. **Expected:** Final damage = 0. `CombatEvent::Damage` is still emitted with `amount = 0`.

### 6. >100% DR produces no panic; damage is 0

```
cargo test --test dr_pipeline dr_over_100pct_no_panic_damage_zero
```
1. Defender has combined DR > 1.0 (e.g. 0.80 + 0.60 = 1.40).
2. **Expected:** No panic. Final damage clamped to 0.

## Edge Cases

### DR entries expire per-turn via tick_all

DR instances have a duration; `advance_turn_system` calls `DrBag::tick_all` each turn alongside `StatusBag::tick_all`. Entries with duration = 1 are removed after one turn. (Not a dedicated test in this slice — covered by T01 unit tests in lib.)

### Zero DR entry (no-op)

A `DrEntry { amount: 0.0 }` should not alter damage. Covered implicitly by baseline damage assertions in the matrix tests.

## Failure Signals

- Any `dr_pipeline` test failing would indicate a regression in the DR formula or DrBag wiring.
- A panic in the `>100%` test would indicate the clamp was removed.
- `CombatEvent::Damage` absent in the 100% test would indicate the event bus short-circuits on zero damage (regression).

## Not Proven By This UAT

- RON `Effect::DR` variant — not introduced in this slice; deferred to a later milestone.
- DrBag expiry events (`OnDrExpired` or equivalent) — deferred; no event is emitted when entries tick to zero in this slice.
- DR interaction with multi-hit skills or follow-up chains — not explicitly tested here; follow-up wiring correctness is covered by existing follow_up test files.
- Live runtime / UI rendering of DR values — headless only.

## Notes for Tester

The `apply_effects` direct-call pattern used in these tests bypasses Bevy's system scheduler. If the Bevy ECS wiring ever changes (e.g. DrBag moved to a different storage), the lib unit tests in `combat::buffs::tests` and pipeline integration tests would surface regressions before dr_pipeline.rs does.
