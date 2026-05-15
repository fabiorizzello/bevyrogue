---
id: S05
parent: M021
milestone: M021
provides:
  - A proven SkillBook→CompiledTimeline→BeatRunner path for canon skills.
  - Load-time typo detection for timeline references before gameplay starts.
  - A runtime dispatch pattern that can be reused by the remaining active canon migration slices.
requires:
  - slice: S04
    provides: Renamon kitsune_grace blueprint state and JSONL round-trip proof that the compiler/runtime path can coexist with existing kernel surfaces.
affects:
  []
key_files:
  - assets/data/skills.ron
  - tests/compiled_timeline_petit_thunder.rs
  - tests/compiled_timeline_tohakken.rs
  - tests/compiled_timeline_runtime_dispatch.rs
  - src/combat/api/timeline.rs
  - src/combat/api/builtins.rs
  - src/combat/api/applier.rs
  - src/combat/api/runner.rs
key_decisions:
  - Route timeline-backed skills through BeatRunner via a deferred command instead of making the resolver exclusive.
  - Intern compiled timeline ids to 'static at dispatch time to satisfy the existing BeatRunner API.
  - Use the live asset id `renamon_ult` in the canon test while preserving the Tohakken name as design-facing documentation.
  - Assert the actual compiled-timeline combat-event stream instead of legacy custom-signal expectations.
patterns_established:
  - Compiled-timeline canon tests should validate emitted combat events and runtime semantics, not legacy signal artifacts.
  - Load-time validation must fail fast with skill and site context for dangling timeline references.
  - Timeline-backed skills can coexist with legacy skill execution behind a dispatch branch until later migration slices absorb the rest of the roster.
observability_surfaces:
  - Startup/load-time validation errors for malformed timeline references with skill and site context.
  - Combat-event stream visibility for timeline-backed execution (`OnDamageDealt`, `OnStatusApplied`, `OnBreak`, `DelayTurn`, `OnActionApplied`, `OnActionResolved`).
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-15T18:05:07.970Z
blocker_discovered: false
---

# S05: Built-in extension fns + RON → CompiledTimeline compiler

**Timeline-backed canon skills now compile from RON and execute through BeatRunner, with Petit Thunder and Renamon ult proving the kernel path and typo failures surfacing before runtime.**

## What Happened

This slice closed the compiled-timeline bridge from data to runtime for the remaining canon proofs. T01 established asset-safe compiled timelines and auto-registered built-in kernel functions in CombatPlugin, so timelines can be loaded without borrowing issues and built-in hook sites are available by default. T02 extended the SkillBook schema with optional timeline definitions, compiled those timelines during load, and made dangling hook, selector, and predicate references fail fast with skill and site context before an encounter can start. T03 routed timeline-backed skills through BeatRunner while preserving the legacy resolver path for unmigrated skills, and extended the intent applier to cover the exact effect surface needed here: damage, toughness break, status apply, delay-turn, ally buff, and blueprint state signaling. T04 then aligned the canon tests with the live Renamon ult asset id `renamon_ult`, kept the Petit Thunder coverage, and verified both skills execute through the kernel timeline path with the expected combat events and typo-proof startup behavior.

The slice’s implementation shape is intentionally incremental: production dispatch now branches on whether a skill has a compiled timeline, so the rest of the roster can keep using the legacy path until later migration slices consume the same compiler/runtime bridge. The tests also capture the important contract that runtime proof should assert the emitted combat-event stream, not a legacy signal artifact, which keeps the slice aligned with the kernel’s source-of-truth surfaces.

## Verification

Verified the slice with the combined timeline canon suite: `cargo test --test compiled_timeline_petit_thunder --test compiled_timeline_tohakken`. The existing task evidence also confirms the supporting targeted checks passed earlier in the slice: `cargo test --test compiled_timeline_builtin_validation`, `cargo test --test compiled_timeline_boot_validation`, and `cargo test --test compiled_timeline_runtime_dispatch -- --nocapture`. Together these checks prove that timeline assets compile, bad references fail before runtime, BeatRunner dispatches timeline-backed skills, and the Petit Thunder / live Renamon ult canon paths emit the expected combat events.

## Requirements Advanced

None.

## Requirements Validated

- R001 — Timeline-backed canon skills compile and execute through the kernel timeline path with typo failures surfacing before runtime.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

Renamon ult is validated under the live asset id `renamon_ult` while the canon test name preserves the Tohakken mapping for design intent.

## Known Limitations

Only the requested Petit Thunder and Renamon ult canon paths are proven here; the broader skill roster still uses the legacy resolver until later migration slices.

## Follow-ups

Migrate the remaining active canon onto the same compiler/runtime bridge in S06+, and continue asserting runtime semantics through combat events rather than legacy signal artifacts.

## Files Created/Modified

- `assets/data/skills.ron` — Added/updated timeline-backed canon skill data for Petit Thunder and the live Renamon ult id.
- `tests/compiled_timeline_petit_thunder.rs` — Verified Petit Thunder executes through the compiled timeline path.
- `tests/compiled_timeline_tohakken.rs` — Aligned the canon test with live `renamon_ult` data and asserted the compiled timeline event stream.
- `tests/compiled_timeline_runtime_dispatch.rs` — Proved timeline-backed runtime dispatch and legacy fallback behavior.
