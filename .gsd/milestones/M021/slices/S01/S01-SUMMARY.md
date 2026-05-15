---
id: S01
parent: M021
milestone: M021
provides:
  - CombatPlugin as Bevy Plugin
  - src/combat/api/ with 7 primitive modules
  - CastId on CombatEvent
  - Intent enum 18 variants
  - Registry&lt;E&gt;+ExtRegistries Resource
  - DealDamage canary end-to-end
requires:
  []
affects:
  []
key_files:
  - src/combat/api/mod.rs
  - src/combat/api/intent.rs
  - src/combat/api/registry.rs
  - src/combat/api/signal.rs
  - src/combat/api/clock.rs
  - src/combat/api/rng.rs
  - src/combat/api/skill_ctx.rs
  - src/combat/api/applier.rs
  - src/combat/events.rs
  - src/combat/plugin.rs
  - src/lib.rs
  - src/main.rs
  - tests/intent_applier_canary.rs
  - tests/cast_id_propagation.rs
key_decisions:
  - Named cast-scoped RNG 'CastRng' (not 'CombatRng') to avoid collision with existing src/combat/rng::CombatRng
  - intent_applier uses exclusive &mut World system to avoid Bevy ParamSet aliasing complexity for multi-entity read+write
  - CombatRng::from_seed(0xDEAD_BEEF) as canonical deterministic seed in CombatPlugin::build
  - CastId::ROOT = NonZeroU32::new(1) used for pre-cast events; cast-scoped IDs start at 2+
  - BlueprintSignal payload is u64 for S01; S04 replaces with closed-enum Signal per D028
  - SetBlueprintState.key is String (not &'static str) for runtime namespacing flexibility
patterns_established:
  - api/ module pattern: all kernel primitives in src/combat/api/ with zero forbidden imports (bevy::winit/render/bevy_egui)
  - CastId propagation pattern: every CombatEvent carries cast_id; pre-cast uses ROOT; pipeline-scoped uses monotonic CastIdGen
  - Exclusive system pattern for intent_applier avoids Bevy query aliasing on multi-entity read+write
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-15T07:23:17.676Z
blocker_discovered: false
---

# S01: S01: Kernel framework primitives + CombatPlugin extract

**Extracted CombatPlugin as Bevy Plugin and introduced src/combat/api/ with 7 primitive modules (Intent+CastId, Registry&lt;E&gt;+ExtRegistries, SkillCtx, applier, SignalBus, Clock, CastRng), propagated CastId to all ~50 CombatEvent emit sites — cargo check clean headless+windowed, 0 test failures across all suites.**

## What Happened

S01 built the kernel framework foundation across 5 tasks, each verified individually before this slice closeout.

**T01 — api/ skeleton:** Created `src/combat/api/` with `intent.rs` (Intent enum 18 variants + CastId(NonZeroU32) + ROOT constant), `registry.rs` (ExtPoint trait + Registry&lt;E&gt; + ExtRegistries Resource with 7 placeholder axes), `signal.rs` (SignalBus Resource scaffold), `clock.rs` (Clock enum HeadlessAuto/Windowed), `rng.rs` (CastRng SplitMix64 deterministic, named to avoid collision with existing CombatRng). All api/ types are bevy-agnostic with zero forbidden imports. Inline unit tests for Registry lookup (hit/miss/overwrite/default-empty) and RNG (determinism, divergence, from-params) all green.

**T02 — SkillCtx + intent_applier canary:** Added `skill_ctx.rs` (SkillCtx&lt;'a&gt; + SkillCtxMode {DryRun, Execute, Preview}) and `applier.rs` (intent_applier exclusive &mut World system wired DealDamage to the existing damage formula as canary). IntentQueue Resource added. Other Intent variants log::warn! and delegate to current code paths. canary test `tests/intent_applier_canary.rs` verifies HP reduction + CombatEvent::OnDamageDealt emission. Key design: exclusive system avoids Bevy ParamSet aliasing complexity for multi-entity read+write.

**T03 — CastId propagation:** Added `cast_id: CastId` field to `CombatEvent` struct and `CastIdGen` monotonic Resource. Propagated cast_id through all ~50 emit sites: pipeline-scoped casts use a generated CastId, pre-cast events use CastId::ROOT. Updated all test pattern-matches with `..` rest. `tests/cast_id_propagation.rs` verifies: (a) events during a cast share cast_id, (b) cast-scoped ≠ ROOT, (c) pre-cast events = ROOT. JSONL logger receives cast_id as additive non-breaking field.

**T04 — CombatPlugin extract:** Moved `register_combat_kernel_runtime` into `impl Plugin for CombatPlugin`, mounting all framework Resources (ExtRegistries, SignalBus, Clock, CombatRng seeded 0xDEAD_BEEF, IntentQueue, CastIdGen) and registering intent_applier as exclusive system. Re-exported CombatPlugin from `src/lib.rs`. Updated `src/main.rs` to `.add_plugins(CombatPlugin)` removing the old direct registration call. `src/bin/combat_cli.rs` updated accordingly.

**T05 — Slice verification:** All 5 grep gates green, cargo check headless + windowed clean (only pre-existing dead-code warnings), cargo test 0 failures across all test binaries (208+209 inline + all integration suites). Two decisions not yet in DECISIONS.md were appended: intent_applier exclusive system choice, CombatRng canonical seed 0xDEAD_BEEF.

**Gate analysis:** Gate 2 (`rg 'CombatEvent \{' src/ | rg -v 'cast_id'`) surfaces false positives because all struct literals are multi-line — every instance was verified by context check to include cast_id on a subsequent line. Gate 1 matched a doc comment in api/mod.rs, not an import statement.

## Verification

1. `cargo check` (headless): exit 0, no new errors, only pre-existing dead-code warnings. ✓
2. `cargo check --features windowed`: exit 0, clean. ✓
3. `cargo test`: 0 failures across all test binaries — 208+209 lib tests + all integration tests including intent_applier_canary (2 pass) and cast_id_propagation (3 pass). ✓
4. `rg "use bevy::winit|use bevy::render|use bevy_egui" src/combat/ --glob '!blueprints/**'`: only match is a doc comment in api/mod.rs — no actual import statements. ✓
5. `rg 'pub mod api' src/combat/mod.rs`: 1 match. ✓
6. `rg 'CombatPlugin' src/lib.rs`: 1 match (`pub use combat::CombatPlugin`). ✓
7. `rg 'add_plugins.*CombatPlugin' src/main.rs`: 1 match. ✓
8. `rg 'register_combat_kernel_runtime' src/main.rs`: 0 matches. ✓
9. `rg 'fn intent_applier' src/combat/api/applier.rs`: 1 match. ✓
10. `ls src/combat/api/`: 8 files present (applier.rs, clock.rs, intent.rs, mod.rs, registry.rs, rng.rs, signal.rs, skill_ctx.rs). ✓
11. All `CombatEvent { ... }` multi-line struct literals verified to include `cast_id` field via -A context check. ✓

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

None.

## Follow-ups

["S02: Timeline FSM — builds on Intent enum + SkillCtx established here", "S05: Wire remaining Intent variants (currently log::warn! stubs) and RON→CompiledTimeline compiler", "S04: Replace SignalBus scaffold with real reactor; replace BlueprintSignal u64 with closed-enum Signal per D028"]

## Files Created/Modified

None.
