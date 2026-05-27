---
id: T01
parent: S06
milestone: M006
key_files:
  - /home/fabio/dev/bevyrogue/tests/animation/registry_starvation.rs
  - /home/fabio/dev/bevyrogue/tests/animation.rs
key_decisions:
  - Used asset_server.load() for path registration (synchronous, no I/O) rather than trying to mock paths or use UUID handles, because get_path() only works for indexed handles
  - Included TaskPoolPlugin+AssetPlugin+RonAssetPlugin<AnimGraph> to wire the shared index allocator between AssetServer and Assets<AnimGraph>
  - Parsed graphs from inline RON constants to keep the test deterministic and self-contained (R004 compliance)
  - Queued skill event BEFORE stance event to match the bug: skill fires first, return exits loop, stance is starved
duration: 
verification_result: mixed
completed_at: 2026-05-27T06:51:57.434Z
blocker_discovered: false
---

# T01: Added failing headless test that reproduces the single-graph-per-batch starvation bug in populate_graph_registries

**Added failing headless test that reproduces the single-graph-per-batch starvation bug in populate_graph_registries**

## What Happened

Read registry.rs to understand the bug: `populate_graph_registries` has `return` statements at lines 275 and 279 that exit the entire function after the first matching graph event, starving subsequent events in the same batch.

Read existing animation tests to understand patterns — all were pure unit tests without Bevy App. Investigated how to drive the system headlessly through its real surface (not mocking).

Key design challenge: `populate_graph_registries` calls `asset_server.get_path(asset_id)` to classify events, which only returns a path for assets with indexed IDs (loaded via `asset_server.load()`). Assets inserted via `Assets::add()` get UUID handles for which `get_path` always returns None.

Solution: `asset_server.load()` registers the path→index mapping **synchronously** in `AssetInfos` even before any file I/O begins. The shared `Arc<AssetIndexAllocator>` between `AssetServer` and `Assets<AnimGraph>` (wired by `register_asset` inside `init_asset`) means the indexed handle from `load()` is valid for direct insertion into `Assets<AnimGraph>` via `insert(handle.id(), graph)`. This lets the test inject parsed inline graphs under server-registered paths without any filesystem reads.

Test structure: App with `TaskPoolPlugin + AssetPlugin + RonAssetPlugin<AnimGraph>` (hot-reload disabled). Call `asset_server.load(SKILL_ASSET_PATH)` and `asset_server.load(STANCE_ASSET_PATH)` to register paths, insert inline-parsed graphs via the returned handles, set `AnimationGraphHandles`, configure `SkillGraphPaths`/`StanceGraphPaths`, then write both `AssetEvent::LoadedWithDependencies` messages in the same frame before calling `app.update()`.

Result: skill registry assertion passes (skill graph inserted correctly), stance registry assertion fails (stance event never processed because `return` on line 275 exited the loop) — exactly reproducing the starvation bug. 119 existing tests still pass.

## Verification

Ran `cargo test --test animation registry_starvation` — new test FAILS red with the stance registry assertion, proving starvation. Ran `cargo test --test animation` — 119 existing tests pass, only the new starvation test fails. Suite compiles clean.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test animation registry_starvation` | 101 | NEW TEST FAILS RED: skill registry populated, stance registry empty — starvation reproduced | 1200ms |
| 2 | `cargo test --test animation` | 101 | 119 passed, 1 failed (only the new starvation test); suite compiles clean | 1000ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `/home/fabio/dev/bevyrogue/tests/animation/registry_starvation.rs`
- `/home/fabio/dev/bevyrogue/tests/animation.rs`
