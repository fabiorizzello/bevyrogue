---
estimated_steps: 6
estimated_files: 2
skills_used:
  - tdd
  - verify-before-complete
---

# T02: Implement DSL-driven target affordance evaluation

Why: The slice demo depends on callers asking which targets are legal before execution. This task implements target-level rules from `SkillDef.targeting` and proves offensive, revive, heal-like, and unsupported-shape behavior without hardcoded skill IDs.

Skills: use `tdd` to add/adjust tests before filling implementation and `verify-before-complete` before completion.

Do:
1. Add pure target-evaluation functions in `src/combat/action_query.rs`, such as `query_target_affordance(snapshot, actor_id, skill_def, target_id)` and a helper that evaluates every unit in the snapshot.
2. Enforce stable target rule priority: missing target -> `TargetNotFound`; commander -> `TargetIsCommander`; forbidden self -> `TargetIsSelf`; side mismatch -> `WrongSide`; life mismatch -> `TargetKo`/`TargetNotKo`; damaged-HP requirement with full HP -> `TargetFullHp`; unsupported non-`Single` shapes -> `Deferred(UnimplementedTargetShape)` unless the skill is hidden/deferred at the implementation layer.
3. Add tests in `tests/action_affordance_query.rs` for an implemented offensive single-target fixture: live enemy legal, ally wrong side, KO enemy target KO, commander rejected, self rejected when forbidden.
4. Add tests for an implemented revive fixture: KO ally legal, live ally target not KO, enemy wrong side.
5. Add tests for a heal-like damaged-target fixture using the new `TargetHpRule::Damaged`: damaged ally legal and full-HP ally illegal with `TargetFullHp`; do not add a real heal effect unless needed for validation.
6. Add tests for deferred row/non-single shape so targets are not reported as legal when execution is deferred.

Failure Modes (Q5): stale or missing targets must produce `TargetNotFound`; contradictory fixture data must not panic; unsupported shapes must remain deferred instead of falling through to single-target legality.

Load Profile (Q6): target evaluation scans candidate units in a snapshot; with 10x units the first bottleneck should be linear vector lookup, not mutable global state.

Negative Tests (Q7): wrong-side, KO/live mismatch, full-HP damaged-target mismatch, commander target, self target, missing target, and unsupported non-single shape.

## Inputs

- `src/combat/action_query.rs`
- `tests/action_affordance_query.rs`
- `src/data/skills_ron.rs`
- `src/combat/team.rs`
- `src/combat/types.rs`

## Expected Output

- `src/combat/action_query.rs`
- `tests/action_affordance_query.rs`

## Verification

cargo test-dev --test action_affordance_query

## Observability Impact

Target failures become inspectable as `TargetStatus` values with stable `LegalityReasonCode` reasons, giving S06/S07 a direct failure-diagnosis contract.
