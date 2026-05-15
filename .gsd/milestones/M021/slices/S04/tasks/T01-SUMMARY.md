---
id: T01
parent: S04
milestone: M021
key_files:
  - src/combat/api/signal.rs
  - src/combat/api/intent.rs
  - src/combat/api/mod.rs
  - src/combat/plugin.rs
key_decisions:
  - D-S04-SIGNAL-TAXONOMY-RESOURCE: Registered SignalTaxonomy as a Bevy resource for validation.
duration: 
verification_result: passed
completed_at: 2026-05-15T10:33:24.360Z
blocker_discovered: false
---

# T01: Implemented typed SignalBus, SignalPayload, and SignalTaxonomy for the reactive layer.

**Implemented typed SignalBus, SignalPayload, and SignalTaxonomy for the reactive layer.**

## What Happened

Rewrote the SignalBus from a placeholder into a working VecDeque-based queue. Defined Signal and SignalPayload enums with serde support for JSONL logging. Implemented the SignalTaxonomy resource to allow registration and validation of (owner, name) signal pairs. Updated Intent::BlueprintSignal to use the new typed payload and explicit source field. Wired the new resources into the CombatPlugin. Verified with 3 new unit tests and clean cargo checks.

## Verification

Ran 3 unit tests in signal.rs covering push/drain order, taxonomy registration, and serde round-trip. Ran cargo check for both headless and windowed features. Verified that BlueprintSignal only exists in intent.rs definition.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --lib -- combat::api::signal` | 0 | ✅ pass | 4700ms |
| 2 | `cargo check && cargo check --features windowed` | 0 | ✅ pass | 5000ms |
| 3 | `rg "BlueprintSignal" src/ tests/` | 0 | ✅ pass | 100ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/combat/api/signal.rs`
- `src/combat/api/intent.rs`
- `src/combat/api/mod.rs`
- `src/combat/plugin.rs`
