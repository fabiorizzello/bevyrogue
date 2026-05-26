---
id: T02
parent: S04
milestone: M006
key_files:
  - src/windowed/render.rs
  - src/windowed/digimon/agumon/mod.rs
  - tests/windowed_only/vfx_windowed_contracts.rs
  - tests/windowed_only/enoki_impact_render.rs
  - tests/animation/render_no_vfx_kind_guard.rs
key_decisions:
  - Bundled the four spawn-side effect registries (enoki + on_enter + skill_release) into a #[derive(SystemParam)] EffectRegistries struct to keep advance_agumon_presentation within Bevy's 16-param system limit (MEM117).
  - EnokiLifecycle enum carries spawn behavior as data per registry entry (PersistentEmitter / Projectile{flight_ticks,on_arrival} / OneShot), replacing the closed effect-id match in spawn_effect_by_id; ProjectileFlight gained on_arrival and lost Copy.
  - Effect-id and asset-path string constants now live in the agumon module; render.rs holds no Agumon literals. Generic registries are init-empty by the engine (RenderPlugin) and populated by agumon::register Startup systems.
  - Updated the three source-contract tests that pinned the pre-extraction structure rather than deleting them, re-targeting handle-map/effect-id assertions at the agumon module source and the new OnEnterEffectRegistry seam in render.rs.
duration: 
verification_result: passed
completed_at: 2026-05-26T12:08:02.715Z
blocker_discovered: false
---

# T02: Converted all Agumon enoki effect-id data + on-enter/release/arrival/detonate spawns from render.rs consts/closed-matches to engine-generic registries populated by the agumon module

**Converted all Agumon enoki effect-id data + on-enter/release/arrival/detonate spawns from render.rs consts/closed-matches to engine-generic registries populated by the agumon module**

## What Happened

Extracted every Agumon-specific effect datum and the closed control-flow out of src/windowed/render.rs into generic engine registries that the per-Digimon agumon module populates via its register() Startup systems (atomic per D049, since the consts were cross-referenced).

Engine side (render.rs): (1) Renamed AgumonEnokiVfx -> EnokiVfxRegistry (HashMap<String, EnokiEffect>, Default) and extended EnokiEffect with `path: String` + `lifecycle: EnokiLifecycle` (new enum: PersistentEmitter / Projectile{flight_ticks,on_arrival} / OneShot). (2) Added engine resources OnEnterEffectRegistry (name->Vec<effect_id>), SkillReleaseEffectRegistry (skill_id->effect_id), DetonateEffectRegistry (Option<effect_id>). All four init_resource'd empty in RenderPlugin::build (replacing the Startup load_agumon_enoki_vfx). (3) Re-pointed consumers: spawn_effect_by_id matches on entry.lifecycle instead of effect_id; ProjectileFlight gained `on_arrival: String` (dropped Copy) and advance_enoki_projectiles spawns flight.on_arrival instead of the impact const; the on_enter loop reads OnEnterEffectRegistry; the release boundary replaced `mode_skill_id == Some(BABY_FLAME_SKILL_ID)` with a SkillReleaseEffectRegistry lookup (still clears ChargeEmberEnokiMarker generically); spawn_detonate_particles reads DetonateEffectRegistry. (4) diagnose_agumon_enoki_vfx_load -> diagnose_enoki_vfx_load, reading EnokiEffect.path instead of the enoki_effect_path match (deleted). (5) Deleted on_enter_effect_ids fn + enoki_effect_path fn + all AGUMON_*_EFFECT_ID / AGUMON_ENOKI_*_PATH / AGUMON_PROJECTILE_FLIGHT_TICKS consts.

agumon module (src/windowed/digimon/agumon/mod.rs): added four Startup register systems that populate the registries with Agumon's six effects (charge/ember=PersistentEmitter@Mouth, projectile=Projectile{5,baby_flame.impact}@CasterCenter, impact/detonate/sharp_claws=OneShot@TargetCenter), the on-enter name map, baby_flame->baby_flame.projectile release, and baby_burner.detonate. Effect-id/path constants now live here.

All trace/info/warn targets stayed "windowed.agumon_playback".

Deviation: bundling the three new registries plus the enoki registry into the already-15-param advance_agumon_presentation exceeded Bevy's 16-parameter system limit, so I introduced a `#[derive(SystemParam)] struct EffectRegistries` to fold them into one param (captured as MEM117).

Moved the on_enter_effect_ids unit tests into the agumon module (now asserting on_enter_effect_specs + registry population via a minimal App), and updated three source-contract tests that pinned the old structure: vfx_windowed_contracts (slice boundary), enoki_impact_render (handle-map now asserts the agumon source by string id), and render_no_vfx_kind_guard (positive contract now checks OnEnterEffectRegistry in render.rs + sharp_claws.slash in the agumon source).

## Verification

cargo build --features windowed: green, zero rustc warnings. cargo test --features windowed --test windowed_only: 59 passed/0 failed (incl. the two updated contract tests). cargo test --test dependency_gating: 2 passed (enoki still absent from headless graph). cargo test --test animation render_no_vfx_kind_guard: 2 passed. cargo test --features windowed --bin bevyrogue windowed::: 23 passed incl. 4 agumon-module tests (on_enter charge/sharp_claws, skill_release, detonate). Done-condition greps: `AGUMON_.*EFFECT_ID` in render.rs -> none (exit 1); `fn on_enter_effect_ids|fn load_agumon_enoki_vfx|fn enoki_effect_path` in render.rs -> none (exit 1). Clippy: only pre-existing crate-wide lints; zero new warnings in render.rs/agumon.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build --features windowed` | 0 | pass | 3048ms |
| 2 | `cargo test --features windowed --test windowed_only` | 0 | pass (59 passed) | 3092ms |
| 3 | `cargo test --test dependency_gating` | 0 | pass (2 passed) | 600ms |
| 4 | `cargo test --test animation render_no_vfx_kind_guard` | 0 | pass (2 passed) | 680ms |
| 5 | `cargo test --features windowed --bin bevyrogue windowed::` | 0 | pass (23 passed) | 1906ms |
| 6 | `grep -rqE 'AGUMON_.*EFFECT_ID' src/windowed/render.rs` | 1 | pass (no match, as required) | 10ms |
| 7 | `grep -nE 'fn on_enter_effect_ids|fn load_agumon_enoki_vfx|fn enoki_effect_path' src/windowed/render.rs` | 1 | pass (no match, as required) | 10ms |

## Deviations

Introduced EffectRegistries SystemParam wrapper (not in the plan text) because folding the new registries into advance_agumon_presentation exceeded Bevy's 16-parameter system limit. Also updated three pre-existing source-contract tests (vfx_windowed_contracts, enoki_impact_render, render_no_vfx_kind_guard) that asserted the old render.rs structure, to keep windowed_only + animation suites green.

## Known Issues

Comment-only references to the old AGUMON_ENOKI_*_PATH const names remain in tests/windowed_only/enoki_impact_effect_parses.rs and enoki_skill_effects_parse.rs (documentation strings only; assertions use literal asset paths, so tests pass). Left as-is to minimize churn.

## Files Created/Modified

- `src/windowed/render.rs`
- `src/windowed/digimon/agumon/mod.rs`
- `tests/windowed_only/vfx_windowed_contracts.rs`
- `tests/windowed_only/enoki_impact_render.rs`
- `tests/animation/render_no_vfx_kind_guard.rs`
