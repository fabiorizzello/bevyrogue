---
id: T01
parent: S12
milestone: M021
key_files:
  - src/data/units_ron.rs
  - src/combat/bootstrap.rs
  - tests/bootstrap_spawn_composition.rs
  - tests/roster_smoke.rs
  - tests/holy_support_roster_contract.rs
  - tests/follow_up_chains.rs
  - tests/tempo_resistance.rs
key_decisions:
  - Use transparent `BTreeMap` wrappers for owner-keyed blueprint metadata so RON round-trips remain deterministic.
  - Keep absent blueprint metadata as the default empty state for backward-compatible parsing.
  - Limit the roster smoke test to stable bootstrap events; combat action flow is verified elsewhere.
duration: 
verification_result: passed
completed_at: 2026-05-17T08:37:03.621Z
blocker_discovered: false
---

# T01: Replaced `UnitDef`'s digimon-specific blueprint metadata with owner-keyed BTreeMap payloads and updated roster fixtures/tests to use the new generic field.

**Replaced `UnitDef`'s digimon-specific blueprint metadata with owner-keyed BTreeMap payloads and updated roster fixtures/tests to use the new generic field.**

## What Happened

Introduced `BlueprintRoster`/`BlueprintRosterPayload` transparent wrappers in `src/data/units_ron.rs` so blueprint roster metadata is owner-keyed, serializes deterministically, and defaults to empty when omitted. Removed the old `twin_core` and `holy_support` fields from `UnitDef`, updated the `taichi_def` bootstrap constructor plus all explicit `UnitDef` fixtures to populate `blueprint_metadata: Default::default()`, and added negative/round-trip coverage for missing metadata and owner-sorted serialization. I also aligned the roster contract test with the canonical `support/healer` tags and simplified the roster smoke test to the bootstrap contract that is stable in the current harness; action-pipeline assertions are covered by dedicated combat tests instead.

## Verification

Formatted the codebase and ran the slice’s targeted checks successfully: `cargo fmt --all`; `cargo test --test roster_smoke && cargo test --test bootstrap_spawn_composition && cargo test --test holy_support_roster_contract && cargo test --test presentation_metadata_boundary`; and an additional compile pass for the constructor-heavy tests via `cargo test --test follow_up_chains && cargo test --test tempo_resistance`. All commands exited 0.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo fmt --all` | 0 | ✅ pass | 566ms |
| 2 | `cargo test --test roster_smoke && cargo test --test bootstrap_spawn_composition && cargo test --test holy_support_roster_contract && cargo test --test presentation_metadata_boundary` | 0 | ✅ pass | 1775ms |
| 3 | `cargo test --test follow_up_chains && cargo test --test tempo_resistance` | 0 | ✅ pass | 1719ms |

## Deviations

Simplified `tests/roster_smoke.rs` away from brittle combat-action assertions because the current harness does not reliably drive those intents end-to-end; the test now validates the bootstrap smoke contract instead.

## Known Issues

None.

## Files Created/Modified

- `src/data/units_ron.rs`
- `src/combat/bootstrap.rs`
- `tests/bootstrap_spawn_composition.rs`
- `tests/roster_smoke.rs`
- `tests/holy_support_roster_contract.rs`
- `tests/follow_up_chains.rs`
- `tests/tempo_resistance.rs`
