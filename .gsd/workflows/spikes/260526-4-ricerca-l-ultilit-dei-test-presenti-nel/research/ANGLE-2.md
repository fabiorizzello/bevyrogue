# ANGLE 2 — Impl-coupled internal tests

**Confidence: high** for the relocated unit tests, **medium-high** for the golden
snapshots (read `validation_snapshot.rs` in full; sampled the internals set).

## The `*_internals.rs` relocated unit tests (23 files)

These were moved out of `src/` per R003 (no inline `mod tests`). They test pure
helpers: fold-of-modifiers math, SP/energy arithmetic, parsers, status-bag
ordering, toughness/resistance, registry lookup, event filtering.

**Verdict: KEEP, broadly.** They are *behavior*-anchored (input → output of a
pure function), not *structure*-anchored. They are cheap, fast, deterministic,
and they don't break on rename of unrelated code — they break only when the math
they pin changes, which is exactly when a test *should* break. The R003
relocation made their import paths a little verbose, but that is a path cost, not
a churn cost. Cutting these would remove coverage, not noise. The user's target
("tests that change too often because they test impl") does **not** describe
these — they change rarely.

Caveat: a few may *duplicate* integration coverage (e.g. SP gain is also asserted
end-to-end in turn_economy). That's a redundancy question, not an impl-coupling
question, and pruning pure-function unit tests to lean on slower integration
tests is usually the wrong trade. Leave them.

## The `validation_snapshot` golden strings (4 `assert_eq!` on full strings)

`format_validation_snapshot` produces a single deterministic line that humans and
agents read as the validation surface (it IS the observable contract — D026/P005
lineage). Four tests pin the *entire* formatted string:
`snapshot_contract_covers_promised_fields_and_shape`,
`snapshot_defaults_empty_optional_surfaces`,
`snapshot_hides_ally_missing_toughness_and_zero_max_enemy_bars`, plus the
status-ordering one.

**Verdict: KEEP, but acknowledge brittleness.** These are *not* impl-coupling in
the bad sense — they pin an *output format that is itself the contract*. A golden
string on a stable, intentional text format is legitimate; the format is the
product. The cost: any field-order/label change re-bakes all four at once. That
is acceptable friction because such a change is a deliberate contract change, not
an incidental refactor. The runtime tests in the same file
(`runtime_registration_applies_all_kernel_transition_domains`,
`runtime_registration_populates_snapshot_kernel_resources`) are behavior-anchored
— unambiguous keep.

This file is the closest thing to a borderline case, but it sits on the "keep"
side: a golden on a deliberate, human-read format ≠ a test that knows where types
live.

## Verdict

No cuts in Angle 2. The relocated unit tests are stable behavior tests; the
golden snapshots pin an intentional contract. Neither matches "churns on
refactor without buying confidence."
