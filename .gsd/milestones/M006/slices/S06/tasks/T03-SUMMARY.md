---
id: T03
parent: S06
milestone: M006
key_files:
  - src/animation/registry.rs
key_decisions:
  - Dedup per asset id via Local<HashSet<AssetId<AnimGraph>>> so warn fires once even across repeated Modified events
  - Warn only for configured handles that fail classification (genuine spawn-miss); untracked assets and not-yet-loaded assets stay silent
  - Used if/else if + built flag to detect the no-match fall-through rather than restructuring the loop
duration: 
verification_result: passed
completed_at: 2026-05-27T07:02:01.429Z
blocker_discovered: false
---

# T03: Added warn-once in populate_graph_registries when a configured graph loads but matches no path list, making spawn-misses visible

**Added warn-once in populate_graph_registries when a configured graph loads but matches no path list, making spawn-misses visible**

## What Happened

Added a `Local<HashSet<AssetId<AnimGraph>>>` dedup set to `populate_graph_registries`. After the skill/stance classification block, the loop now tracks whether a registry entry was actually built. When a graph asset event arrives for a *configured* handle (one present in `AnimationGraphHandles`) and its asset is loaded, but its path matches neither `SkillGraphPaths` nor `StanceGraphPaths`, the system emits a single `warn!` carrying the graph id and path, then records the asset id so repeated Modified events do not re-spam. Events for untracked `AnimGraph` assets and not-yet-loaded assets still `continue` silently — they are not spawn-misses. Refactored the two `if`/`continue` insert arms into an `if/else if` with a `built` flag so the no-match fall-through is detectable. On the happy path every configured handle resolves to either a skill or stance path (handles are built by chaining both path lists in `load_animation_graphs`), so `built` is always true and no warn fires — confirmed by the green headless suite. Skipped the create-gsd-extension/vfx-realtime skill activations as not relevant to this pure logging change; rust-best-practices (single owned `warn!`, no per-frame allocation, dedup via set insert returning bool) was followed.

## Verification

cargo test --test animation: 120 passed, 0 failed (incl. registry_starvation, now green from T02). cargo clippy --tests: exit 0; only registry.rs lint is the pre-existing too_many_arguments on the Bevy system (already tripped before adding the Local param) — no new warning category. Windowed Renamon/Agumon happy-path no-warn check is K001-gated and must be verified manually by a human.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test animation` | 0 | pass | 3083ms |
| 2 | `cargo clippy --tests` | 0 | pass | 4812ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `src/animation/registry.rs`
