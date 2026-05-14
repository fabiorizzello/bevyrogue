---
id: T01
parent: S01
milestone: M021
key_files:
  - src/combat/api/mod.rs
  - src/combat/api/intent.rs
  - src/combat/api/registry.rs
  - src/combat/api/signal.rs
  - src/combat/api/clock.rs
  - src/combat/api/rng.rs
  - src/combat/mod.rs
key_decisions:
  - Named cast-scoped RNG `CastRng` (not `CombatRng`) to avoid ambiguity with existing `combat::rng::CombatRng` (StdRng)
  - Implemented SplitMix64 without new crate dependency — algorithm is 5 arithmetic ops, no rand_xoshiro needed
  - BlueprintSignal payload is u64 for S01; S04 replaces with closed-enum Signal per D028
  - SetBlueprintState.key is String (not &'static str) for runtime namespacing flexibility
  - ExtPoint::Fn placeholder is fn() for all 7 axes; concrete signatures refined per axis in S02+
duration: 
verification_result: passed
completed_at: 2026-05-14T22:53:37.484Z
blocker_discovered: false
---

# T01: Created src/combat/api/ skeleton with CastId, Intent (18 variants), Registry&lt;E&gt;+ExtRegistries (7 axes), SignalBus, Clock, and CastRng (SplitMix64) — all headless-safe, zero forbidden imports.

**Created src/combat/api/ skeleton with CastId, Intent (18 variants), Registry&lt;E&gt;+ExtRegistries (7 axes), SignalBus, Clock, and CastRng (SplitMix64) — all headless-safe, zero forbidden imports.**

## What Happened

Created the entire `src/combat/api/` module tree from scratch. Six files written:

1. `intent.rs` — `CastId(NonZeroU32)` with `const ROOT`, closed `Intent` enum with 18 variants (DealDamage, HealHp, ApplyStatus, RemoveStatus, ApplyBuff, RemoveBuff, AdvanceTurn, DelayTurn, EnqueueFollowUp, BreakToughness, ChargeUltimate, ModifySp, AddEnergy, RemoveEnergy, KoUnit, BlueprintSignal, SetBlueprintState, Reject). `BlueprintSignal.payload` is `u64` placeholder (S04 replaces with closed-enum Signal per D028). `SetBlueprintState.key` is `String` for runtime namespacing. Imports `StatusEffectKind`, `DamageTag`, `UnitId`, `SkillId` from existing `combat` modules — no circular deps.

2. `registry.rs` — `ExtPoint` trait (associated `type Fn: Send + Sync + 'static`), generic `Registry<E>` (HashMap-backed, O(1) lookup), 7 unit-struct axis markers (HookExt, SelectorExt, PredicateExt, FormulaExt, TickExt, AiUtilityExt, CueExt) each with placeholder `type Fn = fn()` to be refined in S02+, and `ExtRegistries` (Bevy Resource + Default) aggregating all 7. Inline tests: hit, miss, overwrite, all-axes-empty.

3. `signal.rs` — `SignalBus` (Bevy Resource + Default) scaffold. Carries a `_pending: u32` placeholder; S04 replaces with `VecDeque<Signal>`.

4. `clock.rs` — `Clock { HeadlessAuto, Windowed }` enum with `#[default] HeadlessAuto`. Documents invariant I3 / D026.

5. `rng.rs` — `CastRng(u64)` SplitMix64. `new(seed)` warms up one step. `from_params(global_seed, cast_id, beat, hop, salt)` mixes parameters via multiplicative hashing before warming up. `next_u64`, `next_u32`, `next_below(n)`, `roll_pct(pct)`. Inline tests: same-seed determinism (20 draws), different-seeds diverge, from_params determinism, parameter sensitivity (each of 5 params changes the stream), roll_pct boundaries, next_below distribution coarse coverage.

6. `mod.rs` — declares all 5 submodules, re-exports stable public surface (CastId, Intent, ExtPoint, Registry, all 7 ExtPoint markers, ExtRegistries, CastRng, Clock, SignalBus), documents the import constraint.

Updated `src/combat/mod.rs` to add `pub mod api;` in a new "Framework API (M021)" section at the top of the core kernel group, before existing modules.

Design note: the new `api::rng::CastRng` (SplitMix64, cast-scoped) is distinct from `combat::rng::CombatRng` (StdRng, global status rolls). They coexist under different module paths without name conflict.

## Verification

1. `cargo check` (headless) — Finished with only pre-existing warnings, exit 0.
2. `cargo check --features windowed` — Finished with only pre-existing warnings, exit 0.
3. `rg '^use bevy::(winit|render)|^use bevy_egui' src/combat/api/` — no matches (CLEAN).
4. `cargo test --lib api::registry::tests` — 4 tests passed (hit, miss, overwrite, ext_registries_default_empty).
5. `cargo test --lib api::rng::tests` — 6 tests passed (determinism_same_seed, different_seeds_diverge, from_params_determinism, from_params_sensitivity, roll_pct_boundaries, next_below_distribution_coarse).
6. `rg 'pub mod api' src/combat/mod.rs` — 1 match.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check` | 0 | pass | 4060ms |
| 2 | `cargo check --features windowed` | 0 | pass | 5490ms |
| 3 | `rg '^use bevy::(winit|render)|^use bevy_egui' src/combat/api/` | 1 | pass — no forbidden imports | 50ms |
| 4 | `cargo test --lib api::registry::tests` | 0 | pass — 4/4 | 4900ms |
| 5 | `cargo test --lib api::rng::tests` | 0 | pass — 6/6 | 130ms |
| 6 | `rg 'pub mod api' src/combat/mod.rs` | 0 | pass — 1 match | 20ms |

## Deviations

Named the SplitMix64 RNG `CastRng` instead of `CombatRng` to avoid naming collision with the existing `src/combat/rng::CombatRng`. Both names refer to the same conceptual entity from the plan; the rename is a disambiguation, not a scope change.

## Known Issues

none

## Files Created/Modified

- `src/combat/api/mod.rs`
- `src/combat/api/intent.rs`
- `src/combat/api/registry.rs`
- `src/combat/api/signal.rs`
- `src/combat/api/clock.rs`
- `src/combat/api/rng.rs`
- `src/combat/mod.rs`
