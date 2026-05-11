---
estimated_steps: 3
estimated_files: 4
skills_used: []
---

# T05: Run the full runtime suite and repair classified regressions

Move from compile green to runtime green. Run `cargo test --no-fail-fast`, classify each failing test before changes, and repair in-scope regressions by default. Expected executor skills: `grill-me`, `tdd`, `verify-before-complete`, `write-docs`.

Steps: (1) run the full headless suite through `gsd_exec`; (2) add runtime classifications to the ledger before fixing; (3) write/adjust tests to current contracts or fix minimal source bugs; (4) rerun broad suite to exit 0, optionally followed by plain `cargo test`.

Must-haves: no ignored/deleted red tests without replacement coverage; fixes preserve S02-S05 authority and presentation boundaries; runtime proof remains deterministic/headless. Failure modes/Q5-Q7: timeouts preserve transcript and use scoped reruns for diagnosis; flaky failures are isolated to deterministic inputs before code changes.

## Inputs

- ``docs/m015_failure_ledger.md` — compile-baseline evidence and classification discipline.`
- ``tests` — integration suite that must be green without deleting coverage.`
- ``src/combat` — combat source only if a classified runtime regression requires repair.`
- ``src/data` — data schema/loaders only if runtime failures prove contract drift.`

## Expected Output

- ``docs/m015_failure_ledger.md` — full-suite runtime classifications and green broad-suite evidence.`
- ``tests` — updated tests/fixtures for current contracts where runtime failures require changes.`
- ``src/combat` — minimal source fixes only if runtime regressions require them.`
- ``src/data` — minimal source fixes only if data contract regressions require them.`

## Verification

`cargo test --no-fail-fast` exits 0 via `gsd_exec`; if practical, also run `cargo test` and record its exit code in `docs/m015_failure_ledger.md`.
