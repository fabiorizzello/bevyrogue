---
estimated_steps: 23
estimated_files: 3
skills_used: []
---

# T04: Add deterministic ledger verification

Create a lightweight tracked script that verifies the failure ledger contains the required S01 categories and that the stale manifest declaration is gone, giving S06 and downstream slices a deterministic acceptance check for this inventory.

Expected executor skills: `tdd`, `verify-before-complete`.

## Steps
1. Add `scripts/verify_m015_failure_ledger.py` using only tracked inputs (`docs/m015_failure_ledger.md` and `Cargo.toml`) so it is safe in normal test environments and does not read ignored `.gsd` fixtures.
2. Make the script fail with clear messages if required headings, target names, classifications, downstream owner markers, or command-evidence markers are missing.
3. Run the script, `cargo test --test battery_loop_kernel`, and `cargo test --no-run`; update the ledger with final S01 verification results and remaining classified blockers.
4. If `cargo test --no-run` still fails, ensure the failure is listed in the ledger and is not the missing `battery_loop_resolution` file.

## Must-Haves
- [ ] The script validates the ledger contains required categories: stale manifest, mechanical fixture drift, obsolete Holy Support API, Twin Core candidate regression/stale assertion, missing doc artifact, runtime reds, CLI gap, and M013 artifact gap.
- [ ] The script validates `Cargo.toml` no longer references absent `tests/battery_loop_resolution.rs`.
- [ ] Final S01 verification commands are recorded in the ledger with pass/fail classification.

## Failure Modes
| Dependency | On error | On timeout | On malformed response |
|------------|----------|------------|-----------------------|
| Ledger verification script | Emit precise missing-section/target message and fail | N/A; script is local and should be fast | Treat unreadable ledger/Cargo.toml as hard failure |

## Load Profile
- **Shared resources**: Local filesystem only.
- **Per-operation cost**: Reads two small tracked files.
- **10x breakpoint**: None expected; avoid spawning Cargo from the verifier itself.

## Negative Tests
- **Malformed inputs**: Script should fail if the ledger file is missing or empty.
- **Error paths**: Script should fail if `Cargo.toml` still names `battery_loop_resolution` without a matching file.
- **Boundary conditions**: Script should fail on placeholder classifications like `TBD` or `unknown` in required sections.

## Inputs

- ``docs/m015_failure_ledger.md``
- ``Cargo.toml``

## Expected Output

- ``scripts/verify_m015_failure_ledger.py``
- ``docs/m015_failure_ledger.md``

## Verification

`python3 scripts/verify_m015_failure_ledger.py`, `cargo test --test battery_loop_kernel`, and `cargo test --no-run` have fresh results; any non-zero `cargo test --no-run` result is fully classified in `docs/m015_failure_ledger.md` and is not the missing `battery_loop_resolution` target.

## Observability Impact

The verification script exposes missing ledger categories and stale manifest state as deterministic, actionable errors with no need to parse a large Cargo log.
