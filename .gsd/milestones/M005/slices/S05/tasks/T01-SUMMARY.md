---
id: T01
parent: S05
milestone: M005
key_files:
  - src/windowed/render.rs
  - tests/windowed_only/enoki_impact_render.rs
key_decisions:
  - Keyed the enoki handle map by effect id (String) rather than asset path, so the spawn seam looks up by the same effect_id it already iterates
  - Generalized the load-failure diagnostic to a Local<HashSet<String>> warned-set with a per-id WARN, so each dead per-skill burst surfaces by name exactly once
  - Added an enoki_effect_path() id->path helper so the diagnostic can report the source asset path without storing paths in the resource
duration: 
verification_result: passed
completed_at: 2026-05-26T09:43:40.865Z
blocker_discovered: false
---

# T01: Generalized the enoki spawn seam from one hardcoded id to a per-effect-id handle map keyed by all three Agumon contact-burst ids

**Generalized the enoki spawn seam from one hardcoded id to a per-effect-id handle map keyed by all three Agumon contact-burst ids**

## What Happened

Replaced the single-handle S04 enoki seam in src/windowed/render.rs with a per-effect-id `HashMap<String, Handle<Particle2dEffect>>`. Changes: (1) added `use std::collections::{HashMap, HashSet}`; (2) added two new const paths (AGUMON_ENOKI_SHARP_CLAWS_PATH, AGUMON_ENOKI_DETONATE_PATH) next to the existing impact path; (3) AgumonEnokiVfx now holds `handles` instead of a single `handle`; (4) load_agumon_enoki_vfx builds the map keyed by AGUMON_SHARP_CLAWS_EFFECT_ID, AGUMON_IMPACT_EFFECT_ID, AGUMON_DETONATE_EFFECT_ID and logs all three paths at INFO on Startup; (5) the spawn_effect_by_id intercept changed from `if effect_id == AGUMON_IMPACT_EFFECT_ID` to `if let Some(enoki) = enoki { if let Some(handle) = enoki.handles.get(effect_id) { ... return 1; } }`, leaving the `for i in 0..count` quad loop untouched for unmatched ids; (6) diagnose_agumon_enoki_vfx_load now iterates the map with a `Local<HashSet<String>>` warned-set, emitting the per-id WARN (path + effect id) once per failed handle — a dead per-skill burst is now visible by name. Added a small enoki_effect_path() id->path helper for the diagnostic. No kernel cue / barrier / FSM control flow was touched (D031/D032 preserved). Updated tests/windowed_only/enoki_impact_render.rs: rewrote the impact-only assertions to assert the map-lookup branch (`enoki.handles.get(effect_id)` + ParticleSpawner + ParticleEffectHandle + OneShot) and the surviving quad loop, added a new test slicing load_agumon_enoki_vfx to assert all three contact-burst ids are inserted into the map, and kept the EnokiPlugin / fire_kernel_cue / request_release assertions. The two new .particle.ron asset files do not exist yet (sharp_claws_slash, baby_burner_detonate) — their handles will report LoadState::Failed at runtime and the generalized diagnostic will name them; creating those assets is downstream slice work.

## Verification

Ran the slice-level verification command `cargo test --features windowed --test windowed_only`: 47 tests pass, 0 fail. The five enoki contract tests pass, including the rewritten spawn_effect_by_id_routes_mapped_ids_through_an_enoki_one_shot and the new enoki_handle_map_is_keyed_by_all_three_contact_burst_ids. Also ran `cargo check --features windowed` to confirm the windowed binary compiles cleanly with the map-based resource (no unused-const or type errors). Per K001 the windowed binary was not executed; verification is via the source-contract test plus compile check.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --features windowed --test windowed_only` | 0 | pass | 30000ms |
| 2 | `cargo test --features windowed --test windowed_only enoki` | 0 | pass | 2000ms |
| 3 | `cargo check --features windowed` | 0 | pass | 34180ms |

## Deviations

Added a small enoki_effect_path() helper not explicitly itemized in the plan, needed so the generalized diagnostic can report the source path per effect id (the single-handle version hardcoded the path constant). Also added a dedicated test (enoki_handle_map_is_keyed_by_all_three_contact_burst_ids) beyond rewriting the existing one, to pin that the map routes all three ids per the plan's intent.

## Known Issues

The two new asset files (assets/digimon/agumon/sharp_claws_slash.particle.ron and baby_burner_detonate.particle.ron) do not exist yet, so their handles will report LoadState::Failed at runtime and the diagnostic will WARN by name until downstream tasks author them. This is expected for T01, whose scope is the seam generalization only.

## Files Created/Modified

- `src/windowed/render.rs`
- `tests/windowed_only/enoki_impact_render.rs`
