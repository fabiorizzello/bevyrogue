---
id: T03
parent: S09
milestone: M006
key_files:
  - src/warn_once.rs
  - src/lib.rs
  - src/animation/registry.rs
key_decisions:
  - Placed the util at lib top level (src/warn_once.rs, pub mod warn_once) rather than under animation/ since windowed consumers (S08/S11/S12/S13/S14) must reuse it engine-generically
  - Modeled the API as should_warn(key) -> bool to mirror HashSet::insert so the existing call-site guard pattern (`if !built && warned.should_warn(id)`) reads identically
  - Added has_warned/clear beyond the strict dedup need to give the shared util a testable, inspectable surface for downstream cue/verb-miss consumers
duration: 
verification_result: passed
completed_at: 2026-05-27T11:36:30.558Z
blocker_discovered: false
---

# T03: Extracted S06's inline warn-once dedup into a generic lib util WarnOnce<K> and repointed the animation registry to it

**Extracted S06's inline warn-once dedup into a generic lib util WarnOnce<K> and repointed the animation registry to it**

## What Happened

Promoted the inline `Local<HashSet<AssetId<AnimGraph>>>` warn-once dedup in src/animation/registry.rs into a generic, reusable util. Added `src/warn_once.rs` exposing `WarnOnce<K: Eq + Hash>` with `should_warn(key) -> bool` (mirrors HashSet::insert semantics: true on first sight, false thereafter), plus `has_warned` and `clear` for inspection/reset. Registered it as `pub mod warn_once;` in src/lib.rs so both the animation and windowed (S08/S11/S12/S13/S14) consumers reuse one consistent surface instead of re-implementing the pattern. Repointed populate_graph_registries to `Local<WarnOnce<AssetId<AnimGraph>>>` and `warned.should_warn(asset_id)`, then dropped the now-unused HashSet import (kept HashMap) to stay warnings-clean. Behavior is identical: an unbuildable configured graph still logs exactly once per asset id. Added two short unit tests in the module (per-key dedup; has_warned/clear lifecycle).

## Verification

cargo test (headless) — all suites green incl. 2 new warn_once unit tests. cargo test --features windowed --test windowed_only — 75 passed. RUSTFLAGS="-D warnings" cargo check — clean (confirms unused-import drop). cargo test --lib warn_once — 2 passed.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test` | 0 | pass | 8202ms |
| 2 | `cargo test --features windowed --test windowed_only` | 0 | pass | 6150ms |
| 3 | `cargo test --lib warn_once` | 0 | pass | 6150ms |
| 4 | `RUSTFLAGS="-D warnings" cargo check` | 0 | pass | 20874ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `src/warn_once.rs`
- `src/lib.rs`
- `src/animation/registry.rs`
