---
id: T03
parent: S14
milestone: M021
key_files:
  - tests/add_new_digimon_isolation.rs
key_decisions:
  - Keep the add-new-digimon claim at the extension boundary: existing units remain optional on blueprint metadata, and new blueprint owners must register explicitly.
  - Use a git-tracked integration test rather than shared runtime edits to evidence the add-new-digimon isolation boundary.
duration: 
verification_result: passed
completed_at: 2026-05-17T13:39:11.537Z
blocker_discovered: false
---

# T03: Added a git-tracked add-new-digimon isolation regression proving roster metadata defaults stay optional and blueprint dispatch remains owner-keyed.

**Added a git-tracked add-new-digimon isolation regression proving roster metadata defaults stay optional and blueprint dispatch remains owner-keyed.**

## What Happened

T03 closed the remaining extension-boundary proof by turning the implicit add-new-digimon claim into an explicit regression in tests/add_new_digimon_isolation.rs. The new proof exercises two observable boundaries: existing roster entries still round-trip with empty blueprint metadata by default, and blueprint signal dispatch rejects unknown owners instead of leaking into shared kernel names. The test also anchors the intended owner-keyed path for an existing blueprint skill so future additions stay confined to blueprint-owned seams. No shared runtime code needed changes; the work narrowed the milestone claim to what the codebase actually proves today and documented that boundary with a durable test.

## Verification

Fresh verification in this session passed: cargo test --test add_new_digimon_isolation -- --nocapture, cargo test -- --nocapture add_new_digimon, and cargo test -- --nocapture blueprint all exited 0. The focused test reports showed 3/3 passing for add_new_digimon_isolation, the add_new_digimon filter surfaced the new regression plus existing blueprint-related proofs, and the blueprint filter remained green with the registry/metadata boundary checks intact.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test add_new_digimon_isolation -- --nocapture` | 0 | ✅ pass | 0ms |
| 2 | `cargo test -- --nocapture add_new_digimon` | 0 | ✅ pass | 0ms |
| 3 | `cargo test -- --nocapture blueprint` | 0 | ✅ pass | 0ms |

## Deviations

Instead of editing shared combat code, the task narrowed the claim to a truthful boundary proof and captured it as a dedicated integration test.

## Known Issues

The broad `cargo test -- --nocapture blueprint` run still emits pre-existing warnings and one expected panic in `tests/passive_kitsune_grace.rs` (the test is designed to assert the debug panic path), but the targeted verification exits 0.

## Files Created/Modified

- `tests/add_new_digimon_isolation.rs`
