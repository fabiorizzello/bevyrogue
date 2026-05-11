---
estimated_steps: 16
estimated_files: 5
skills_used: []
---

# T02: Close S04 docs and executable audit gate

Update the tracked architecture docs and verifier so the new test becomes a durable S04 contract rather than a one-off regression test.

Executor setup: load `write-docs`, `api-design` for contract honesty, `design-an-interface` only as a check against inventing a premature presentation-runtime API, and `verify-before-complete`. The doc reader is a future engineer adding UI/CLI presentation cues or new RON metadata; after reading, they should know which surfaces may consume presentation data and which surfaces remain gameplay-authoritative.

Steps:
1. Add a concise `docs/presentation_metadata_boundary.md` that states the invariant cold: RON may carry animation/QTE/presentation trigger strings, but gameplay outcomes are decided by effects, typed custom signals, per-Digimon Rust blueprints, kernel transitions, canonical events, and state/snapshot appliers.
2. Update `docs/combat_authority_map.md` to link the new doc, mark R097/S04 as proven by `tests/presentation_metadata_boundary.rs`, and clarify that canonical beats are shared lifecycle/kernel output while RON presentation metadata remains non-authoritative.
3. Update `docs/combat_mixed_pattern_drift_ledger.md` so D8 is closed/normalized by S04 with evidence from the new test/doc, while preserving S05 CLI and S06 full-suite boundaries.
4. Extend `scripts/verify_combat_authority_audit.py` with claim-scoped S04 checks: require the new test and doc references, D8 closed/normalized status, and explicit statements that `animation_sequence`, `qte`, beat metadata, and presentation triggers cannot decide damage, legality, state transitions, or status outcomes.
5. Run the full S04 verification command, including the existing failure-ledger verifier, and report any remaining `cargo test --no-run` blockers only as known S06/S05 scope if they appear during optional exploration.

Must-haves:
- Docs do not imply presentation metadata owns canonical `OnCombatBeat` or kernel beat transitions.
- Docs distinguish typed RON `custom_signals` from free-form presentation strings.
- The verifier fails on missing S04 proof markers, not just broad keyword presence.
- No direct `.gsd/` or ignored-path fixture dependencies are added to tests or verifier checks.

Failure Modes (Q5): if the new test file/doc is renamed or removed, `scripts/verify_combat_authority_audit.py` should fail with a specific missing-marker message; if docs regress to saying metadata can drive gameplay, the verifier should fail on the claim-scoped boundary text.

Load Profile (Q6): shared resources are only tracked docs/scripts and a fast Python verifier; per-operation cost is local file reads; 10x growth risk is verifier brittleness, so checks should prefer stable markers and contract phrases over fragile line-number assertions.

Negative Tests (Q7): manually exercise verifier failure while editing by temporarily missing a marker if practical, or at minimum ensure each new verifier assertion has a precise error message and that the passing run covers all S04 markers.

## Inputs

- ``tests/presentation_metadata_boundary.rs` — proof surface created by T01 that docs and verifier must reference.`
- ``docs/combat_authority_map.md` — existing source-of-truth authority map to update for S04/R097.`
- ``docs/combat_mixed_pattern_drift_ledger.md` — existing D8 drift record to close/normalize.`
- ``scripts/verify_combat_authority_audit.py` — executable audit gate to extend with S04 markers.`
- ``scripts/verify_m015_failure_ledger.py` — existing failure-ledger gate to keep passing without expanding S04 into S06.`
- ``docs/m015_failure_ledger.md` — known broad-suite blocker context that should remain downstream unless changed deliberately.`

## Expected Output

- ``docs/presentation_metadata_boundary.md` — new reader-facing contract doc for future presentation/UI/CLI work.`
- ``docs/combat_authority_map.md` — updated authority map with S04/R097 proof and link to the boundary doc.`
- ``docs/combat_mixed_pattern_drift_ledger.md` — updated D8 status and evidence.`
- ``scripts/verify_combat_authority_audit.py` — stricter S04-aware audit verifier.`

## Verification

cargo test --test presentation_metadata_boundary --test event_stream --test patamon_blueprint_seam && python3 scripts/verify_combat_authority_audit.py && python3 scripts/verify_m015_failure_ledger.py

## Observability Impact

- Signals added/changed: no runtime signals; the audit verifier becomes the diagnostic surface for documentation/contract drift.
- How a future agent inspects this: run `python3 scripts/verify_combat_authority_audit.py` for specific missing-marker failures, then inspect the named doc/test path.
- Failure state exposed: verifier error messages should identify whether the missing evidence is the S04 test, standalone doc, R097 claim, D8 closure, or metadata non-authority wording.
