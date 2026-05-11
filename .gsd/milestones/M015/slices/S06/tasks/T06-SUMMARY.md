---
id: T06
parent: S06
milestone: M015
key_files:
  - docs/m015_failure_ledger.md
  - docs/combat_cli_shared_surface_proof.md
  - scripts/verify_m015_failure_ledger.py
  - scripts/verify_combat_authority_audit.py
  - .gsd/REQUIREMENTS.md
key_decisions:
  - Treat the S05 combat_cli proof as consumer proof over shared combat surfaces, not gameplay authority.
  - Preserve the overclaim guards while rewriting verifier wording toward final-baseline closure.
  - Mirror the validated requirement updates into the visible requirements snapshot so the DB-backed evidence and markdown artifact stay aligned.
duration: 
verification_result: passed
completed_at: 2026-05-08T22:33:59.757Z
blocker_discovered: false
---

# T06: Closed M015 with truthful final-baseline closure packaging and verified the real combat_cli shared-surface proof.

**Closed M015 with truthful final-baseline closure packaging and verified the real combat_cli shared-surface proof.**

## What Happened

I added a cold-reader closure summary to the M015 failure ledger so a future engineer can see what M013 proved, what it missed, what M015 fixed, what stays future work, and which commands rerun the proof. I tightened the CLI proof doc boundary so the real-binary S05 proof is clearly consumer-only and cannot be mistaken for the whole-suite M015 baseline, and I updated the verifier script wording to final-baseline markers while preserving the overclaim guards.

I also updated requirement records R089/R091/R099/R100 through gsd_requirement_update so the closure evidence is tracked in the project contract, then mirrored that validated state into the visible `.gsd/REQUIREMENTS.md` snapshot so the worktree matches the DB-backed updates. After the packaging edits, I reran the targeted S03-S05 regression bundle, both verifier scripts, `cargo test --no-run`, and the real `BEVYROGUE_JSONL=1 BEVYROGUE_CLI_PROOF=1 cargo run --bin combat_cli` proof. All checks passed; the CLI proof emitted the required `Action affordances`, `OnCombatBeat`, `OnKernelTransition`, `OnActionResolved`, `OnDamageDealt`, `OnSkillCast`, `[CLI_PROOF] validation_snapshot:`, and `holy_support=grace=` markers and did not emit the forbidden panic, readiness, or SkillBook fallback markers.

## Verification

Fresh verification after the final artifact write passed: the targeted regression bundle (`cargo test --test combat_cli_shared_surface --test presentation_metadata_boundary --test event_stream --test patamon_blueprint_seam`) passed, `python3 scripts/verify_combat_authority_audit.py` passed, `python3 scripts/verify_m015_failure_ledger.py` passed, `cargo test --no-run` passed, and the real `BEVYROGUE_JSONL=1 BEVYROGUE_CLI_PROOF=1 cargo run --bin combat_cli` proof exited 0 while containing all required shared-surface markers and none of the forbidden failure markers.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test combat_cli_shared_surface --test presentation_metadata_boundary --test event_stream --test patamon_blueprint_seam` | 0 | ✅ pass | 511ms |
| 2 | `python3 scripts/verify_combat_authority_audit.py` | 0 | ✅ pass | 20ms |
| 3 | `python3 scripts/verify_m015_failure_ledger.py` | 0 | ✅ pass | 16ms |
| 4 | `cargo test --no-run` | 0 | ✅ pass | 162ms |
| 5 | `env BEVYROGUE_JSONL=1 BEVYROGUE_CLI_PROOF=1 cargo run --bin combat_cli` | 0 | ✅ pass | 488ms |

## Deviations

The requirement updater succeeded, but the visible `.gsd/REQUIREMENTS.md` snapshot in this worktree did not refresh automatically, so I mirrored the validated requirement rows into the file to keep the artifact consistent.

## Known Issues

None.

## Files Created/Modified

- `docs/m015_failure_ledger.md`
- `docs/combat_cli_shared_surface_proof.md`
- `scripts/verify_m015_failure_ledger.py`
- `scripts/verify_combat_authority_audit.py`
- `.gsd/REQUIREMENTS.md`
