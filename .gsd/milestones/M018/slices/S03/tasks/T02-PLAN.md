---
estimated_steps: 5
estimated_files: 3
skills_used: []
---

# T02: Widen three validation gates to accept Bounce(_)

Update the three allowlist sites that previously gated non-Single shapes behind UnimplementedTargetShape to accept Bounce(_):
1. `src/data/skills_ron.rs:~282` in `validate_skill_def` — change the match `TargetShape::Single | TargetShape::Blast | TargetShape::AllEnemies` to `... | TargetShape::Bounce(_)`. Also enforce `N >= 1` in the same block: if shape is Bounce(0), return `UnimplementedTargetShape` ("Bounce(0) has no hops"). Row and SelfOnly remain deferred.
2. `src/combat/resolution.rs:241-243` in `target_shape_is_executable_now` — extend the allowlist with `TargetShape::Bounce(_)`.
3. `src/combat/action_query.rs:485-492` in `target_status_for_unit` — extend the same allowlist.
Update any test asserting on the rejected-shape error message (Row remains the canonical reject case in `validate_rejects_implemented_non_single_shape`). Add a positive unit test confirming Bounce(3) validates and Bounce(0) is rejected.

## Inputs

- ``src/data/skills_ron.rs` — validate_skill_def allowlist line ~282 (verified)`
- ``src/combat/resolution.rs` — target_shape_is_executable_now line 241 (verified)`
- ``src/combat/action_query.rs` — target_status_for_unit allowlist line 485 (verified)`
- ``src/data/skills_ron.rs` (T01 output) — must contain Bounce(u8) variant`

## Expected Output

- ``src/data/skills_ron.rs` — validate_skill_def allowlist includes Bounce(_); Bounce(0) rejected with UnimplementedTargetShape`
- ``src/combat/resolution.rs` — target_shape_is_executable_now allowlist includes Bounce(_)`
- ``src/combat/action_query.rs` — target_status_for_unit allowlist includes Bounce(_)`

## Verification

cargo test --lib skills_ron::tests && cargo test --lib resolution::tests && cargo check
