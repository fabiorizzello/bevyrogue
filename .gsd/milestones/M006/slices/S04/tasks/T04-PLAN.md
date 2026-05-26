---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T04: Add agumon_module_extraction source-contract test and run the full verification gate

Why: the S04 contract is a grep-checkable structural claim that auto-mode cannot prove by launching the binary; an include_str! source-contract test (MEM030/MEM101 pattern, same as digimon_sprite_cue_dispatch.rs) pins it so a future edit that re-couples the engine fails CI. Do: (1) Create tests/windowed_only/agumon_module_extraction.rs with include_str! over src/windowed/render.rs and src/windowed/mod.rs asserting: NO occurrence of `AGUMON_`; NO `fn on_enter_effect_ids`; NO `fn skill_start_node`; NO `fn load_agumon_enoki_vfx`; NO `enoki_effect_path`; NO literal "digimon/agumon_atlas.png". (2) include_str! over src/windowed/digimon/agumon/mod.rs (and any agumon submodules) asserting the registrations are present: `EnokiVfxRegistry`, `OnEnterEffectRegistry`, `SkillStartNodeRegistry`, `SkillReleaseEffectRegistry`, `SpritePresentationRegistry`, the cue ids `hit_flash`/`hit_shake`/`camera_impact`, and the atlas path string now live here. (3) include_str! over src/windowed/digimon/mod.rs asserting `fn register_all` + `mod agumon`. (4) Register the file in tests/windowed_only.rs via the `#[path = ...] mod ...;` convention. Use code-shaped token assertions, no formula/value assertions, so the test survives behavior-preserving refactors. Done when: `cargo test --features windowed --test windowed_only agumon_module_extraction` passes; full `cargo test --features windowed --test windowed_only` green; `cargo test --test dependency_gating` 2/2 green; `cargo build --features windowed` zero warnings.

## Inputs

- `src/windowed/render.rs`
- `src/windowed/mod.rs`
- `src/windowed/digimon/mod.rs`
- `src/windowed/digimon/agumon/mod.rs`
- `tests/windowed_only.rs`

## Expected Output

- `tests/windowed_only/agumon_module_extraction.rs`
- `tests/windowed_only.rs`

## Verification

cargo test --features windowed --test windowed_only agumon_module_extraction
