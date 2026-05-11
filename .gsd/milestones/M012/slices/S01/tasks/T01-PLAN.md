---
estimated_steps: 4
estimated_files: 2
skills_used:
  - write-docs
  - verify-before-complete
---

# T01: Write and lock the UI-readiness gap matrix

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

## Inputs

- `.gsd/milestones/M012/slices/S01/S01-RESEARCH.md`
- `.gsd/REQUIREMENTS.md`
- `.gsd/DECISIONS.md`

## Expected Output

- `docs/combat_ui_readiness_gap_matrix.md`
- `tests/ui_readiness_gap_matrix_docs.rs`

## Verification

cargo test --test ui_readiness_gap_matrix_docs

## Observability Impact

No runtime signals are added. The test failure messages are the diagnostic surface for missing R085 mechanics, status classifications, or hard-boundary language.
