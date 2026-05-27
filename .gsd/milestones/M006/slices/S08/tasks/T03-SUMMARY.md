---
id: T03
parent: S08
milestone: M006
key_files:
  - src/windowed/render.rs
  - tests/windowed_only/renamon_extension_contract.rs
key_decisions:
  - Warn at the cue level keyed by cue id (descriptor.particle.0), unifying both spawn-miss sub-cases (unmapped cue → empty effect_ids, and registered cue → effect id missing from EnokiVfxRegistry) under one warning gated on cue_spawned==0, rather than warning per-effect-id
  - Reused the S06 Local<HashSet<String>> warn-once dedup pattern (not a shared helper function — S06 established a pattern, not a reusable fn) keyed by cue id so each unregistered cue logs at most once across frames
  - Added the contract via include_str! source-contract test rather than a live behavior test because src/windowed is a binary crate unreachable from tests/ and K001 forbids launching the windowed binary
duration: 
verification_result: mixed
completed_at: 2026-05-27T08:51:39.291Z
blocker_discovered: false
---

# T03: Added warn-once cast-cue spawn-miss diagnostic: a SpawnParticle cue that spawns no particle is now logged once per cue id instead of silently no-op'ing

**Added warn-once cast-cue spawn-miss diagnostic: a SpawnParticle cue that spawns no particle is now logged once per cue id instead of silently no-op'ing**

## What Happened

Reused the S06 `Local<HashSet>` warn-once pattern to make cast-cue spawn misses visible in `advance_digimon_presentation` (src/windowed/render.rs). Previously, when an on_enter `SpawnParticle` cue resolved to zero spawned particles — either because the cue name was unmapped in `OnEnterEffectRegistry` (empty `effect_ids` slice) or because its mapped effect ids were absent from `EnokiVfxRegistry` (`spawn_effect_by_id` returns 0) — the spawn loop simply did nothing, producing a silent no-op with no log trail.

Implementation: the on_enter spawn loop now accumulates `cue_spawned` across the cue's effect ids. After the loop, if `cue_spawned == 0` and the cue id has not been warned before, a single `warn!(target: "windowed.digimon_playback", cue = descriptor.particle.0.as_str(), node, source_unit, ...)` fires. Dedup state is a new `mut cast_cue_spawn_miss_warned: Local<HashSet<String>>` system param keyed by the authored cue id (`descriptor.particle.0`). This brings the system to exactly 16 params (Bevy's limit) — verified to compile. Registered cues that spawn ≥1 particle stay silent because the warn is gated on the zero-spawn condition.

This unifies both spawn-miss sub-cases under one cue-keyed warning: the empty-map case (cue not registered at all) and the registered-but-unbacked case (effect id present in OnEnterEffectRegistry but missing from EnokiVfxRegistry). It complements the existing `diagnose_enoki_vfx_load` which only catches asset *load* failures, not missing registrations.

Test: added the source-contract test `cast_cue_spawn_miss_warns_once_with_cue_id` to tests/windowed_only/renamon_extension_contract.rs. Following the established include_str! approach (src/windowed is a binary crate unreachable from tests/, and K001 forbids launching the windowed binary), it pins the warn-once seam by co-occurring canonical tokens: the `Local<HashSet<String>>` dedup state, the `cue_spawned += spawned` accumulation, the `if cue_spawned == 0` gate, the cue-id-keyed `.insert(descriptor.particle.0.clone())` dedup, the `cue = descriptor.particle.0.as_str()` log field, and the `windowed.digimon_playback` log target. The existing `engine_files_stay_species_agnostic` guard still passes — the change introduces no species-specific tokens into render.rs.

## Verification

cargo test --features windowed --test windowed_only → 72 passed, 0 failed (includes new cast_cue_spawn_miss_warns_once_with_cue_id and the no-warn-on-happy-path is encoded by the cue_spawned==0 gate). cargo check --features windowed → exit 0, no warnings (confirms the 16th system param is within Bevy's limit). cargo check (headless) → exit 0 (change is fully windowed-gated, headless unaffected). Manual winx clean-for-registered-cues sign-off is K001-blocked: auto-mode cannot launch the windowed binary; pending human confirmation that registered cues emit no spawn-miss warning at runtime.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --features windowed --test windowed_only` | 0 | pass — 72 passed; 0 failed (incl. new cast_cue_spawn_miss_warns_once_with_cue_id) | 2511ms |
| 2 | `cargo check --features windowed (after touch render.rs)` | 0 | pass — binary compiles with 16-param advance_digimon_presentation | 669ms |
| 3 | `cargo check (headless default)` | 0 | pass — headless build unaffected | 1561ms |
| 4 | `manual winx registered-cue silence` | -1 | blocked by K001 — pending human verification | 0ms |

## Deviations

none

## Known Issues

Runtime confirmation that registered cues stay silent (no spawn-miss warn on the happy path) requires launching the windowed binary, which K001 forbids in auto-mode — pending manual human sign-off.

## Files Created/Modified

- `src/windowed/render.rs`
- `tests/windowed_only/renamon_extension_contract.rs`
