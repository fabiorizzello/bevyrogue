# S14: Prove two-clock parity and extension boundaries — UAT

**Milestone:** M021
**Written:** 2026-05-17T13:41:55.431Z

# S14: Prove two-clock parity and extension boundaries — UAT

**Milestone:** M021
**Written:** 2026-05-17

## UAT Type

- UAT mode: artifact-driven
- Why this mode is sufficient: The slice goal is to document boundary evidence and regression proofs already captured in the repository, so the authoritative checks are the recorded test outputs, grep audits, and task summaries rather than a live interactive flow.

## Preconditions

- The repository contains the completed task artifacts for T01, T02, and T03.
- The focused add-new-digimon regression test is present at `tests/add_new_digimon_isolation.rs`.
- The current tree is used as the source of truth for the blueprint boundary grep audit.

## Smoke Test

- Review the slice task summaries and confirm they jointly cover parity discovery, blueprint boundary audit, and add-new-digimon isolation proof.

## Test Cases

### 1. Add-new-digimon isolation remains owner-keyed

1. Run `cargo test --test add_new_digimon_isolation -- --nocapture`.
2. Inspect the regression assertions for roster metadata defaults and unknown-owner dispatch rejection.
3. **Expected:** The focused regression passes and demonstrates that new blueprint owners must register explicitly while existing units can keep empty blueprint metadata.

### 2. Blueprint isolation is checked against the live tree

1. Run `rg "use bevy" src/combat/blueprints/`.
2. Run `rg -n "fn register\(" src/combat/blueprints/`.
3. **Expected:** The grep audit reports the current boundary truth: register seams exist, but shared blueprint modules still import Bevy directly, so the absolute no-Bevy claim is not yet satisfied.

### 3. Parity claim stays bounded to actual test discovery

1. Run `cargo test -- --nocapture windowed || true`.
2. Run `cargo test -- --nocapture parity || true`.
3. **Expected:** The filters currently match no tracked tests, so the slice records that parity is not yet proven by discovery in the current tree.

## Edge Cases

### Broad suite warnings

1. Run `cargo test -- --nocapture blueprint`.
2. **Expected:** The suite still exits 0 even if pre-existing warnings or intentionally failing-path assertions appear in individual tests that are designed to validate panic behavior.

## Failure Signals

- The focused regression fails or no longer rejects unknown blueprint owners.
- The blueprint grep audit unexpectedly stops reporting Bevy imports in shared modules without a corresponding code change explanation.
- The parity filters begin matching tests but the task summaries still claim zero discovery.

## Not Proven By This UAT

- Live runtime equivalence between HeadlessAuto and Windowed intent streams.
- Full milestone readiness for closure; that remains the job of the final closeout slice.
- Any future blueprint refactor that would eliminate Bevy imports from shared modules.

## Notes for Tester

This UAT intentionally records the truth of the current boundary state instead of overstating it. The important result is that the add-new-digimon isolation path is now evidenced in-tree, and the remaining parity/no-Bevy claims are clearly bounded by the current test discovery and grep results.
