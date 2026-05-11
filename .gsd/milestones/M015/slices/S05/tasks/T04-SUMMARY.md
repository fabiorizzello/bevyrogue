---
id: T04
parent: S05
milestone: M015
key_files:
  - docs/combat_cli_shared_surface_proof.md
  - docs/combat_authority_map.md
  - docs/combat_mixed_pattern_drift_ledger.md
  - docs/m015_failure_ledger.md
  - scripts/verify_combat_authority_audit.py
  - scripts/verify_m015_failure_ledger.py
key_decisions:
  - Treat S05 as closing targeted real-binary CLI shared-surface proof only; preserve S06 ownership of broad fixture repair and final full-suite baseline closure.
duration: 
verification_result: passed
completed_at: 2026-05-08T20:11:09.847Z
blocker_discovered: false
---

# T04: Closed S05 with durable combat_cli shared-surface proof docs and claim-scoped audit verifiers that preserve the S06 full-suite boundary.

**Closed S05 with durable combat_cli shared-surface proof docs and claim-scoped audit verifiers that preserve the S06 full-suite boundary.**

## What Happened

Added `docs/combat_cli_shared_surface_proof.md` for the real-binary CLI proof contract: proof env vars, stable markers, shared query/event/beat/kernel/snapshot surfaces, failure markers, and the explicit rule that CLI presentation metadata cannot decide gameplay outcomes. Updated the authority map, mixed-pattern drift ledger, and M015 failure ledger so D3/D9 and the CLI runtime/asset-path/consumer proof are closed for targeted S05 evidence while broad `cargo test --no-run`, fixture repair, full CLI/future UI migration, and final full-suite baseline closure remain S06 or later responsibilities. Extended both Python verifier scripts with S05 claim-scoped doc/test/env/shared-surface markers, S06 boundary markers, placeholder checks, and explicit overclaim guards for final baseline closure.

## Verification

Ran the updated Python verifier scripts with py_compile, the full task-required Rust verifier bundle, and the direct CLI proof command. The required bundle passed: `cargo test --test combat_cli_shared_surface --test presentation_metadata_boundary --test event_stream --test patamon_blueprint_seam && python3 scripts/verify_combat_authority_audit.py && python3 scripts/verify_m015_failure_ledger.py` exited 0. The direct proof command `BEVYROGUE_JSONL=1 BEVYROGUE_CLI_PROOF=1 cargo run --bin combat_cli` also exited 0 and emitted the expected shared-surface markers including action affordances, beat/kernel transitions, action/damage/cast events, and `[CLI_PROOF] validation_snapshot:` with `holy_support=grace=`.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `python3 -m py_compile scripts/verify_combat_authority_audit.py scripts/verify_m015_failure_ledger.py && python3 scripts/verify_combat_authority_audit.py && python3 scripts/verify_m015_failure_ledger.py` | 0 | ✅ pass | 61ms |
| 2 | `cargo test --test combat_cli_shared_surface --test presentation_metadata_boundary --test event_stream --test patamon_blueprint_seam && python3 scripts/verify_combat_authority_audit.py && python3 scripts/verify_m015_failure_ledger.py` | 0 | ✅ pass | 1077ms |
| 3 | `BEVYROGUE_JSONL=1 BEVYROGUE_CLI_PROOF=1 cargo run --bin combat_cli` | 0 | ✅ pass | 477ms |

## Deviations

None.

## Known Issues

Broad `cargo test --no-run` blockers remain intentionally assigned to S06; no new blocker was discovered.

## Files Created/Modified

- `docs/combat_cli_shared_surface_proof.md`
- `docs/combat_authority_map.md`
- `docs/combat_mixed_pattern_drift_ledger.md`
- `docs/m015_failure_ledger.md`
- `scripts/verify_combat_authority_audit.py`
- `scripts/verify_m015_failure_ledger.py`
