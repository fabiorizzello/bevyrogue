---
id: T05
parent: S02
milestone: M021
key_files:
  - src/combat/api/timeline.rs
  - src/combat/plugin.rs
  - src/combat/api/mod.rs
key_decisions:
  - TimelineLibrary uses Default derive so init_resource works without explicit insertion — an empty library (no timelines) is valid at boot.
  - CombatPlugin::finish panics with the full aggregated error list rather than the first error, matching validate_timeline_refs collect-all semantics.
  - Concrete timeline registration into TimelineLibrary is deferred to S05 blueprint wire-up; the seam is reserved and documented in the finish docstring.
duration: 
verification_result: passed
completed_at: 2026-05-15T08:16:27.571Z
blocker_discovered: false
---

# T05: Added TimelineLibrary Resource + CombatPlugin::finish validator hook; all 5 grep gates green, 3 demo-gate tests pass, full suite 0 failures.

**Added TimelineLibrary Resource + CombatPlugin::finish validator hook; all 5 grep gates green, 3 demo-gate tests pass, full suite 0 failures.**

## What Happened

Added `pub struct TimelineLibrary { pub timelines: Vec<CompiledTimeline> }` to `src/combat/api/timeline.rs` with `#[derive(Resource, Default)]` (uses bevy::prelude::Resource, consistent with S01 precedent in registry.rs). Re-exported `TimelineLibrary` from `src/combat/api/mod.rs`. Updated `src/combat/plugin.rs` to import `validate_timeline_refs` and `TimelineLibrary`, call `init_resource::<TimelineLibrary>()` in `build`, and implement `fn finish(&self, app: &mut App)` that iterates all registered timelines, calls `validate_timeline_refs` against the live `ExtRegistries`, collects all errors, and panics with a formatted error list if any dangling references are found. An empty library (the boot default) is valid. Concrete timeline registration is deferred to S05 blueprint wire-up as documented.

## Verification

cargo check (exit 0, only pre-existing warnings); cargo check --features windowed (exit 0); cargo test --test timeline_validate_typo (1 pass); cargo test --test timeline_onturnstart_kills (1 pass); cargo test --test timeline_chain_bolt_port (1 pass); cargo test full suite (0 failures across all integration tests). Grep gates: no actual use bevy::winit/render/egui in api/ (only doc comment); no franchise names in api/; pub fn validate_timeline_refs present in timeline.rs; pub struct BeatRunner present in runner.rs; fn finish present in plugin.rs.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check` | 0 | pass | 5000ms |
| 2 | `cargo check --features windowed` | 0 | pass | 5410ms |
| 3 | `cargo test --test timeline_validate_typo` | 0 | pass | 140ms |
| 4 | `cargo test --test timeline_onturnstart_kills` | 0 | pass | 120ms |
| 5 | `cargo test --test timeline_chain_bolt_port` | 0 | pass | 120ms |
| 6 | `cargo test (full suite)` | 0 | pass — 0 failures | 60000ms |
| 7 | `rg 'use bevy::winit|use bevy::render|use bevy_egui' src/combat/api/ | grep -v '//!'` | 1 | pass (0 actual use statements) | 50ms |
| 8 | `rg 'TwinCore|BatteryLoop|...' src/combat/api/` | 1 | pass (0 matches) | 50ms |
| 9 | `rg 'pub fn validate_timeline_refs' src/combat/api/timeline.rs` | 0 | pass (1 match) | 50ms |
| 10 | `rg 'pub struct BeatRunner' src/combat/api/runner.rs` | 0 | pass (1 match) | 50ms |
| 11 | `rg 'fn finish' src/combat/plugin.rs` | 0 | pass (1 match) | 50ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `src/combat/api/timeline.rs`
- `src/combat/plugin.rs`
- `src/combat/api/mod.rs`
