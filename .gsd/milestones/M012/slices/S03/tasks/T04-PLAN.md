---
estimated_steps: 46
estimated_files: 5
skills_used: []
---

# T04: Run final S03 contract regressions and align documentation references

---
estimated_steps: 4
estimated_files: 3
skills_used:
  - test
  - verify-before-complete
---

Close the slice by running the contract-level regression set and making any small documentation/test-name alignment needed so future S04 executors know that S03's output is the `SkillDef` metadata contract, not a completed query API.

Steps:
1. Run the planned S03 verification commands and fix any failures that are caused by stale imports, outdated test fixture metadata, or contract vocabulary drift.
2. If validation introduces new public reason-code names or metadata terminology, update `docs/skill_legality_contract.md` only to keep names aligned; do not expand S03 into the S04 query API design.
3. Ensure no CLI/windowed/UI file gained per-skill legality hardcoding while migrating data. Use ripgrep checks for obvious skill-id legality branches if touched files raise suspicion.
4. Leave a concise comment in `src/data/skills_ron.rs` or tests explaining that side/life/self metadata is declared in S03 and enforced/queryable in later S04/S06 slices.

Must-haves:
- Final focused regression commands pass freshly in this workspace.
- Contract vocabulary in docs/tests/code is aligned on stable reason names.
- No consumer-specific legality table or per-skill UI/CLI workaround was introduced.
- The slice stops at metadata plus resolution shape propagation; no partial pure-query API is added here.

Failure Modes:
- **Vocabulary drift**: doc contract test should catch missing reason/status names; update code or docs deliberately, not casually.
- **Scope creep**: if implementation starts adding preflight query API or engine side/life enforcement, stop and defer that work to S04/S06 unless needed to fix compile/test regressions.
- **Windowed surprise**: S03 should not normally require windowed compile, but if exported type changes break windowed imports, run `cargo check --features "dev windowed"` and fix compile-only fallout.

Load Profile:
- Shared resources: test runner only.
- Per-operation cost: deterministic Rust tests; no runtime load concerns.
- 10x breakpoint: none for this validation/documentation task.

Negative Tests:
- **Malformed inputs**: covered by `cargo test-dev skills_ron` from T02.
- **Boundary conditions**: covered by target-shape and revive regressions from T03.
- **Error paths**: covered by unsupported shape rejection tests and validation errors.

Verification:
- `cargo test-dev --test target_shape_truthfulness --test skill_legality_contract_docs --test revive_semantics --test patamon_revive`
- `cargo test-dev skills_ron`
- Optional if exported type changes touch feature-gated UI imports: `cargo check --features "dev windowed"`

Inputs:
- `docs/skill_legality_contract.md` — contract vocabulary baseline.
- `src/data/skills_ron.rs` — final schema/validation/tests.
- `src/combat/resolution.rs` — final metadata-to-resolution wiring.
- `tests/skill_legality_contract_docs.rs` — doc vocabulary regression.
- `tests/target_shape_truthfulness.rs` — unsupported-shape regression.

Expected Output:
- `docs/skill_legality_contract.md` — only updated if needed for exact reason-code alignment.
- `src/data/skills_ron.rs` — final comments/import fixes if needed.
- `src/combat/resolution.rs` — final import/helper cleanup if needed.
- `tests/skill_legality_contract_docs.rs` — only updated if names intentionally change.
- `tests/target_shape_truthfulness.rs` — final fixture cleanup if needed.

## Inputs

- `docs/skill_legality_contract.md`
- `src/data/skills_ron.rs`
- `src/combat/resolution.rs`
- `tests/skill_legality_contract_docs.rs`
- `tests/target_shape_truthfulness.rs`

## Expected Output

- `docs/skill_legality_contract.md`
- `src/data/skills_ron.rs`
- `src/combat/resolution.rs`
- `tests/skill_legality_contract_docs.rs`
- `tests/target_shape_truthfulness.rs`

## Verification

cargo test-dev --test target_shape_truthfulness --test skill_legality_contract_docs --test revive_semantics --test patamon_revive && cargo test-dev skills_ron

## Observability Impact

Final regression evidence ensures the diagnostic surfaces introduced by validation and preserved by unsupported-shape rejection remain discoverable through tests and stable reason names.
