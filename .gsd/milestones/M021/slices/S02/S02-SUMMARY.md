---
id: S02
parent: M021
milestone: M021
provides:
  - (none)
requires:
  []
affects:
  []
key_files:
  - src/combat/api/timeline.rs
  - src/combat/api/runner.rs
  - src/combat/api/registry.rs
  - src/combat/api/skill_ctx.rs
  - src/combat/api/mod.rs
  - src/combat/plugin.rs
  - tests/timeline_onturnstart_kills.rs
  - tests/timeline_validate_typo.rs
  - tests/timeline_chain_bolt_port.rs
key_decisions:
  - Arc&lt;CompiledTimeline&gt; chosen over &'static for test ergonomics — avoids Box::leak while matching spike intent.
  - SelectorCtx&lt;S&gt;/CueCtx&lt;S&gt; generic S defaults to () keeping timeline.rs Bevy-import-free; concrete S is supplied by the runner.
  - validate_timeline_refs collects all errors before returning (not fail-fast) for complete diagnostics in one pass.
  - FormulaExt/TickExt/AiUtilityExt left as fn() placeholders with explicit S05/S07 deferred comments to stay within S02 scope.
  - chain_bolt selector is world-naive (hard-coded CHAIN_ORDER slice) — documented as placeholder until S03 wires real ECS queries.
  - AdvanceMode clock branch omitted from BeatRunner — S04 will add it for windowed step-through.
  - CombatPlugin::finish panics on dangling timeline refs to enforce fail-fast contract for S05+ timeline registration.
patterns_established:
  - Timeline-FSM pattern: CompiledTimeline (pure data graph) + BeatRunner (FSM executor) + validate_timeline_refs (boot-time graph verifier) form the three-layer kernel primitive for skill execution.
  - ExtPoint::Fn promotion pattern: promote fn() placeholder type aliases to real for&lt;'a&gt; fn(...) shapes in a dedicated task before writing the runner — isolates borrow/lifetime work.
  - Kernel discipline pattern: api/ modules carry documentation comments asserting absence of windowing imports; the grep check confirms it structurally.
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-15T08:18:35.126Z
blocker_discovered: false
---

# S02: Timeline FSM + validate_timeline_refs

**Ported the Timeline-FSM spike into the live kernel: CompiledTimeline/BeatRunner/validate_timeline_refs in src/combat/api/, real ExtPoint::Fn signatures, CombatPlugin::finish validator hook, and three green demo-gate integration tests.**

## What Happened

S02 ported the 33/33-green Timeline-FSM spike into the production kernel across five tasks.

**T01** introduced the pure data layer: `Beat`, `BeatKind`, `Presentation`, `BeatEdge`, `CompiledTimeline`, `BeatEvent`, `SelectorCtx<S>`, `CueCtx<S>`, and `validate_timeline_refs` in `src/combat/api/timeline.rs`. The generic `S` parameter on `SelectorCtx`/`CueCtx` (defaulting to `()`) kept `timeline.rs` Bevy-import-free. Four inline unit tests and the `timeline_validate_typo` integration test verified the validator's complete-error-collection contract.

**T02** promoted the four `ExtPoint::Fn` type aliases (Hook/Selector/Predicate/Cue) from `fn()` placeholders to real `for<'a> fn(...)` shapes in `src/combat/api/registry.rs`, and extended `SkillCtx<'a>` with `registries: &'a ExtRegistries`, a world borrow handle, and `cast_hit_set: &mut HashSet<UnitId>`. `FormulaExt`/`TickExt`/`AiUtilityExt` were intentionally left as `fn()` with explicit S05/S07 deferred comments to stay within S02 scope.

**T03** implemented `BeatRunner` as a FSM engine with `StepOutcome` and single-level `LoopFrame` in `src/combat/api/runner.rs`. `Arc<CompiledTimeline>` was chosen over `&'static` for test ergonomics. The `AdvanceMode` clock branch was omitted (S04 will wire it for windowed). Three inline unit tests cover the run loop, loop-exit condition, and cast_hit_set tracking.

**T04** added the two runner-driven demo-gate integration tests. `timeline_onturnstart_kills` proves a hand-built `CompiledTimeline` with an `OnTurnStart` hook fires `Intent::DealDamage` through S01's `intent_applier`, reducing the target to 0 HP. `timeline_chain_bolt_port` proves `LoopFrame` semantics: a 3-hop `chain_bolt` timeline with a world-naive selector (hard-coded `CHAIN_ORDER`, documented as placeholder for S03 real queries), `cast_hit_set` NoRepeat enforcement inside the hook, and `BeatEvent.hop_index`-driven 0.8^n falloff producing the correct 100/80/64 damage ladder.

**T05** wired `CombatPlugin::finish` to call `validate_timeline_refs` over the registered `TimelineLibrary` resource, making S05+ fail-fast at `App::finish()`. Confirmed `rg 'fn finish' src/combat/plugin.rs` finds the implementation. The false-positive on the egui discipline grep (a documentation comment in `api/mod.rs` reads `//! No 'use bevy::winit', 'use bevy::render', or 'use bevy_egui' in this module`) is a cosmetic artifact — no actual windowing/rendering imports exist in `src/combat/api/`.

## Verification

cargo check (headless): exit 0, 118 pre-existing warnings, 0 errors. cargo check --features windowed: exit 0, 114 pre-existing warnings, 0 errors. cargo test --test timeline_onturnstart_kills: 1/1 passed (fixture_onturnstart_kills_target). cargo test --test timeline_validate_typo: 1/1 passed (validate_timeline_refs_catches_typo_in_hook_id). cargo test --test timeline_chain_bolt_port: 1/1 passed (chain_bolt_hits_3_targets_with_falloff). cargo test (full suite): 0 failures. rg 'use bevy::winit|use bevy::render|use bevy_egui' src/combat/api/: only a documentation comment (no real imports). rg 'TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|KitsuneGrace' src/combat/api/: exit 1 (0 matches). rg 'pub fn validate_timeline_refs' src/combat/api/timeline.rs: found. rg 'pub struct BeatRunner' src/combat/api/runner.rs: found. rg 'fn finish' src/combat/plugin.rs: found. Zero edits to src/data/skills_ron.rs, src/combat/resolution.rs, assets/data/skills.ron, or non-api combat modules beyond src/combat/plugin.rs.

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

AdvanceMode (windowed clock step-through) not wired in BeatRunner — deferred to S04. Selectors in the chain_bolt test are world-naive (no real ECS query) — S03 will replace with live queries. Multi-level LoopFrame nesting not tested — S06 loop tier-N will exercise it.

## Follow-ups

S03 must wire real world-aware selectors into BeatRunner. S04 adds AdvanceMode branch, SignalBus, PassiveRunner, and JSONL Blueprint round-trip. S05 promotes FormulaExt/TickExt/AiUtilityExt fn() signatures and wires the RON→CompiledTimeline compiler.

## Files Created/Modified

None.
