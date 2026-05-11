---
estimated_steps: 5
estimated_files: 4
skills_used:
  - write-docs
  - rust-best-practices
  - verify-before-complete
---

# T03: Update combat authority docs for the migrated predator blueprint

**Slice:** S02 — Dorumon/DORUgamon Predator Loop Blueprint
**Milestone:** M016

## Description

Update combat authority documentation and audit markers so Dorumon/DORUgamon is recorded as a migrated Predator Loop blueprint seam without overclaiming full roster or UI/CLI completion.

## Failure Modes

| Dependency | On error | On timeout | On malformed response |
|------------|----------|-----------|----------------------|
| Authority audit script | Report missing marker names clearly; do not make broad regexes that hide omissions. | Not applicable; local file scan. | Treat absent files/markers as audit failures. |
| Documentation evidence | Tests should remain the source of proof; docs must not overclaim behavior not verified by T01/T02. | Not applicable. | If docs and tests disagree, update docs to match executable proof rather than changing proof wording only. |

## Negative Tests

- **Malformed inputs**: Audit should fail if expected Dorumon blueprint/test evidence is missing.
- **Error paths**: If the audit script has explicit markers, add Dorumon-specific marker checks rather than broad wildcard matches.
- **Boundary conditions**: Docs must distinguish migrated Dorumon/DORUgamon from later roster identities that remain future work.

## Steps

1. Update `docs/combat_current.md` to describe the Dorumon/DORUgamon path: RON custom signal -> `src/combat/blueprints/dorumon.rs` -> generic `PredatorLoopTransition` -> `CombatEvent`/snapshot surfaces.
2. Update `docs/contracts/combat_authority_map.md` with `src/combat/blueprints/dorumon.rs`, `tests/dorumon_blueprint.rs`, and `tests/dorumon_predator_runtime.rs` evidence alongside existing blueprint seams.
3. Update `docs/contracts/combat_mixed_pattern_drift_ledger.md` only if it still classifies the Dorumon/DORUgamon predator migration as future-only drift; preserve M015 historical wording.
4. If `scripts/verify_combat_authority_audit.py` has explicit marker expectations, extend it to recognize the new Dorumon evidence; otherwise leave it unchanged and run it as a regression check.
5. Run the audit plus targeted Dorumon and predator-loop tests.

## Must-Haves

- [ ] Docs state this is contract/headless integration proof, not full playable CLI/windowed UX proof.
- [ ] CD001/CD004/CD005/CD006/CD007 remain honored: no shared character branches, RON as declarative intent, CLI/UI as consumers.
- [ ] Audit reads only tracked files and fails clearly when expected markers are missing.
- [ ] Later roster identities remain explicitly future work.

## Verification

- `python3 scripts/verify_combat_authority_audit.py`
- `cargo test --test dorumon_blueprint --test dorumon_predator_runtime --test predator_loop_kernel --no-fail-fast`

## Inputs

- `docs/combat_current.md` — current combat authority summary.
- `docs/contracts/combat_authority_map.md` — current authority map evidence.
- `docs/contracts/combat_mixed_pattern_drift_ledger.md` — drift/future-work status table.
- `scripts/verify_combat_authority_audit.py` — deterministic authority marker audit.
- `tests/dorumon_blueprint.rs` — executable direct mapping evidence from T01.
- `tests/dorumon_predator_runtime.rs` — executable runtime seam evidence from T02.

## Expected Output

- `docs/combat_current.md` — documents Dorumon/DORUgamon as migrated predator-loop blueprint seam.
- `docs/contracts/combat_authority_map.md` — adds Dorumon blueprint/test evidence.
- `docs/contracts/combat_mixed_pattern_drift_ledger.md` — updates Dorumon migration status only if applicable.
- `scripts/verify_combat_authority_audit.py` — includes Dorumon markers only if needed for audit coverage.
