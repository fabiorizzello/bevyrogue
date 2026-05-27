# ANGLE 1 — Source-token structural guards

**Confidence: high.** Read the actual files; counted assertions.

## What they are

Tests under `tests/windowed_only/` that `include_str!("../../src/windowed/**.rs")`
and assert `.contains("token")` on the source *text*. Forced by MEM030:
`src/windowed/` is binary-crate code, unreachable from `tests/`, and K001 forbids
launching the windowed binary in auto-mode. So there is no runtime path — text
inspection is the only headless option these chose.

`.contains(` assertion density:

| File | `.contains` | test fns | character |
|---|---|---|---|
| `digimon_sprite_cue_dispatch.rs` | 19 | 5 | mostly shape-freezing presence pins |
| `enoki_impact_render.rs` | 17 | 6 | mixed: real "no manual particle loop" boundary + shape pins |
| `vfx_windowed_contracts.rs` | 5 | 1 | pure visual proxy (HDR/Bloom token presence) |
| `agumon_module_extraction.rs` | 4 | 3 | real "engine stays species-agnostic" boundary |
| `renamon_extension_contract.rs` | (helper-driven, many tokens) | 8 | real boundary + asset-shape contracts |

## The split that matters

Two distinct kinds hide inside these files:

1. **Boundary guards (real value, durable).** *Absence* assertions that catch an
   architectural regression that has no behavioral test because the code is
   unreachable:
   - `engine_files_stay_species_agnostic` (renamon) — `render.rs`/`mod.rs` must
     NOT contain `"renamon"`, `"diamond_storm"`, etc. This is the entire M006
     thesis (engine consumes registries, never branches on species). A real bug
     (someone hardcodes a species in the engine) trips this and nothing else.
   - `agumon_module_extraction` forbidden-token set — same boundary.
   - `enoki_impact_render`: `!contains("for i in 0..count")`, `!contains("fn
     advance_vfx_particles")`, `!contains("VfxParticle {")` — guards "the deleted
     hand-rolled quad particle system did not come back" (D-level decision). Real.

2. **Shape-freezing presence pins (churn, low value).** *Presence* assertions on
   exact identifiers: `struct DigimonSprite`, `enum DigimonPlaybackMode`,
   `stance_graph_id`, `flash_tint_parametric`, `&mut Transform`, `Camera2d`,
   `CameraRest`, `CameraShakeState`. These freeze the current names/types. A
   pure rename with identical behavior breaks them. M006 history confirms the
   churn: these were inverted/rewritten across S01, S03, and S04 as the code
   moved — three rewrites in one milestone is the smell.

## Verdict

- Keep the **boundary/absence** guards — they protect a real invariant that has
  no other home.
- Cut the **presence shape-pins** — they buy "the identifier still exists,"
  which the compiler already guarantees for anything actually wired up, and they
  generate refactor friction.
- `vfx_windowed_contracts.rs` belongs to Angle 3 (visual proxy) — see there.
