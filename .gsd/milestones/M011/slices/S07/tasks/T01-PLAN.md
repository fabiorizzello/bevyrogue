---
estimated_steps: 1
estimated_files: 4
skills_used: []
---

# T01: Add ToughnessCategory enum + RoundFlags component; extend Toughness::apply_hit with category and seal logic

Introduce the foundational types for the slice. Define a `ToughnessCategory` enum (Standard, Armored, Shielded) in `src/combat/toughness.rs` and add it as a field on the `Toughness` component (default Standard for back-compat with existing constructors). Extend `Toughness::new` with a category parameter and add a convenience `Toughness::with_category(max, weaknesses, category)` constructor; keep the existing `new` signature working by defaulting to Standard inside the body. Modify `Toughness::apply_hit` to accept a `break_sealed: bool` flag and to behave as follows: (a) Shielded NEVER transitions to broken from a `ToughnessHit` and never decrements `current` past 0 (clamp at 0, return false); (b) Armored halves the incoming toughness damage (rounded up: `(amount + 1) / 2`) before applying — so an Armored unit needs roughly 2x the cumulative toughness damage to break; (c) Standard preserves the existing semantics. In all categories, if `break_sealed` is true the function returns false without mutating `current`. Define the `RoundFlags` component in a new file `src/combat/round_flags.rs` with a single `pub break_sealed: bool` field (Default = false), register the module in `src/combat/mod.rs`, and re-export the type. Update the existing in-file unit tests in `toughness.rs` to (i) keep passing for Standard via the default, (ii) add three new unit tests: `shielded_never_breaks_from_toughness_hit`, `armored_requires_double_damage_to_break`, `apply_hit_is_noop_when_break_sealed`. Do NOT yet wire the new component into bootstrap or threadthe new `apply_hit` signature into resolution — those happen in T02. To keep the codebase compiling at the end of T01, update the single call site in `src/combat/resolution.rs::apply_effects` to pass `false` as the new `break_sealed` argument (placeholder; T02 will replace it with the real query).

## Inputs

- `src/combat/toughness.rs`
- `src/combat/mod.rs`
- `src/combat/resolution.rs`

## Expected Output

- `src/combat/toughness.rs`
- `src/combat/round_flags.rs`
- `src/combat/mod.rs`
- `src/combat/resolution.rs`

## Verification

cargo test --lib combat::toughness 2>&1 | grep -E 'test result' && cargo check 2>&1 | tail -5

## Observability Impact

No event-bus changes — adds only component fields. Failure mode if mis-wired: existing toughness unit tests would regress; `cargo test --lib combat::toughness` is the canary.
