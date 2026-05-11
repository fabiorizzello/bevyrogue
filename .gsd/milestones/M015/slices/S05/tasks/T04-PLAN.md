---
estimated_steps: 16
estimated_files: 7
skills_used: []
---

# T04: Close S05 docs and audit verifiers

Turn the real-binary proof into durable architecture evidence and update the claim-scoped audit gates without claiming S06's final baseline.

Executor setup: load `write-docs` and `verify-before-complete`; use `api-design` as a contract checklist for named outputs and honest failure semantics. Preserve S06 as the owner of full-suite closure.

Steps:
1. Add `docs/combat_cli_shared_surface_proof.md` explaining the proof env vars, stable markers, shared surfaces consumed, and the explicit non-authority rule for CLI presentation metadata.
2. Update `docs/combat_authority_map.md` so the CLI row changes from pending/downstream proof to S05 real-binary proof, while retaining the boundary that full CLI/future UI migration and full-suite baseline are not complete until S06.
3. Update `docs/combat_mixed_pattern_drift_ledger.md` to close/normalize the S05 CLI drift items (notably runtime registration/snapshot proof and D9), preserving any rewrite-scale or S06-only fixture/full-suite work.
4. Update `docs/m015_failure_ledger.md` with the new S05 proof command/result, mark the CLI / asset-path / consumer proof repair as closed by S05, and keep broad `cargo test --no-run`/no-fail-fast blockers assigned to S06.
5. Extend `scripts/verify_combat_authority_audit.py` and `scripts/verify_m015_failure_ledger.py` with claim-scoped S05 markers for `tests/combat_cli_shared_surface.rs`, `docs/combat_cli_shared_surface_proof.md`, proof env vars, shared surfaces, and no full-suite overclaim.

Must-haves:
- Docs identify the CLI proof as real-binary evidence over shared query/event/beat/kernel/snapshot surfaces.
- Docs state the CLI does not parse `animation_sequence`, `qte`, beat wording, or presentation trigger text for gameplay outcomes.
- Verifier scripts fail if the S05 proof test/doc markers disappear or if docs overclaim final S06 baseline closure.
- The M015 ledger continues to distinguish targeted S05 proof from S06 full-suite closure.

Failure Modes (Q5): stale marker wording should fail through verifier scripts; missing proof docs should fail explicitly; overclaiming final baseline should be prevented by required S06 boundary markers.

Load Profile (Q6): shared resources are tracked docs and small Python verifier scripts; per run is simple file scanning; 10x load is negligible.

Negative Tests (Q7): verifier checks should cover missing S05 doc/test references, missing proof env markers, missing shared-surface terms, missing S06 boundary wording, and placeholder `TBD`/`unknown` classifications.

## Inputs

- `tests/combat_cli_shared_surface.rs`
- `docs/combat_authority_map.md`
- `docs/combat_mixed_pattern_drift_ledger.md`
- `docs/m015_failure_ledger.md`
- `scripts/verify_combat_authority_audit.py`
- `scripts/verify_m015_failure_ledger.py`

## Expected Output

- `docs/combat_cli_shared_surface_proof.md`
- `docs/combat_authority_map.md`
- `docs/combat_mixed_pattern_drift_ledger.md`
- `docs/m015_failure_ledger.md`
- `scripts/verify_combat_authority_audit.py`
- `scripts/verify_m015_failure_ledger.py`

## Verification

cargo test --test combat_cli_shared_surface --test presentation_metadata_boundary --test event_stream --test patamon_blueprint_seam && python3 scripts/verify_combat_authority_audit.py && python3 scripts/verify_m015_failure_ledger.py

## Observability Impact

Signals documented: the docs/verifiers become the long-term inspection surface for CLI proof health. Future agents inspect with the targeted Rust test bundle and the two Python verifier scripts; failures should name the missing S05 claim marker or stale S06 boundary.
