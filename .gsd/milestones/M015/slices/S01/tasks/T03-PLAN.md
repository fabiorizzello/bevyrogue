---
estimated_steps: 19
estimated_files: 5
skills_used: []
---

# T03: Capture M013 artifact and CLI gap evidence

Add the non-test artifact truth gaps to the same ledger: missing M013 validation/summary/context artifacts and the current CLI cwd-sensitive asset lookup failure.

Expected executor skills: `verify-before-complete`.

## Steps
1. Inspect the actual M013 artifact paths from the worktree and record whether `M013-VALIDATION.md`, `MILESTONE-SUMMARY.md`, and `M013-CONTEXT.md` exist without creating or repairing them in this task.
2. Inspect `src/bin/combat_cli.rs` and record the exact asset-path assumption for `assets/data/*.ron`.
3. Run the CLI smoke enough to capture whether it still panics from the assigned worktree cwd; classify the fix as S05 unless the failure disappears after prior changes.
4. Update `docs/m015_failure_ledger.md` with artifact mismatch and CLI gap sections, including downstream owner and verification command.

## Must-Haves
- [ ] The ledger states that M013 artifact truth is a closure/artifact gap, not a gameplay regression.
- [ ] The ledger states whether CLI proof is currently blocked by cwd-sensitive asset loading and points to `src/bin/combat_cli.rs` for S05.
- [ ] No `.gsd` artifact repair is performed here; S06 owns closure packaging.

## Failure Modes
| Dependency | On error | On timeout | On malformed response |
|------------|----------|------------|-----------------------|
| CLI smoke command | Capture panic/error and classify as CLI gap | Record timeout with command and defer deeper CLI proof to S05 | If output is ambiguous, inspect `src/bin/combat_cli.rs` path reads and classify from code evidence |

## Negative Tests
- **Malformed inputs**: If asset files are missing from `assets/data/`, classify separately from cwd-sensitive lookup.
- **Error paths**: A CLI panic before shared-surface proof is a successful inventory signal, not a completed CLI feature.
- **Boundary conditions**: Do not claim M013 closure is repaired merely because the gap is documented.

## Inputs

- ``docs/m015_failure_ledger.md``
- ``src/bin/combat_cli.rs``
- ``assets/data/units.ron``
- ``assets/data/skills.ron``
- ``assets/data/party.ron``
- ``.gsd/milestones/M013/M013-DISCUSSION.md``
- ``.gsd/milestones/M013/M013-PARKED.md``

## Expected Output

- ``docs/m015_failure_ledger.md``

## Verification

`grep -E "M013-VALIDATION|MILESTONE-SUMMARY|M013-CONTEXT|assets/data/units.ron|src/bin/combat_cli.rs" docs/m015_failure_ledger.md` finds the artifact and CLI evidence.

## Observability Impact

Artifact and CLI gaps are made inspectable in a stable document with exact paths, command evidence, and downstream ownership instead of being rediscovered from panics or missing files.
