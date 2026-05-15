# S05: Built-in extension fns + RON → CompiledTimeline compiler

**Goal:** Introduce built-in extension functions plus a SkillBook-to-CompiledTimeline compiler, then prove real canon skills can execute through the kernel timeline path with strict typo failure before runtime.
**Demo:** Tohakken + Petit Thunder via CompiledTimeline; typo→errore boot.

## Must-Haves

- `assets/data/skills.ron` carries timeline-backed canon entries for `petit_thunder` and the live Renamon ult id (`renamon_ult`, satisfying the roadmap's Tohakken proof under the current asset id).
- `TimelineLibrary` is populated from validated `SkillBook` timelines, and dangling hook/selector/predicate references in RON panic during startup/load with skill + site context before an encounter can proceed.
- Production execution can dispatch a timeline-backed skill through `BeatRunner` while preserving the legacy effects path for unmigrated skills.
- Kernel intent application covers the exact effect surface needed by this slice: damage, toughness break, status apply, delay-turn, ally buff, and blueprint signal sequencing.
- Asset-backed integration tests prove Petit Thunder and Renamon ult/Tohakken semantics via `CompiledTimeline`, and both `cargo test` and `cargo check` stay green for the touched path.

## Proof Level

- This slice proves: integration — real runtime required: yes; human/UAT required: no. This slice is done only when asset-backed skills compile into `CompiledTimeline`, runtime dispatch uses the kernel path for those skills, typo failures are caught before gameplay, and end-to-end tests assert both intent/effect semantics and failure-path diagnostics.

## Integration Closure

Consumes `src/data/skills_ron.rs`, `src/data/mod.rs`, `src/combat/api/{timeline,registry,runner,applier}.rs`, `src/combat/{resolution,state,turn_system/pipeline}.rs`, and `assets/data/skills.ron`. New wiring is the compiler/load bridge into `TimelineLibrary`, built-in registry registration at combat bootstrap, and a production dispatch branch that routes timeline-backed skills into `BeatRunner` while legacy effect execution remains available for the rest of the roster. After this slice, S06 can migrate the rest of the active canon onto the already-proven compiler/runtime path instead of inventing more kernel plumbing.

## Verification

- Compile/validation failures must surface the offending skill id plus beat/edge site during SkillBook load. Successful runtime execution must remain visible through existing `CombatEvent` surfaces (`OnDamageDealt`, `OnStatusApplied`, `OnBreak`, `DelayTurn`, `OnKernelTransition`) so future agents can distinguish compiler bugs from applier bugs without instrumenting the path again.

## Tasks

- [x] **T01: Make CompiledTimeline asset-safe and built-in-hook-capable** `est:3h`
  Skills used: bevy, rust-best-practices, rust-testing, verify-before-complete.
  - Files: `src/combat/api/timeline.rs`, `src/combat/api/registry.rs`, `src/combat/api/builtins.rs`, `src/combat/api/mod.rs`, `src/combat/plugin.rs`, `tests/compiled_timeline_builtin_validation.rs`
  - Verify: cargo test --test compiled_timeline_builtin_validation

- [x] **T02: Compile SkillBook timelines on load and fail fast on bad refs** `est:4h`
  Skills used: bevy, rust-best-practices, rust-testing, verify-before-complete.
  - Files: `src/data/skills_ron.rs`, `src/data/skill_timeline.rs`, `src/data/mod.rs`, `src/combat/api/timeline.rs`, `tests/compiled_timeline_boot_validation.rs`, `assets/data/skills.ron`
  - Verify: cargo test --test compiled_timeline_boot_validation

- [x] **T03: Route timeline-backed actions through BeatRunner and cover required intent variants** `est:5h`
  Skills used: bevy, rust-best-practices, rust-testing, verify-before-complete.
  - Files: `src/combat/state.rs`, `src/combat/resolution.rs`, `src/combat/turn_system/pipeline.rs`, `src/combat/api/applier.rs`, `src/combat/api/runner.rs`, `tests/compiled_timeline_runtime_dispatch.rs`
  - Verify: cargo test --test compiled_timeline_runtime_dispatch

- [x] **T04: Port Petit Thunder and Renamon ult data to timeline-backed canon tests** `est:4h`
  Skills used: bevy, rust-best-practices, rust-testing, verify-before-complete.
  - Files: `assets/data/skills.ron`, `tests/compiled_timeline_petit_thunder.rs`, `tests/compiled_timeline_tohakken.rs`
  - Verify: cargo test --test compiled_timeline_petit_thunder --test compiled_timeline_tohakken

## Files Likely Touched

- src/combat/api/timeline.rs
- src/combat/api/registry.rs
- src/combat/api/builtins.rs
- src/combat/api/mod.rs
- src/combat/plugin.rs
- tests/compiled_timeline_builtin_validation.rs
- src/data/skills_ron.rs
- src/data/skill_timeline.rs
- src/data/mod.rs
- tests/compiled_timeline_boot_validation.rs
- assets/data/skills.ron
- src/combat/state.rs
- src/combat/resolution.rs
- src/combat/turn_system/pipeline.rs
- src/combat/api/applier.rs
- src/combat/api/runner.rs
- tests/compiled_timeline_runtime_dispatch.rs
- tests/compiled_timeline_petit_thunder.rs
- tests/compiled_timeline_tohakken.rs
