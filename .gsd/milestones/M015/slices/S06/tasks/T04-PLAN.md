---
estimated_steps: 3
estimated_files: 6
skills_used: []
---

# T04: Finish UnitDef and duplicate-field compile repairs until no-run is green

Complete the compile baseline by fixing current `UnitDef` fixture metadata, duplicate fields, and same-class residual compile-only drift. This task owns `cargo test --no-run` exit 0. Expected executor skills: `design-an-interface`, `tdd`, `verify-before-complete`, `write-docs`.

Steps: (1) classify remaining no-run failures after T03; (2) add neutral `twin_core: Default::default()` / `holy_support: Default::default()` to stale fixtures unless metadata is under test; (3) remove duplicate fields preserving intentional values; (4) run no-run to green and record exec ID/exit code.

Must-haves: no obsolete API restoration, no `windowed` dependency, no nondeterminism. Failure modes/Q5-Q7: missing target names require file-existence check before manifest edits; source shims are avoided unless a real current contract is broken.

## Inputs

- ``docs/m015_failure_ledger.md` — T01-T03 classification and retired blocker trail.`
- ``tests/tempo_resistance.rs` — known `UnitDef` fixture drift surface.`
- ``tests/follow_up_chains.rs` — known duplicate-field and `UnitDef` drift surface.`
- ``tests/roster_smoke.rs` — known duplicate-field and `UnitDef` drift surface.`
- ``tests/combat_coherence.rs` — cross-contract fixture surface.`
- ``tests/twin_core_integration.rs` — Twin Core boundary that must not restore obsolete fields.`

## Expected Output

- ``tests/tempo_resistance.rs` — current `UnitDef` fixture shape where needed.`
- ``tests/follow_up_chains.rs` — duplicate fields removed/current fixture shape where needed.`
- ``tests/roster_smoke.rs` — duplicate fields removed/current fixture shape where needed.`
- ``tests/combat_coherence.rs` — residual compile drift repaired if exposed.`
- ``tests/twin_core_integration.rs` — residual compile drift repaired without obsolete API restoration.`
- ``docs/m015_failure_ledger.md` — `cargo test --no-run` green evidence recorded.`

## Verification

`cargo test --no-run` exits 0 via `gsd_exec`; `docs/m015_failure_ledger.md` records the green command ID/exit code.
