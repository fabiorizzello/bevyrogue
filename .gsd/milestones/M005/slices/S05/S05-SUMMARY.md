---
id: S05
parent: M005
milestone: M005
provides:
  - (none)
requires:
  []
affects:
  []
key_files: []
key_decisions:
  - AgumonEnokiVfx holds a HashMap keyed by effect id (not asset path), matching the same string constants in vfx.ron — new bursts need only a const + map entry in load_agumon_enoki_vfx with no spawn_effect_by_id branching (MEM105)
  - enoki_effect_path() id→path helper keeps the diagnostic reporting-complete without leaking paths into the resource struct
  - baby_burner.flash central pop folded into baby_burner_detonate.particle.ron as a brighter initial color_curve core rather than a separate asset (seam routes one handle per effect id)
  - baby_flame.impact enriched in-place (spawn_amount 28→40, linear_speed 150→190) rather than a separate shard layer, keeping existing parse test invariants intact
  - K001 windowed visual sign-off is a human/UAT gate; auto-mode documents the requirement but cannot execute it
patterns_established:
  - Per-effect-id enoki handle map pattern: add const path + map entry in load_agumon_enoki_vfx to route any new contact-burst id through enoki without touching spawn_effect_by_id
  - Generalized load-failure diagnostic: Local<HashSet<String>> warned-set in diagnose_agumon_enoki_vfx_load emits one WARN per failed handle naming both the asset path and effect id
observability_surfaces:
  - diagnose_agumon_enoki_vfx_load: iterates the enoki handle map at runtime (windowed), emitting a WARN per failed .particle.ron load naming effect id and asset path — dead per-skill burst is visible by name rather than silently absent
  - load_agumon_enoki_vfx: logs the three requested asset paths at INFO on Startup
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-26T09:50:01.100Z
blocker_discovered: false
---

# S05: Full Agumon VFX migration to enoki

**Generalized the enoki spawn seam to a per-effect-id handle map and routed all three Agumon contact bursts (sharp_claws.slash, baby_flame.impact, baby_burner.detonate) through bevy_enoki one-shots with authored .particle.ron assets and full regression sweep green; K001 manual visual sign-off remains pending.**

## What Happened

S05 extended the single-handle S04 enoki seam into a general-purpose per-effect-id handle map covering all three Agumon contact-burst VFX.

**T01 — Generalize the enoki spawn seam:** `AgumonEnokiVfx` in `src/windowed/render.rs` changed from a single `handle` field to `handles: HashMap<String, Handle<Particle2dEffect>>`. Three const asset paths were added (AGUMON_ENOKI_SHARP_CLAWS_PATH, existing impact path, AGUMON_ENOKI_DETONATE_PATH) and `load_agumon_enoki_vfx` builds the map keyed by AGUMON_SHARP_CLAWS_EFFECT_ID / AGUMON_IMPACT_EFFECT_ID / AGUMON_DETONATE_EFFECT_ID with an INFO log on Startup. The `spawn_effect_by_id` intercept simplified to a map lookup (`enoki.handles.get(effect_id)` → ParticleSpawner + ParticleEffectHandle + OneShot::Despawn), leaving the `for i in 0..count` quad loop as the fallback for every other id. The diagnostic `diagnose_agumon_enoki_vfx_load` was generalized to iterate the map with a `Local<HashSet<String>>` warned-set so each dead per-skill burst surfaces by name exactly once. A small `enoki_effect_path()` id→path helper keeps the diagnostic reporting-complete without storing paths in the resource. No kernel cue / barrier / FSM control flow was touched (D031/D032). The windowed_only contract test in `enoki_impact_render.rs` was rewritten to assert the map-lookup branch for all three ids and the surviving quad loop.

**T02 — Author .particle.ron assets + parse tests:** Two new assets were authored and one enriched, all listing all 19 Particle2dEffect fields explicitly (MEM102 — no serde defaults, Option fields as Some(..)/None): (a) `sharp_claws_slash.particle.ron` — pale yellow-white slash burst (spawn_rate 0, spawn_amount 18, Circle(4.0), ~0.25s lifetime, color_curve overbright (3.0,3.0,2.2) → transparent, scale_curve pop-then-shrink); (b) `baby_burner_detonate.particle.ron` — orange radial shard burst folding the central baby_burner.flash pop (spawn_amount 44, linear_speed 220, brighter core (3.4,2.0,0.8) → hot orange (2.2,1.0,0.3) → transparent); (c) `baby_flame_impact.particle.ron` enriched (spawn_amount 28→40, linear_speed 150→190) so the burst reads as central flash plus radiating shards. No numeric gameplay payload on SpawnParticle (R012). A new `enoki_skill_effects_parse.rs` test file asserts one-shot invariants for both new assets via a shared `assert_one_shot_burst` helper; the module was registered in `windowed_only.rs`.

**T03 — Full regression sweep:** All four commands exited 0 with no source changes: `cargo test` (51 passed headless), `cargo build --features windowed` (enoki stack compiles windowed-gated), `cargo test --features windowed --test windowed_only` (49 passed, including all three asset parse tests and the generalized seam contract tests), `cargo test --test dependency_gating` (2 passed: bevy_enoki absent from headless graph, present in windowed graph — R005/R016 dep-isolation invariant holds). K001 windowed visual sign-off is deferred to the user as a required manual UAT step.

## Verification

Four-command regression sweep, all exit 0:
1. `cargo test` — 51 passed, 0 failed (headless suite including standalone dependency_gating binary)
2. `cargo build --features windowed` — exit 0, enoki render stack compiles windowed-gated
3. `cargo test --features windowed --test windowed_only` — 49 passed, 0 failed (three Agumon contact-burst parse tests: enoki_skill_effects_parse + enriched baby_flame via enoki_impact_effect_parses; generalized seam source-contract tests in enoki_impact_render: spawn_effect_by_id_routes_mapped_ids_through_an_enoki_one_shot, enoki_handle_map_is_keyed_by_all_three_contact_burst_ids)
4. `cargo test --test dependency_gating` — 2 passed (bevy_enoki_absent_from_headless_graph, bevy_enoki_present_in_windowed_graph)

K001 windowed visual sign-off: cannot be executed from auto-mode; documented as required manual UAT step.

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

K001 windowed visual sign-off is pending — auto-mode cannot launch the windowed binary. The user must run `cargo winx` and confirm all three Agumon contact bursts render through enoki and look better than the flat-quad placeholder before considering M005 visually complete.

## Follow-ups

D040: migrate charge buildup and traveling projectile body VFX to enoki emitters. D041: delete the now-dormant Agumon quad path behind the seam once K001 visual sign-off is obtained.

## Files Created/Modified

- `src/windowed/render.rs` — 
- `tests/windowed_only/enoki_impact_render.rs` — 
- `assets/digimon/agumon/sharp_claws_slash.particle.ron` — 
- `assets/digimon/agumon/baby_burner_detonate.particle.ron` — 
- `assets/digimon/agumon/baby_flame_impact.particle.ron` — 
- `tests/windowed_only/enoki_skill_effects_parse.rs` — 
- `tests/windowed_only.rs` — 
