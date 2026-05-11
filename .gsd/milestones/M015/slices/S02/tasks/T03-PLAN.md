---
estimated_steps: 31
estimated_files: 3
skills_used: []
---

# T03: Add executable audit coverage verification

Add an executable verifier so S02 completion is objective and downstream edits cannot silently drop required audit coverage. Expected executor skills: `test`, `verify-before-complete`.

Steps:
1. Create `scripts/verify_combat_authority_audit.py` that reads `docs/combat_authority_map.md` and `docs/combat_mixed_pattern_drift_ledger.md`.
2. Check for required requirement IDs R092-R098, required authority topics (RON, custom signals, per-Digimon blueprint ownership, kernel transitions/state, beats, snapshots, CLI), drift IDs D1-D11, downstream owners S03/S04/S05, and local-vs-rewrite-scale classification terms.
3. Extract backtick-wrapped file references from both docs and fail if a referenced project path is missing, ignored-only `.gsd/` evidence is used as required proof, or TODO/TBD/placeholder text remains.
4. Print a concise pass/fail summary naming the missing coverage category so future agents can repair docs quickly.
5. Run the verifier and update the docs from T01/T02 until it passes.

Must-haves:
- The verifier uses only tracked docs/source paths and standard Python 3 libraries.
- Failures are specific enough to identify which requirement/topic/drift ID is missing.
- The verifier does not assert fragile exact prose, only required coverage and path sanity.
- The final slice verification command is `python3 scripts/verify_combat_authority_audit.py`.

Failure Modes:
| Dependency | On error | On timeout | On malformed response |
|------------|----------|------------|------------------------|
| Audit docs | Exit non-zero with a missing-file or missing-section message. | Not applicable; local file checks only. | Exit non-zero with a named coverage/category failure instead of traceback-only output. |

Load Profile:
- Shared resources: local filesystem only.
- Per-operation cost: two markdown reads plus path existence checks.
- 10x breakpoint: many file references can make output noisy; aggregate failures by category.

Negative Tests:
- Malformed inputs: docs missing, empty docs, docs with TODO/TBD, missing drift IDs, missing requirement IDs.
- Error paths: nonexistent backtick paths and ignored `.gsd/` references used as audit proof must fail.
- Boundary conditions: allow command snippets and non-path backticks only when clearly not shaped like project paths, or keep verifier path extraction conservative to avoid false positives.

Verification:
- `python3 scripts/verify_combat_authority_audit.py`
- Optional smoke: temporarily inspect the verifier code path for clear diagnostics, then revert any intentional negative edits before completion.

Observability Impact:
- Signals added/changed: deterministic pass/fail audit verifier.
- How a future agent inspects this: run `python3 scripts/verify_combat_authority_audit.py`.
- Failure state exposed: missing requirement/topic/drift/path coverage is printed with a category-specific error.

## Inputs

- `docs/combat_authority_map.md`
- `docs/combat_mixed_pattern_drift_ledger.md`

## Expected Output

- `scripts/verify_combat_authority_audit.py`

## Verification

python3 scripts/verify_combat_authority_audit.py

## Observability Impact

Adds a deterministic verifier that future agents can run to localize missing audit coverage or stale source-path references.
