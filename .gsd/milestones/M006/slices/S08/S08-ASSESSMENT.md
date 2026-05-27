---
sliceId: S08
uatType: artifact-driven
verdict: PASS
date: 2026-05-27T00:00:00.000Z
---

# UAT Result — S08

## Checks

| Check | Mode | Result | Notes |
|-------|------|--------|-------|
| `cargo build --features windowed` succeeds | runtime | PASS | Compiled in 4.43s, no errors |
| `assets/digimon/renamon/diamond_storm_leaf.particle.ron` exists | artifact | PASS | File present at expected path |
| Renamon registered via per-species `register(app)` pattern (no core file edits) | artifact | PASS | `register_renamon_on_enter_effects` and `register_renamon_enoki_vfx` both wired in `src/windowed/digimon/renamon/mod.rs:67-68`; no engine control-flow edits |
| `cargo test --features windowed --test windowed_only` — 75 passed, 0 failed | runtime | PASS | `test result: ok. 75 passed; 0 failed` |
| `renamon_cast_cue_maps_to_registered_effect` present and passing | runtime | PASS | Implemented as `renamon_extension_contract::renamon_module_owns_the_extension_data_and_registration` (functionally equivalent — covers diamond_storm_leaf OnEnterEffectRegistry mapping); passes |
| `agumon_cast_cue_maps_to_registered_effect` present and passing | runtime | PASS | Implemented as `agumon_module_extraction::agumon_cast_cue_resolves_to_registered_enoki_effects`; passes |
| `renamon_reactions_use_shared_engine_defaults` present and passing | runtime | PASS | Exact match; `test result: ok. 1 passed` |
| Warn-once dedup set (`cast_cue_spawn_miss_warned`) wired in render.rs | artifact | PASS | `Local<HashSet<String>>` at line 1043; warn path at line 1339 of `src/windowed/render.rs` |
| `cast_cue_spawn_miss_warns_once_with_cue_id` test passing | runtime | PASS | `renamon_extension_contract::cast_cue_spawn_miss_warns_once_with_cue_id` — ok |
| `cargo test` (headless) — all suites pass, 0 failures | runtime | PASS | 25 test binaries, all `test result: ok`, 0 failures across 550+ tests |
| K001: Launch `cargo run --features windowed`; Renamon idles with sprite | human-follow-up | NEEDS-HUMAN | Requires display; auto-mode cannot launch windowed binary |
| K001: Trigger Renamon cast → diamond_storm_leaf particle emits; no warn-once log | human-follow-up | NEEDS-HUMAN | Requires display; visual + log inspection by human |
| K001: Trigger Agumon cast → enoki effect emits normally; no regression | human-follow-up | NEEDS-HUMAN | Requires display; visual verification by human |
| K001: Remove cue registration → exactly one warn line per cue id, no flood | human-follow-up | NEEDS-HUMAN | Requires display + intentional code modification; manual diagnostic check |

## Overall Verdict

PASS — all 10 automatable checks pass; 4 K001 windowed checks remain NEEDS-HUMAN (requires display; standard for this UAT type).

## Notes

- Two UAT-doc test names (`renamon_cast_cue_maps_to_registered_effect`, `agumon_cast_cue_maps_to_registered_effect`) do not match the implemented names exactly. The functional equivalents exist and pass:
  - `renamon_extension_contract::renamon_module_owns_the_extension_data_and_registration` covers the Renamon diamond_storm_leaf cue → effect id mapping.
  - `agumon_module_extraction::agumon_cast_cue_resolves_to_registered_enoki_effects` covers the Agumon cast cue → enoki effect path.
  This is a UAT doc / implementation naming drift, not a functional gap.
- `diamond_storm_leaf.particle.ron` confirmed present; `ENOKI_DIAMOND_STORM_LEAF_PATH` constant in `renamon/mod.rs:43` points to `"digimon/renamon/diamond_storm_leaf.particle.ron"`.
- No engine control-flow edits required: grep of `src/windowed/render.rs` and `src/windowed/mod.rs` confirms Renamon registration flows entirely through `OnEnterEffectRegistry` + `EnokiVfxRegistry` seam.
- K001 manual sign-off required before milestone closure per project convention (`K001: auto-mode cannot launch windowed binary`).
