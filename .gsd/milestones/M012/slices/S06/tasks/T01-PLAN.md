---
estimated_steps: 43
estimated_files: 3
skills_used: []
---

# T01: Add pure query_intent_legality helper and reason-priority tests

# T01: Add pure query_intent_legality helper and reason-priority tests

**Slice:** S06 — Engine validation integration
**Milestone:** M012

## Description

Add a pure `query_intent_legality()` function to `src/combat/action_query.rs` that validates a specific selected intent (actor + action kind + target) against the existing query infrastructure and returns `Result<(), LegalityReasonCode>`. This helper is the bridge between the aggregate affordance API (which returns all targets with statuses) and the engine's need to validate one specific submitted intent. The function must have explicit, tested reason priority so the engine failure matches what preflight would have told the caller.

The function should:
1. Resolve the skill via `resolve_action_skill()` from the actor's kit + skill book. On failure: `MissingSkill`.
2. Check implementation status via the skill's `ImplementationStatus`. If hidden/deferred: return the corresponding reason.
3. Check actor/phase/resource status via `action_and_resource_status_for_snapshot()`. If not `ActionStatus::Available`: map to the corresponding `LegalityReasonCode` (e.g. `AttackerKo`, `AttackerStunned`, `NotActiveUnit`, `WrongPhase`, `SpShortfall`, `UltimateNotReady`). Skip `NoValidTargets` at this stage.
4. Check the specific submitted target via `target_status_for_unit()`. If not `TargetStatus::Valid`: return the target-specific reason (`WrongSide`, `TargetKo`, `TargetNotKo`, `TargetFullHp`, `TargetIsSelf`, `TargetIsCommander`, `TargetNotFound`, `UnimplementedTargetShape`).
5. Return `Ok(())` if all checks pass.

Reason priority matters: if a revive targets a live ally, the reason should be `TargetNotKo` (the specific target reason), not `NoValidTargets` (the aggregate). The priority is: missing skill > implementation > actor/resource > selected target.

## Steps

1. Add `pub fn query_intent_legality(snapshot: &CombatQuerySnapshot, skill_book: &SkillBook, actor_id: UnitId, kind: &ActionQueryKind, target_id: UnitId) -> Result<(), LegalityReasonCode>` to `src/combat/action_query.rs`.
2. Implement the 5-step validation chain described above, reusing existing helpers (`resolve_action_skill`, `action_and_resource_status_for_snapshot`, `target_status_for_unit`).
3. For `ActionStatus` → `LegalityReasonCode` mapping, add a small helper or match block. `ActionStatus::Available` and `ActionStatus::Unavailable(NoValidTargets)` both pass through to step 4 (target check). All other `Unavailable` reasons map directly.
4. Add tests in `tests/action_affordance_query.rs` covering reason priority:
   - Revive skill forced against live ally → `TargetNotKo`
   - Offensive skill forced against ally → `WrongSide`
   - Offensive skill forced against KO enemy → `TargetKo`
   - Offensive skill forced against commander → `TargetIsCommander`
   - Skill with missing target ID → `TargetNotFound`
   - SP shortfall actor → `SpShortfall`
   - Non-active actor (snapshot `is_active = false`) → `NotActiveUnit`
   - KO attacker → `AttackerKo`
   - Stunned attacker → `AttackerStunned`
   - Valid intent → `Ok(())`
5. Run `cargo test-dev --test action_affordance_query` and confirm all pass.

## Must-Haves

- [ ] `query_intent_legality()` is public and returns `Result<(), LegalityReasonCode>`
- [ ] Reason priority: missing skill > implementation > actor/resource > selected target
- [ ] `NoValidTargets` aggregate is never returned when a specific target reason exists
- [ ] At least 10 pure test cases covering the priority chain
- [ ] No Bevy dependency — function is pure over `CombatQuerySnapshot` + `SkillBook`

## Verification

- `cargo test-dev --test action_affordance_query` passes with new intent legality tests

## Inputs

- `src/combat/action_query.rs` — existing query helpers to compose
- `src/data/skills_ron.rs` — `LegalityReasonCode` enum, `SkillDef`, `ActionQueryKind`
- `tests/action_affordance_query.rs` — existing test helpers (snapshot_with, unit, actor_with_skills, skill factories)

## Expected Output

- `src/combat/action_query.rs` — new `query_intent_legality()` function added
- `tests/action_affordance_query.rs` — new intent legality test cases added

## Inputs

- ``src/combat/action_query.rs` — existing pure query helpers to compose into intent validation`
- ``src/data/skills_ron.rs` — LegalityReasonCode enum, SkillDef, ActionQueryKind definitions`
- ``tests/action_affordance_query.rs` — existing test helpers (snapshot_with, unit, actor_with_skills, skill factories)`

## Expected Output

- ``src/combat/action_query.rs` — new public query_intent_legality() function`
- ``tests/action_affordance_query.rs` — 10+ new intent legality pure test cases`

## Verification

cargo test-dev --test action_affordance_query
