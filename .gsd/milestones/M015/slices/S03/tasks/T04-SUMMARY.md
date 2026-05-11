---
id: T04
parent: S03
milestone: M015
key_files:
  - src/combat/blueprints/mod.rs
  - src/combat/blueprints/patamon.rs
  - src/combat/mod.rs
  - src/combat/turn_system/pipeline.rs
  - tests/patamon_blueprint_seam.rs
  - tests/holy_support_resolution.rs
  - tests/holy_support_roster_contract.rs
key_decisions:
  - Chose a minimal `blueprints::transitions_for_action(&ResolvedAction)` seam so generic routing owns only wrapper dispatch and Patamon owns signal interpretation.
  - Dispatched blueprint transitions only after `ResolutionOutcome::succeeded`, preserving `resolution.rs` as effect-only and preventing failed actions from mutating Holy Support state.
duration: 
verification_result: passed
completed_at: 2026-05-08T16:16:12.679Z
blocker_discovered: false
---

# T04: Seeded Patamon blueprint dispatch so its RON custom signal now becomes a canonical Holy Support kernel transition, event, state update, and snapshot signal.

**Seeded Patamon blueprint dispatch so its RON custom signal now becomes a canonical Holy Support kernel transition, event, state update, and snapshot signal.**

## What Happened

Added the first per-Digimon blueprint seam under `src/combat/blueprints`. The generic router scans only `ResolvedAction.custom_signals` and delegates Patamon signals to `blueprints::patamon`, where `BuildHolySupportGrace` becomes the canonical `CombatKernelTransition::HolySupport(HolySupportTransition::build_grace(amount))`. Wired the shared action pipeline to emit blueprint transitions through `emit_kernel_transition` after successful actions in both self-target and normal target branches, so registry fanout and `OnKernelTransition` events stay canonical. Updated Patamon/Holy Support tests to prove no-signal negative behavior, exact Patamon transition mapping, live pipeline event emission, applied `HolySupportState`, and `format_validation_snapshot` output. Replaced stale direct HolySupport effect/roster expectations with the custom-signal blueprint contract.

## Verification

Ran the required scoped Rust tests after formatting: `cargo test --test patamon_blueprint_seam --test holy_support_resolution --test holy_support_roster_contract` passed with exit code 0. Also ran the slice authority audit `python3 scripts/verify_combat_authority_audit.py`, which printed `Combat authority audit verification passed.` and exited 0. LSP diagnostics were clean before formatting, but the LSP formatter later failed because `rust-analyzer` is unavailable in the pinned nightly toolchain; used `cargo fmt` and reran verification afterward.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test patamon_blueprint_seam --test holy_support_resolution --test holy_support_roster_contract` | 0 | ✅ pass | 545ms |
| 2 | `python3 scripts/verify_combat_authority_audit.py` | 0 | ✅ pass | 19ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/combat/blueprints/mod.rs`
- `src/combat/blueprints/patamon.rs`
- `src/combat/mod.rs`
- `src/combat/turn_system/pipeline.rs`
- `tests/patamon_blueprint_seam.rs`
- `tests/holy_support_resolution.rs`
- `tests/holy_support_roster_contract.rs`
