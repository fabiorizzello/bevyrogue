---
estimated_steps: 12
estimated_files: 6
skills_used: []
---

# T01: Generalize multi-presentation and demo composition seams

Expected executor skills/frontmatter: rust-development, rust-skills, bevy, tdd, write-docs, verify-before-complete.

Why: S05 research and code checks found the current S04 seam still assumes one presentation entry and one demo species: build_digimon_atlas() and spawn_unit_sprites() read SpritePresentationRegistry.entries.first(), render.rs still owns an AgumonAtlas resource name, and windowed_bootstrap_system() hardcodes EncounterPreset::AgumonTrainingDummy. Renamon cannot appear in cargo winx until these species-agnostic seams are fixed.

Do:
1. Add or start tests/windowed_only/renamon_extension_contract.rs with source-contract assertions for the generic seam: no presentation.entries.first() in src/windowed/render.rs, no AgumonAtlas/advance_agumon_presentation engine naming, no Renamon tokens in engine files, and no windowed-only hardcoded AgumonTrainingDummy demo composition.
2. Replace the single shared AgumonAtlas resource with a generic atlas collection keyed by a presentation id supplied by SpritePresentationEntry. Build atlas handles/layouts for every registered entry whose clip is readable; warnings should include presentation id and path.
3. Extend SpritePresentationEntry with enough registry-owned matching data for spawn_unit_sprites() to choose the correct stance graph, skill graph, atlas image, and clip geometry per Unit without species-specific matches in engine code. Prefer stable UnitId selectors supplied by per-Digimon modules.
4. Add a small windowed demo composition registry/resource (in src/windowed/demo.rs or an equivalent windowed module) that per-Digimon modules populate. Change windowed_bootstrap_system() to build/apply its demo composition from that registry instead of selecting only EncounterPreset::AgumonTrainingDummy.
5. Update src/windowed/digimon/agumon/mod.rs to populate any new SpritePresentationEntry fields and the generic demo registry for Agumon's existing demo behavior.

Done when: the current Agumon demo behavior still has a registered presentation, the engine can host more than one presentation entry, and the source contract proves the remaining engine seams are generic before Renamon is added.

Failure Modes (Q5): Missing clip/atlas assets should keep sprites deferred/blank with a one-time warning, not panic. Empty presentation/demo registries should leave the windowed bootstrap/spawn systems idle. Malformed source contracts should fail cargo test loudly.

Load Profile (Q6): Shared resources are in-memory Bevy registries and asset handles. Per-frame cost should remain small: unit count times registered presentation count, acceptable for the current tiny demo roster; avoid asset reload loops after handles are inserted.

Negative Tests (Q7): Source contract must fail if entries.first(), AgumonAtlas, advance_agumon_presentation, Renamon engine tokens, or hardcoded windowed AgumonTrainingDummy composition return.

## Inputs

- `src/windowed/render.rs`
- `src/windowed/mod.rs`
- `src/windowed/digimon/agumon/mod.rs`
- `src/windowed/digimon/mod.rs`
- `tests/windowed_only.rs`
- `tests/windowed_only/agumon_module_extraction.rs`
- `src/combat/encounter/bootstrap.rs`
- `src/combat/unit.rs`

## Expected Output

- `src/windowed/render.rs`
- `src/windowed/mod.rs`
- `src/windowed/demo.rs`
- `src/windowed/digimon/agumon/mod.rs`
- `tests/windowed_only.rs`
- `tests/windowed_only/renamon_extension_contract.rs`

## Verification

cargo test --features windowed --test windowed_only renamon_extension_contract -- --nocapture

## Observability Impact

Generic atlas/demo warnings should identify presentation id, unit id, and asset path so a future agent can distinguish a bad Renamon asset from a registry wiring issue.
