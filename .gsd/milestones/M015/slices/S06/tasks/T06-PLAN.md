---
estimated_steps: 3
estimated_files: 10
skills_used: []
---

# T06: Preserve S03-S05 proof surfaces and package truthful M013/M015 closure

Run the targeted S03-S05 regression bundle, real `combat_cli` proof, and audit verifiers; then update closure docs and requirement evidence so M015 truthfully supersedes M013 gaps. Expected executor skills: `api-design`, `grill-me`, `tdd`, `verify-before-complete`, `write-docs`.

Steps: (1) run the targeted regression bundle; (2) update verifier scripts from pre-S06 boundary wording to final-baseline checks while preserving overclaim guards; (3) run both verifier scripts; (4) run real `BEVYROGUE_JSONL=1 BEVYROGUE_CLI_PROOF=1 cargo run --bin combat_cli` and check required/forbidden markers; (5) update final closure prose and use `gsd_requirement_update` for R089/R091/R099/R100.

Must-haves: targeted proof and CLI proof run after final code changes; verifier scripts still reject fixture-only proof, presentation authority overclaims, and erased M013 gaps; closure docs tell a cold future agent what M013 proved/missed, what M015 fixed, what remains future work, and how to rerun proof. Failure modes/Q5-Q7: CLI failures are classified by query/event/beat/kernel/snapshot surface; verifier failures are fixed by evidence/docs/guard logic, not suppression.

## Inputs

- ``docs/m015_failure_ledger.md` — final compile/runtime evidence and closure trail.`
- ``scripts/verify_m015_failure_ledger.py` — verifier to update for S06-final evidence semantics.`
- ``scripts/verify_combat_authority_audit.py` — verifier preserving authority/presentation/CLI boundaries.`
- ``tests/combat_cli_shared_surface.rs` — S05 shared-surface contract test.`
- ``tests/presentation_metadata_boundary.rs` — presentation non-authority regression test.`
- ``tests/event_stream.rs` — event-stream regression test.`
- ``tests/patamon_blueprint_seam.rs` — Patamon seam regression test.`
- ``src/bin/combat_cli.rs` — real CLI proof entrypoint.`
- ``docs/combat_cli_shared_surface_proof.md` — CLI proof documentation.`

## Expected Output

- ``scripts/verify_m015_failure_ledger.py` — final-baseline verifier semantics with overclaim guards preserved.`
- ``scripts/verify_combat_authority_audit.py` — audit verifier updated only where final evidence changes boundary wording.`
- ``docs/m015_failure_ledger.md` — final proof table and truthful M013/M015 closure packaging.`
- ``docs/combat_cli_shared_surface_proof.md` — real CLI proof evidence and rerun instructions.`
- ``.gsd/REQUIREMENTS.md` — generated requirement evidence updates for R089, R091, R099, and R100 via `gsd_requirement_update`.`

## Verification

`cargo test --test combat_cli_shared_surface --test presentation_metadata_boundary --test event_stream --test patamon_blueprint_seam`; `python3 scripts/verify_combat_authority_audit.py`; `python3 scripts/verify_m015_failure_ledger.py`; `BEVYROGUE_JSONL=1 BEVYROGUE_CLI_PROOF=1 cargo run --bin combat_cli`; and fresh broad-suite evidence remains recorded after final changes.
