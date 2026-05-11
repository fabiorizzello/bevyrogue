---
estimated_steps: 5
estimated_files: 2
skills_used:
  - tdd
  - verify-before-complete
---

# T03: Implement action and resource affordance evaluation

Why: Target filtering alone is not enough; UI/CLI/engine adapters need to ask whether an actor can execute a specific action right now and why not. This task implements the one-call action query around the target evaluator.

Skills: use `tdd` for action/resource cases and `verify-before-complete` before marking done.

Do:
1. Implement a pure action query function in `src/combat/action_query.rs`, e.g. `query_action_affordance(snapshot, skill_book, actor_id, ActionQueryKind) -> ActionAffordance`, that resolves `Basic`, `Skill(SkillId)`, and `Ultimate` through the actor `UnitSkills` and `SkillBook`.
2. Enforce stable action priority before target aggregation: missing actor/kit/skill -> `MissingSkill`; non-active actor -> `NotActiveUnit`; phase other than `CombatPhase::WaitingAction` -> `WrongPhase`; actor KO -> `AttackerKo`; actor stunned -> `AttackerStunned`; `SkillImplementation::Deferred`/`Hidden` -> matching action and implementation statuses; SP shortfall -> disabled resource/action with `SpShortfall`; ultimate charge below trigger -> disabled resource/action with `UltimateNotReady`; zero legal targets -> `NoValidTargets`; otherwise enabled.
3. Return resource details with current/required values for SP and ultimate readiness, while leaving future tamer/energy resource codes available but not wired in S04.
4. Include target affordances in every non-missing action result so callers can explain both action and target reasons from one pure call.
5. Add tests for `NotActiveUnit`, `WrongPhase`, `AttackerKo`, `AttackerStunned`, `MissingSkill`, SP shortfall, ultimate not ready, no legal targets, implemented enabled offensive action, hidden implementation, and deferred implementation.

Failure Modes (Q5): missing actor, missing kit, missing skill book entry, stale phase, and resource shortfalls must all return explicit statuses/reasons rather than panicking or requiring display-string parsing.

Load Profile (Q6): action evaluation performs skill lookup plus target scan over the snapshot; with 10x skills/units, linear lookup is acceptable for S04 tests but should be localized for later optimization if needed.

Negative Tests (Q7): missing actor/skill, wrong active unit, wrong phase, KO/stunned actor, insufficient SP, insufficient ultimate charge, hidden/deferred skill, and empty/no-valid target lists.

## Inputs

- `src/combat/action_query.rs`
- `tests/action_affordance_query.rs`
- `src/combat/kit.rs`
- `src/combat/state.rs`
- `src/combat/ultimate.rs`
- `src/combat/sp.rs`
- `src/data/skills_ron.rs`

## Expected Output

- `src/combat/action_query.rs`
- `tests/action_affordance_query.rs`

## Verification

cargo test-dev --test action_affordance_query

## Observability Impact

Action/resource failures become inspectable as structured statuses with current/required resource values, replacing future UI guesswork and making engine parity failures easier to localize.
