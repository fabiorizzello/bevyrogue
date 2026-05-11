# S01: UI-readiness gap matrix and legality contract

**Goal:** Produce tracked, executable-proof documentation that defines the UI-readiness gap matrix and the shared legality/affordance contract for M012. The matrix must classify UI-affecting mechanics as implemented, to-fix-now, deferred, or hidden, and the contract must specify action, target, resource, and implementation statuses with stable machine-readable reasons sourced from skill data plus a world snapshot rather than UI-specific rules.
**Demo:** After this: a design/code gap matrix says exactly what M012 will fix, what it will declare as deferred, and what remains post-UI.

## Must-Haves

- `docs/combat_ui_readiness_gap_matrix.md` exists and maps every researched UI-affecting mechanic to one of `Implemented`, `ToFixNow`, `Deferred`, or `Hidden`, including owning downstream slice(s) and UI truth risk.
- `docs/skill_legality_contract.md` exists and defines the M012 pre-execution legality contract for `ActionStatus`, `TargetStatus`, `ResourceStatus`, and `ImplementationStatus` plus stable reason-code families for R084.
- The docs explicitly cover R084 and R085, decisions D053/D054, and the hard boundary that CLI/windowed UI must not contain skill-ID-specific legality rules.
- Executable tests assert that the matrix and contract contain the required mechanics, status vocabulary, reason codes, and requirement/decision links so downstream slices cannot accidentally drop the contract.
- No code changes in this slice implement gameplay behavior; later slices consume these docs/tests as the contract baseline.

## Proof Level

- This slice proves: Contract proof. This slice does not wire runtime behavior; it creates tracked contract artifacts plus deterministic tests that verify the artifacts contain the required requirement coverage and status/reason vocabulary. Real runtime is not required beyond the existing Rust test runner. Human/UAT is not required.

## Integration Closure

Upstream surfaces consumed: `.gsd/milestones/M012/slices/S01/S01-RESEARCH.md` findings, `.gsd/REQUIREMENTS.md` R084/R085, and `.gsd/DECISIONS.md` D053/D054. New wiring introduced in this slice: tracked docs under `docs/` and doc-contract integration tests under `tests/`. Remaining before milestone usability: S02-S08 must implement the matrix decisions in the DSL, pure query API, engine validation, CLI/windowed adapters, and deferred mechanic declaration surfaces.

## Verification

- The slice adds no runtime observability. Diagnostics are provided through executable doc-contract tests: failures point at missing mechanics, status vocabulary, reason codes, or requirement/decision links in the tracked documentation. Redaction constraints: none; docs contain gameplay/system design only, no secrets or player data.

## Tasks

- [x] **T01: Write and lock the UI-readiness gap matrix** `est:1h`
  Create the tracked R085 gap-matrix artifact that downstream implementation slices will follow. Executor skills_used frontmatter expectation: `write-docs`, `verify-before-complete`.

Why: R085 requires UI-affecting mechanics to be truthfully implemented, declared deferred, or hidden before UI exposes them. The research already identified the gaps; this task turns those findings into a durable tracked matrix and executable guardrail.

Do:
1. Add `docs/combat_ui_readiness_gap_matrix.md` with a concise intro linking R085, D053, and D054, then a table with columns for mechanic/area, current evidence, UI truth risk, classification (`Implemented`, `ToFixNow`, `Deferred`, `Hidden`), downstream slice, and contract note.
2. Include at least these mechanics/areas: offensive single-target damage, revive, heal-like examples, cleanse/silence/guard, Row/AllEnemies TargetShape, SelfOnly, mixed-effect target semantics such as `angemon_ult`, ally Toughness, zero-max enemy Toughness, Energy caps, SP/ultimate readiness, attacker state, commander target, Tamer Gauge/Commands, Child Tamer Gauge boost, enemy counterplay/telegraphs, structured failure reasons, and windowed active unit.
3. State the hard boundary: no CLI/windowed skill-ID-specific legality rules; if UI needs a rule it must come from DSL/query output or a queryable deferred/hidden declaration.
4. Add `tests/ui_readiness_gap_matrix_docs.rs` that reads the tracked doc with `include_str!("../docs/combat_ui_readiness_gap_matrix.md")` and asserts the required classification vocabulary, R085/D053/D054 links, hard-boundary text, and all required mechanics/areas are present.

Failure Modes:
- Dependency: tracked docs. On missing doc, the test fails at compile/test time through `include_str!`. On malformed content, assertion messages should name the missing mechanic/status/link.

Load Profile:
- Shared resources: none beyond the test runner reading one tracked markdown file.
- Per-operation cost: one static file include and string assertions; trivial.
- 10x breakpoint: not applicable.

Negative Tests:
- Malformed inputs: the test should reject missing required mechanics, missing classification statuses, or placeholder text such as `TBD`/`TODO`.
- Boundary conditions: ensure at least one item is classified in each required status family where applicable, especially `ToFixNow`, `Deferred`, and `Hidden`.

Verify: `cargo test --test ui_readiness_gap_matrix_docs`
Done when: the gap matrix is tracked under `docs/`, names the required mechanics with classifications and downstream slices, and the targeted doc test passes.
  - Files: `docs/combat_ui_readiness_gap_matrix.md`, `tests/ui_readiness_gap_matrix_docs.rs`
  - Verify: cargo test --test ui_readiness_gap_matrix_docs

- [x] **T02: Write and lock the legality/affordance contract** `est:1h`
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
  - Files: `docs/skill_legality_contract.md`, `tests/skill_legality_contract_docs.rs`
  - Verify: cargo test --test skill_legality_contract_docs

## Files Likely Touched

- docs/combat_ui_readiness_gap_matrix.md
- tests/ui_readiness_gap_matrix_docs.rs
- docs/skill_legality_contract.md
- tests/skill_legality_contract_docs.rs
