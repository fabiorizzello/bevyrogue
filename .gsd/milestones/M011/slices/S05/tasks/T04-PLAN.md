---
estimated_steps: 14
estimated_files: 1
skills_used: []
---

# T04: Add integration test exercising SP cap and Child discount scenario end-to-end

Integration test proving the slice demo: 3 turns of Basic with a Child unit show discount at 3rd skill; SP cap enforced when attempting to exceed +2 non-Basic.

## Steps

1. Create `tests/resource_caps.rs` with two test functions:
   - `child_discount_after_two_basics`: Build a minimal world with a Child unit. Execute 2 Basic actions via ActionIntent. Then execute a Skill — verify SP cost is reduced by 1. Execute another Skill — verify no discount (streak was reset).
   - `sp_non_basic_cap_enforced`: Build a world with SpPool and RoundSpTracker. Attempt 3 non-Basic SP gains of +1 each — verify only 2 are applied. Reset tracker, gain 2 more — verify success.
2. Run `cargo test --test resource_caps` — both tests pass.
3. Run full `cargo test` — all tests pass.

## Must-Haves

- [ ] Integration test for Child discount scenario
- [ ] Integration test for SP non-Basic cap
- [ ] All existing tests still pass

## Verification

- `cargo test --test resource_caps` passes
- `cargo test` passes with zero failures

## Inputs

- ``src/combat/sp.rs` — from T01`
- ``src/combat/resolution.rs` — from T03`
- ``src/combat/unit.rs` — from T03`
- ``src/combat/energy.rs` — from T02`
- ``src/combat/bootstrap.rs` — from T03`

## Expected Output

- ``tests/resource_caps.rs` — integration tests for SP cap and Child discount`

## Verification

cargo test --test resource_caps 2>&1 | tail -5
