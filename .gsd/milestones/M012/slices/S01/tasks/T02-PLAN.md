---
estimated_steps: 5
estimated_files: 2
skills_used:
  - write-docs
  - verify-before-complete
---

# T02: Write and lock the legality/affordance contract

Create the tracked R084 legality contract that downstream implementation slices must implement in data and code. Executor skills_used frontmatter expectation: `write-docs`, `verify-before-complete`.

Why: R084 requires the same DSL-backed pre-execution query surface to answer action status, target status, resource readiness, and stable reasons for UI, CLI, AI, tests, and engine validation. This task defines the exact vocabulary before S03-S07 implement it.

Do:
1. Add `docs/skill_legality_contract.md` with sections for goals/non-goals, data ownership, query inputs, status shapes, reason-code families, target/resource/implementation semantics, engine parity, and consumer rules.
2. Define the contract vocabulary: `ActionStatus = Enabled | Disabled { reason } | Deferred { reason } | Hidden { reason }`, `TargetStatus = Legal | Illegal { reason } | Deferred { reason } | Hidden { reason }`, `ResourceStatus = Ready | Insufficient { reason, current, required } | Deferred | Hidden`, and `ImplementationStatus = Implemented | Deferred { reason } | Hidden { reason }`.
3. Include stable reason codes at minimum: `NotActiveUnit`, `WrongPhase`, `AttackerKo`, `AttackerStunned`, `MissingSkill`, `SpShortfall`, `UltimateNotReady`, `UnimplementedEffect`, `UnimplementedTargetShape`, `TargetNotFound`, `TargetIsSelf`, `TargetIsCommander`, `WrongSide`, `TargetKo`, `TargetNotKo`, `TargetFullHp`, `TargetNotDamaged`, `NoValidTargets`, `ToughnessEnemyOnly`, `TamerGaugeDeferred`, `TamerCommandDeferred`, `ChargedTelegraphDeferred`, `EnemyTraitDeferred`, and `EnergyCapReached`.
4. Specify that the pure query API should consume skill data plus a world snapshot, keep display strings separate from reason codes, and that S06 engine rejection must derive `CombatEventKind::OnActionFailed` text from the same reason returned by preflight.
5. Add `tests/skill_legality_contract_docs.rs` that reads the tracked doc with `include_str!("../docs/skill_legality_contract.md")` and asserts the status vocabulary, reason-code set, R084/D053 links, engine parity requirement, and no skill-ID-specific UI rule boundary are present.

Failure Modes:
- Dependency: tracked contract doc. On missing doc, the test fails at compile/test time through `include_str!`. On malformed content, assertion messages should name the missing status, reason code, or parity rule.

Load Profile:
- Shared resources: none beyond the test runner reading one tracked markdown file.
- Per-operation cost: one static file include and string assertions; trivial.
- 10x breakpoint: not applicable.

Negative Tests:
- Malformed inputs: the test should reject missing statuses, missing reason-code families, missing R084/D053 links, or placeholder text such as `TBD`/`TODO`.
- Boundary conditions: ensure all four status types are represented and that action-, target-, resource-, and implementation-level reasons are all present.

Verify: `cargo test --test skill_legality_contract_docs`
Done when: the legality contract is tracked under `docs/`, contains the required status/reason vocabulary and parity rules, and the targeted doc test passes.

## Inputs

- `docs/combat_ui_readiness_gap_matrix.md`
- `.gsd/milestones/M012/slices/S01/S01-RESEARCH.md`
- `.gsd/REQUIREMENTS.md`
- `.gsd/DECISIONS.md`

## Expected Output

- `docs/skill_legality_contract.md`
- `tests/skill_legality_contract_docs.rs`

## Verification

cargo test --test skill_legality_contract_docs

## Observability Impact

No runtime signals are added. The contract test failure messages are the diagnostic surface for missing R084 statuses, reason codes, or engine/query parity language.
