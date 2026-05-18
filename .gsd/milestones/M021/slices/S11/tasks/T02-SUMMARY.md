---
id: T02
parent: S11
milestone: M021
key_files:
  - src/combat/preview.rs
  - src/ui/combat_panel.rs
  - src/windowed.rs
  - src/combat/turn_order.rs
  - src/lib.rs
  - tests/windowed_preview_cache.rs
key_decisions:
  - Shared preview summaries are cached only after a successful shared-preview refresh; the UI renders them only when actor/kind/target still match the cached context.
  - The windowed panel now resolves its skill book from the existing asset collection rather than threading one more handle resource, avoiding Bevy's system-arity limit.
duration: 
verification_result: passed
completed_at: 2026-05-17T07:33:37.249Z
blocker_discovered: false
---

# T02: Added a cached windowed preview bridge that refreshes combat damage estimates from the shared preview stream.

**Added a cached windowed preview bridge that refreshes combat damage estimates from the shared preview stream.**

## What Happened

Implemented a `PreviewDamageCache` resource plus an exclusive refresh system in the windowed UI layer. The bridge snapshots the active actor and pending action, resolves the preview target from the action affordance, runs `query_skill_preview`, and collapses the returned `Intent::DealDamage` stream into a cached numeric summary. The egui combat panel now reads that cache for action and target hover text instead of predicting damage inline, and the windowed app registers the cache/resource bridge on startup. I also exposed the UI module from the library crate so the focused integration test can exercise the real bridge path, and made `TurnOrder` clonable so the exclusive refresh system can snapshot it before querying the world mutably.

## Verification

Verified with `cargo test --features windowed --test windowed_preview_cache --no-run` and `cargo test --features windowed --test windowed_preview_cache`. The test asserts that the cached preview summary matches the shared preview-stream summary and that the cache remains unchanged when preview refresh is unavailable.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --features windowed --test windowed_preview_cache --no-run` | 0 | ✅ pass | 6483ms |
| 2 | `cargo test --features windowed --test windowed_preview_cache` | 0 | ✅ pass | 226ms |

## Deviations

Dropped the skill-book handle param from the egui panel and resolved the active skill book from the asset collection instead. This kept the panel system under Bevy's system-arity limit after adding the preview cache resource.

## Known Issues

None.

## Files Created/Modified

- `src/combat/preview.rs`
- `src/ui/combat_panel.rs`
- `src/windowed.rs`
- `src/combat/turn_order.rs`
- `src/lib.rs`
- `tests/windowed_preview_cache.rs`
