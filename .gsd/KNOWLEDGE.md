# Project Knowledge

Append-only register. Agents read this before every unit.

## Onboarding

```bash
cargo check                   # headless (default)
cargo test                    # integration suite (tests/)
cargo run                     # headless run
cargo run --features windowed # egui UI (== `cargo winx`; includes bevy/dynamic_linking)
```

Toolchain: `rust-toolchain.toml`. Dev profile: cranelift on the workspace member, LLVM at `opt-level=0` on all dependencies (`Cargo.toml`). The `windowed` feature pulls `bevy/dynamic_linking`, so incremental rebuilds skip the static relink of the bevy_* graph. Resulting binary requires the bevy `.so` from `target/` — dev-only, not distributable.
Test workflow (nextest `agent` profile, seeded `bevy_rand`, insta snapshots): `docs/agent-testing.md`.
## Rules

| # | Scope | Rule | Added |
|---|-------|------|-------|
| R001 | Passive bootstrap | Per-Digimon passive listeners bootstrapped from `CombatPlugin` post core-init: fixed canonical `UnitId` owners, shared `kernel/ult_used` triggers, per-blueprint guard keys. Declarative, no per-test scaffolding. | 2026-05-17 |
| R002 | Headless-first | Every system runs without `windowed`. Gate egui/winit only via `#[cfg(feature = "windowed")]`. | 2026-05-18 |
| R003 | Test layout | Integration tests are aggregated into 19 scope harness binaries under `tests/<scope>.rs` (one binary per scope, ~90% fewer binaries than the previous flat layout). Each harness uses `#[path = "<scope>/case.rs"] mod case;` to include case files from `tests/<scope>/`. Shared helpers live in `tests/common/`. Case files that use `common::` must import via `crate::common::` (not `common::` or `self::common::`). Scopes: effects_kernel, invariants, windowed_only, passives_infra, blueprints_infra, follow_up, target_shape, action_query, tempo_toughness, assets_data, bootstrap_encounter, animation, damage_resolution, turn_economy, status_effects, preview_ai, runtime_events_obs, timeline, digimon_kits. New integration tests go under the appropriate `tests/<scope>/` directory. `rstest` for table cases, `proptest` for invariants. No `src/` unit tests except short `#[cfg(test)] mod tests`. | 2026-05-21 |
| R004 | Determinism | No wall-clock, no unseeded RNG. Seeded `bevy_rand` + insta depend on it. | 2026-05-18 |
| R005 | Dep gating | No winit/wgpu/egui deps outside `windowed`. | 2026-05-18 |
| R006 | Repo hygiene | No `.md` in repo root — use `docs/` or `.gsd/`. | 2026-05-18 |
| K001 | global | never execute the windowed binary from auto mode, if verification is needed demand the user to verify manually | — | manual |

## Patterns

| # | Pattern | Where | Notes |
|---|---------|-------|-------|
| P001 | Keep combat dispatch metadata on `InFlightAction`, not `ResolvedAction`. | `src/combat/turn_system/` | `ResolvedAction` stays semantic; routing details for the pipeline live on the in-flight execution wrapper. |
| P002 | Reuse the same compiled timeline resolution/interner path for preview and execute. | `src/combat/preview.rs`, timeline-backed consumers | Preview should run `BeatRunner` in preview mode and return the pending queue without touching `intent_applier`, so preview/execution drift is caught early. |
| P003 | Use owner-gated generic Blueprint envelopes while preserving typed owner-side observability seams. | Blueprint runtime modules + observability surfaces | Raw transport can stay generic without breaking downstream assertions if the owner module preserves the typed resolved-state contract. |
| P004 | Apply damage modifiers in canonical order: Intrinsic → Status → Buff → Passive. | Damage modifier ledger / incoming-damage pipeline | Fixed fold order keeps layered mitigation deterministic and replayable regardless of insertion order. |
| P005 | Blueprint `*Snapshot::last_transition` fields are typed observability contract surfaces, not internal latches. | `src/combat/blueprints/{tentomon,dorumon}/{identity,apply,mod}.rs` | Consumed by `ValidationExt` rows + per-step JSONL transition logging. Tests asserting on these fields are the regression guard for those display surfaces. See D026. |

## Lessons Learned

| # | What Happened | Root Cause | Fix | Scope |
|---|--------------|------------|-----|-------|
| L001 | Passive listeners that enqueue state changes for later predicates can loop forever or read stale state. | Queued intents were not flushed between `BeatRunner` steps, so later predicates observed old state. | Flush queued intents between passive-runner steps; for outer passive timelines, stop when the cursor cycles back to the entry beat while keeping explicit `BeatKind::Loop` bodies on the normal 256-hop breaker. | Passive timelines / follow-up runtime |
| L002 | Pre-damage events like `IncomingDamage` did not affect the current hit even when reactions subscribed to them. | `IncomingDamage` is observational only; modifiers were being armed too late. | Arm target-scoped modifiers in state before the damage intent is processed, then let the damage applier drain the ledger and emit the post-mitigation trigger once. | Incoming-damage / mitigation pipeline |
| L003 | A `#[ignore]`d unit test (`follow_up_low_hp_event_targets_alive_enemy`) claimed the timeline pipeline does not emit `OnAllyLowHp`/`OnEnemyBreak`. Un-ignored, the break test passed; the low-hp one still failed. | The ignore reason was stale. `OnAllyLowHp` (and break) threshold-crossing events ARE emitted for real RON-driven skills (proven by passing `renamon_low_hp_follow_up_targets_the_attacker` / `gabumon_triggers_follow_up_on_ally_low_hp`). The failing test used the synthetic `skill()` builder in `follow_up/mod.rs` tests, whose compiled timelines do not reproduce real-skill damage-derived observability. | Kept the now-passing break unit test; deleted the superseded synthetic low-hp test (behavioral coverage lives in the real-config integration tests). Assert `OnAllyLowHp`/break follow-ups via integration tests with real pilot config, not synthetic-skill unit tests. | Follow-up trigger tests / timeline observability |
| L004 | enoki `.particle.ron` effects "don't read as fire" — scattered confetti, no glowing body. | (1) Spawn path uses `ParticleSpawner::default()` (`src/windowed/render.rs:1608`) = `ColorParticle2dMaterial`, whose shader paints each quad a **flat solid square** (`particle_color_frag.wgsl`) — no soft alpha falloff, so particles never accumulate into a mass. The `.ron` carries NO texture/material; the spawn code picks it. (2) `spawn_rate` is the *interval in seconds between emissions*, not particles/sec (`bevy_enoki-0.6.0/src/update.rs:137`) — large values emit almost nothing; for `R`/sec author `1/R`. (3) The bevy_enoki web editor / SwiftShader headless capture use their own material and no bloom — authoritative for particle COUNT/motion, never for the LOOK. | Soft-blob look needs a windowed code change to spawn `ParticleSpawner::<SpriteParticle2dMaterial>(soft_texture)` + layered emitters (core/flames/embers), not an asset edit. Fixed `spawn_rate` in `assets/vfx/{fire,water}_test.particle.ron` (test-only, never spawned in-game). Full art guidance in skill `bevy-enoki-vfx` (`references/soft-particle-and-layering.md`). Aesthetic signoff is windowed-only (K001). | VFX / bevy_enoki rendering |
