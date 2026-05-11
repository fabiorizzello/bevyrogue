---
estimated_steps: 5
estimated_files: 5
skills_used:
  - tdd
  - verify-before-complete
---

# T01: Define the pure action-query vocabulary and DSL health rule

Why: S04 needs a stable machine-readable vocabulary before legality rules can be implemented. This task extends the existing DSL contract without skill-ID tables, adds the pure query module boundary, and creates focused compile-time/fixture coverage that downstream tasks can fill in.

Skills: use `tdd` for the contract-first vertical slice and `verify-before-complete` before marking the task complete.

Do:
1. Extend `LegalityReasonCode` in `src/data/skills_ron.rs` with the missing contract reasons: `NotActiveUnit`, `WrongPhase`, `AttackerKo`, `AttackerStunned`, `MissingSkill`, `SpShortfall`, `UltimateNotReady`, `TargetNotFound`, `TamerGaugeDeferred`, `TamerCommandDeferred`, `ChargedTelegraphDeferred`, `EnemyTraitDeferred`, and `EnergyCapReached`.
2. Add a small DSL field for damaged-target intent, e.g. `TargetHpRule::{Any, Damaged}` on `SkillTargeting`; migrate canonical `assets/data/skills.ron` to declare the default `Any` explicitly so data remains inspectable rather than implicit.
3. Create `src/combat/action_query.rs` with pure structs/enums for `CombatQuerySnapshot`, `UnitQuerySnapshot`, `ActionQueryKind`, `ActionAffordance`, `TargetAffordance`, `ActionStatus`, `TargetStatus`, `ResourceStatus`, `ImplementationStatus`, and a `ToughnessAffordance` shape; derive `Debug`, `Clone`, `PartialEq`, `Eq` where practical.
4. Export the module from `src/combat/mod.rs` and add initial tests in `tests/action_affordance_query.rs` that compile against the public vocabulary and build inline fixture skill/snapshot data.
5. Keep the module headless and pure: no Bevy `World`, no systems, no UI/CLI imports, and no skill-ID-specific legality rules.

Failure Modes (Q5): invalid or incomplete canonical RON should fail existing skill parsing tests; missing fixture fields should be caught at compile time; unsupported query calls should initially return explicit disabled/deferred/hidden statuses rather than panic once later tasks implement behavior.

Load Profile (Q6): shared resources are immutable in-memory snapshots and skill books; per-operation cost should stay linear in units and candidate skills; 10x unit count should increase vector scanning cost, not introduce global state or ECS borrow pressure.

Negative Tests (Q7): include fixture construction for missing skill IDs, damaged vs full HP targets, and unsupported shape declarations so later tasks can assert stable reasons.

## Inputs

- `src/data/skills_ron.rs`
- `assets/data/skills.ron`
- `src/combat/mod.rs`
- `src/combat/types.rs`
- `src/combat/team.rs`
- `src/combat/kit.rs`
- `src/combat/state.rs`
- `src/combat/toughness.rs`

## Expected Output

- `src/data/skills_ron.rs`
- `assets/data/skills.ron`
- `src/combat/action_query.rs`
- `src/combat/mod.rs`
- `tests/action_affordance_query.rs`

## Verification

cargo test-dev --test action_affordance_query && cargo test-dev skills_ron

## Observability Impact

The query vocabulary is the diagnostic surface for later slices: future agents should inspect `tests/action_affordance_query.rs` expected statuses/reasons and the public enums in `src/combat/action_query.rs` to understand why an action is not enabled.
