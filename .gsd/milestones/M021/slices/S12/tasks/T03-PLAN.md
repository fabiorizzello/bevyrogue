---
estimated_steps: 1
estimated_files: 8
skills_used: []
---

# T03: Realign CLI and shared-surface proofs to the generic validation contract

Why: once roster and validation become generic, the CLI/headless/windowed consumers and shared-surface tests must prove the new contract instead of the retired named fields. Skills: tdd, bevy, rust-best-practices, verify-before-complete. Do: update CLI/headless/windowed validation formatting call sites only as needed for the new snapshot shape; rewrite shared-surface and affordance assertions to check owner-keyed output and the absence of digimon-named shared fields; keep proof focused on executable boundary checks rather than roadmap-wide auto-discovery claims. Include negative checks that snapshot rendering stays stable when optional blueprint sections are absent. Done when: CLI/shared-surface regressions assert the new contract, structural greps are part of the final proof, and both cargo check modes stay green.

## Inputs

- `src/bin/combat_cli.rs`
- `src/headless.rs`
- `src/windowed.rs`
- `tests/combat_cli_shared_surface.rs`
- `tests/holy_support_affordance.rs`
- `tests/holy_support_mechanics.rs`
- `tests/holy_support_resolution.rs`
- `src/combat/observability.rs`

## Expected Output

- `src/bin/combat_cli.rs`
- `src/headless.rs`
- `src/windowed.rs`
- `tests/combat_cli_shared_surface.rs`
- `tests/holy_support_affordance.rs`
- `tests/holy_support_mechanics.rs`
- `tests/holy_support_resolution.rs`
- `tests/presentation_metadata_boundary.rs`

## Verification

cargo test --test combat_cli_shared_surface --test holy_support_affordance --test holy_support_mechanics --test holy_support_resolution

## Observability Impact

Keeps headless and CLI diagnostics as the primary failure-inspection surfaces for registry-owned validation output, with deterministic text snapshots suitable for future grep/diff checks.
