---
estimated_steps: 14
estimated_files: 2
skills_used: []
---

# T05: CombatPlugin::finish validator hook + slice verification (grep gates, headless+windowed, full suite)

Why: closes the slice with the `App::finish()` seam and the full demo-closure verification. The validator hook makes S05+ fail-fast on dangling references; the verification confirms every roadmap success criterion for S02 is green now.

Do:
1. Edit `src/combat/plugin.rs`:
   - Add `fn finish(&self, app: &mut App)` to `impl Plugin for CombatPlugin` that iterates a `Resource<TimelineLibrary>` (or simply a `Vec<CompiledTimeline>` pulled from `ExtRegistries` — pick the simpler shape; if no library Resource exists yet, the finish hook is a no-op that just compiles and reserves the seam, documented with a `///` referencing S05 wire-up). If using a Resource, define `pub struct TimelineLibrary { pub timelines: Vec<CompiledTimeline> }` in `timeline.rs` with `Resource` derive (it's bevy::prelude::Resource — allowed in api/ per S01 precedent in `registry.rs`).
   - On any validation failure, `panic!` with the aggregated error list (fail-fast at boot is the documented intent — F8).
2. Add a tiny inline `#[cfg(test)]` test in `plugin.rs` (or an integration test) that constructs an `App`, inserts a deliberately-broken `TimelineLibrary` if the library Resource exists, and asserts `app.finish()` panics. If the library shape was deferred, skip this and document in T05's inputs why.
3. Run the slice-verification battery and capture evidence in the task summary:
   - `cargo check` — exit 0, no NEW warnings (compare against S01 baseline).
   - `cargo check --features windowed` — same.
   - `cargo test timeline_validate_typo timeline_onturnstart_kills timeline_chain_bolt_port` — all green.
   - `cargo test` — full suite 0 failures.
   - Grep gates: `rg 'use bevy::winit|use bevy::render|use bevy_egui' src/combat/api/` → 0; `rg 'TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|KitsuneGrace' src/combat/api/` → 0; `rg 'pub fn validate_timeline_refs' src/combat/api/timeline.rs` → 1; `rg 'pub struct BeatRunner' src/combat/api/runner.rs` → 1; `rg 'fn finish' src/combat/plugin.rs` → 1.
4. If any new structural decision was made (e.g. SkillCtx state-borrow shape `&'a World` vs SystemParam aggregate, or TimelineLibrary Resource shape), call `gsd_save_decision` for each before completing.

Done-when: all grep gates green, both cargo check invocations exit 0, full `cargo test` 0 failures, three demo-gate tests named in the success criteria green. Evidence captured in the task SUMMARY with command, exitCode, verdict for each verification.

## Inputs

- `src/combat/plugin.rs`
- `src/combat/api/timeline.rs`
- `src/combat/api/runner.rs`
- `src/combat/api/registry.rs`
- `src/combat/api/skill_ctx.rs`
- `.gsd/milestones/M021/slices/S01/S01-SUMMARY.md`
- `.gsd/milestones/M021/M021-ROADMAP.md`

## Expected Output

- `src/combat/plugin.rs`
- `src/combat/api/timeline.rs`

## Verification

cargo check && cargo check --features windowed && cargo test && rg 'use bevy::winit|use bevy::render|use bevy_egui' src/combat/api/ ; rg 'TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|KitsuneGrace' src/combat/api/ ; rg 'pub fn validate_timeline_refs' src/combat/api/timeline.rs && rg 'pub struct BeatRunner' src/combat/api/runner.rs && rg 'fn finish' src/combat/plugin.rs

## Observability Impact

None — validator panics at App::finish() on broken timelines (fail-fast boot, F8); no runtime telemetry added.
