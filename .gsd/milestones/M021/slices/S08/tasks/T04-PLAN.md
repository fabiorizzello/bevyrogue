---
estimated_steps: 6
estimated_files: 3
skills_used: []
---

# T04: Write deterministic end-to-end tests for Twin Core Blueprint path and Bouncing Fire OFF=baseline

**Why:** The twin_core_integration and twin_core_mechanics tests currently pump `CombatKernelTransition::TwinCore(...)` directly. After T01 that variant is gone. T01 already updates imports, but this task ensures the tests exercise the new `Blueprint { owner: \"twin_core\" }` event path end-to-end, and adds the Bouncing Fire deterministic test.

**Do:**
1. Write `tests/bouncing_fire_off_baseline.rs`: set up a minimal combat world with Agumon (baby_flame timeline from skills.ron, TalentRanks default = rank 0). Execute baby_flame through the timeline runner. Capture the Intent stream. Compare against a baseline captured from the same setup without the Loop branch present (or assert specific expected intents: DealDamage(18), BreakToughness(10), BlueprintSignal). Confirm no loop-related intents appear.
2. Write a second test in the same file: set TalentRanks `"agumon::bouncing_fire" = 1`, add 2+ enemy targets. Execute baby_flame. Confirm the baseline intents appear PLUS exactly 1 bounce hop (DealDamage(9) to a second target).
3. Verify existing `tests/twin_core_integration.rs` and `tests/twin_core_mechanics.rs` still pass after T01 changes (they should, since T01 updates their imports). If they fail due to the transition type change, fix them here to pump Blueprint events instead.

**Done when:** `cargo test bouncing_fire` passes; `cargo test twin_core` passes; full `cargo test` green.

## Inputs

- `src/combat/blueprints/agumon/mod.rs`
- `src/combat/blueprints/twin_core/mod.rs`
- `src/combat/api/runner.rs`
- `assets/data/skills.ron`
- `tests/twin_core_integration.rs`
- `tests/twin_core_mechanics.rs`

## Expected Output

- `tests/bouncing_fire_off_baseline.rs`

## Verification

cargo test --test bouncing_fire_off_baseline && cargo test twin_core && cargo test
