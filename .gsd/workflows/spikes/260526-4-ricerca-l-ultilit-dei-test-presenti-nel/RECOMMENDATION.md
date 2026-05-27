# RECOMMENDATION — Prune low-value tests

**Date:** 2026-05-26 · **Spike:** 260526-4 · **Status:** knowledge + delete list (no code shipped)

## Executive summary

The suite is **748 test fns / 124 rstest case-rows** across the 19 R003 scope
harnesses. The two classes the user named — purely-visual tests and
impl-coupling churners — are **small and concentrated**, not pervasive. The bulk
of the suite (headless behavior tests, pure-function unit tests, RON parse
contracts) is healthy and stays.

Only **one file is a clean cut** (a visual look faked as a source-text check).
The real churn lives **inside** two source-token files as *presence* pins mixed
with genuine *boundary* guards — the recommendation is to **thin those, not
delete them**, because they also hold the only guard for an invariant
(engine-stays-species-agnostic) that has no runtime test by construction
(`src/windowed/` is binary-crate, unreachable from `tests/`, K001).

Net effect of acting on this: **−1 file, −1 test fn outright; ~−25 source-text
assertions thinned** from two files. Small surface, real friction removed, zero
behavioral coverage lost.

## Comparison matrix

| Category | Where | Anchored to | Churns on refactor? | Buys confidence? | Disposition |
|---|---|---|---|---|---|
| Visual proxy (source-text HDR/bloom) | `vfx_windowed_contracts.rs` | source text mentioning `"Bloom"` | yes | no (K001-manual) | **CUT** |
| Shape-freezing presence pins | `digimon_sprite_cue_dispatch.rs`, `enoki_impact_render.rs` | exact struct/enum/fn/field names | yes (3× in M006) | marginal (compiler already enforces) | **THIN** |
| Architectural boundary guards | same two files + `agumon_module_extraction.rs`, `renamon_extension_contract.rs` | *absence* of species branches / revived quad system | no | yes — only guard for the invariant | **KEEP** |
| RON parse / data contracts | `enoki_skill_effects_parse.rs`, `vfx_asset_impact_render.rs`, `enoki_impact_effect_parses.rs` | parsed asset structure | no | yes (catches broken `.ron`) | **KEEP** |
| Runtime windowed tests | 6 files that build real `App`s | observable ECS state | no | yes | **KEEP** |
| Relocated `*_internals.rs` (23 files) | pure-function math/parsers | input→output | no | yes | **KEEP** |
| `validation_snapshot` goldens (4) | deterministic format = the contract | output format | only on deliberate contract change | yes | **KEEP** (brittle, acceptable) |

## Delete list (concrete, ready to act on)

### Tier 1 — CUT outright
1. **`tests/windowed_only/vfx_windowed_contracts.rs`** — delete the whole file
   (1 test fn, 5 source-text assertions on `"Hdr"`/`"Bloom"`/`"Tonemapping"`/
   `"DebandDither"`/`"Color::linear_rgba"`).
   - **Coverage preserved by:** the HDR-bloom *look* is K001-manual and owned by
     `docs/uat/M004-vfx-signoff.md` (WAIVED). Dep-gating (R005/R016) is enforced
     by the `#![cfg(feature="windowed")]` build + `tests/dependency_gating.rs`,
     not by this file. Overbright/HDR color *values* live in the parsed `.ron`
     and are covered by the parse contracts. No invariant left unguarded.
   - Remove its `#[path = ...] mod vfx_windowed_contracts;` line from
     `tests/windowed_only.rs`.

### Tier 2 — THIN (rewrite to keep only the boundary, drop the shape pins)
2. **`tests/windowed_only/digimon_sprite_cue_dispatch.rs`** — keep the *absence*
   assertions that enforce "Agumon-named types are gone / legacy `flash_tint(` &
   `shake_offset(` lib calls are gone" (the real D048 + generalization
   boundary). Drop the *presence* pins on exact identifiers (`stance_graph_id`,
   `skill_graph_id`, `CameraRest`, `CameraShakeState`, `Camera2d`,
   `&mut Transform`, `struct DigimonSprite`, `enum DigimonPlaybackMode`) — those
   freeze names the compiler already requires.
3. **`tests/windowed_only/enoki_impact_render.rs`** — keep the *absence* guards
   that prove the deleted hand-rolled quad particle system did not return
   (`!contains("fn advance_vfx_particles")`, `!contains("VfxParticle {")`,
   `!contains("for i in 0..count")`) — these have no runtime equivalent. Drop the
   presence pins on Enoki API identifiers (`ParticleSpawner`,
   `ParticleEffectHandle`, `OneShot`, `ChargeEmberEnokiMarker`, `ProjectileFlight`,
   exact fn names) that merely echo current code shape.

   *Rationale for thin-not-delete:* these two files are the **only** guard for
   "engine stays species-agnostic / no revived quad VFX" — the M006 thesis. That
   guard must survive; the name-echo half is what churns.

### Tier 3 — KEEP (explicitly not cut, despite looking suspect)
- All `*_internals.rs` (Angle 2): stable pure-function tests, not impl-coupling
  churn.
- `validation_snapshot.rs` goldens: pin an intentional human-read format; brittle
  only to deliberate contract changes.
- All RON parse contracts and the 6 runtime windowed tests.
- `renamon_extension_contract.rs` / `agumon_module_extraction.rs`: the
  species-agnostic *absence* guards are the highest-value tests in the windowed
  set — keep.

## What would change this recommendation

- If `src/windowed/` ever becomes reachable from `tests/` (e.g. a thin lib-side
  re-export of the presentation seam), the Tier-2 *absence* guards should be
  rewritten as real runtime assertions and the source-text versions deleted
  entirely — that closes the churn at the root instead of thinning it.
- If a future Digimon is added and the species-agnostic guard does *not* fire on
  an intentional engine edit, the guard's token list is stale and needs widening,
  not deleting.

## Next steps if accepted

1. A follow-up slice (or `/gsd quick`) executes Tier 1 + Tier 2 with
   `cargo test --features windowed` green before/after to prove no behavioral
   coverage lost.
2. Consider appending a one-line decision to `.gsd/DECISIONS.md`: *"Source-text
   contract tests for `src/windowed/` keep only architectural absence-guards;
   presence/shape pins are prohibited as churn."* — this prevents the shape pins
   from growing back the next time a windowed seam is added.
