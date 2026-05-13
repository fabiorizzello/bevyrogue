---
estimated_steps: 6
estimated_files: 3
skills_used: []
---

# T03: Widen the three validation gates to accept Blast and AllEnemies

Three sites currently gate non-Single shapes behind UnimplementedTargetShape. Widen each consistently so Blast and AllEnemies pass; Row and SelfOnly remain deferred.

1. `src/data/skills_ron.rs:277-289` — validate_skill_def allowlist. Add Blast and AllEnemies to the matches!() expression that lets Implemented skills past the shape check. Verify the secondary Damage{target,..} vs targeting.shape consistency check at lines 291-330 still passes when shape == AllEnemies or Blast.
2. `src/combat/resolution.rs:185-194` — target_shape_is_executable_now / target_shape_rejection_reason. Extend the matches!() allowlist to Blast and AllEnemies.
3. `src/combat/action_query.rs:485-489` — target_status_for_* deferral gate. Allow Blast and AllEnemies past the gate.

Must-haves: all three gates use the SAME allowlist set (Single, Blast, AllEnemies). Greppable invariant — running `rg -n 'TargetShape::(Blast|AllEnemies)' src/` lists all three sites.

No behavior change for existing skills — gates only widen. All 554 baseline tests must remain green.

## Inputs

- `src/data/skills_ron.rs`
- `src/combat/resolution.rs`
- `src/combat/action_query.rs`

## Expected Output

- `src/data/skills_ron.rs`
- `src/combat/resolution.rs`
- `src/combat/action_query.rs`

## Verification

cargo test 2>&1 | tail -20 | grep -E '(test result|FAILED)' && cargo check --features windowed 2>&1 | tail -5
