# S05: Second digimon (Renamon) with zero engine edits

**Goal:** Register Renamon end-to-end in the windowed demo while preserving the extension-first presentation boundary. Because planning verified hidden S04 blockers, this slice first closes species-agnostic multi-presentation and demo-composition seams, then proves the Renamon-specific work is limited to Renamon assets, src/windowed/digimon/renamon/, and the digimon aggregator registration.
**Demo:** In cargo winx, Renamon appears as a combatant with working idle/skill/hurt/death presentation and cue-driven flash/shake; git diff shows the only changes are the two new renamon module trees plus their registration call — zero edits to engine/core files. Full cargo test green.

## Must-Haves

- Must-haves:
- Windowed presentation supports multiple SpritePresentationRegistry entries instead of presentation.entries.first(), with per-unit atlas/graph selection and no engine-resident Renamon tokens.
- Engine/windowed generic files no longer carry Agumon-specific names such as AgumonAtlas or advance_agumon_presentation, and windowed bootstrap no longer hardcodes EncounterPreset::AgumonTrainingDummy as the only demo composition.
- Renamon has a stance graph, all-range clip coverage, a bridged diamond_storm skill graph with ReleaseKernel, sprite presentation registration, skill start-node registration, and windowed demo registration.
- Source-contract tests assert Renamon lives in src/windowed/digimon/renamon/ plus assets and that engine/core files remain species-agnostic.
- Full verification gates pass: cargo test, cargo test --features windowed --test windowed_only, cargo test --test dependency_gating, and RUSTFLAGS='-D warnings' cargo build --features windowed.
- Threat Surface (Q3): No auth, network, secrets, or user-provided data are introduced. The only input trust boundary is local RON/asset loading; malformed or missing assets must produce existing warnings/fallbacks rather than panics.
- Requirement Impact (Q4): Supports the milestone-local extension-first constraint and R002/R005 headless/dep-gating. Re-verify dependency_gating so bevy_enoki/windowed symbols stay out of headless paths. Decision revisited/recorded: D051.

## Proof Level

- This slice proves: Final-assembly contract plus build/test proof. Auto-mode can prove source contracts, asset parse/build, and headless/windowed test gates; live cargo winx visual quality remains K001 manual sign-off.

## Integration Closure

Consumes S04's per-Digimon register(app) seam, S03's generic DigimonSprite/cue dispatch, S02's CueRegistry, and S01's enoki-only VFX path. Introduces multi-entry sprite/atlas selection, registry-driven windowed demo composition, and Renamon presentation registration. After this slice, the milestone is structurally usable end-to-end except for K001 human visual sign-off in the live window.

## Verification

- Keeps and generalizes windowed presentation diagnostics so missing graph/clip/atlas/effect failures are inspectable by presentation id/path rather than silently blank. Source-contract tests become the durable CI signal for future agents that engine files remain species-agnostic and Renamon is an extension module.

## Tasks

- [x] **T01: Generalize multi-presentation and demo composition seams** `est:2h`
  Expected executor skills/frontmatter: rust-development, rust-skills, bevy, tdd, write-docs, verify-before-complete.
  - Files: `src/windowed/render.rs`, `src/windowed/mod.rs`, `src/windowed/demo.rs`, `src/windowed/digimon/agumon/mod.rs`, `tests/windowed_only.rs`, `tests/windowed_only/renamon_extension_contract.rs`
  - Verify: cargo test --features windowed --test windowed_only renamon_extension_contract -- --nocapture

- [ ] **T02: Add Renamon presentation module and animation assets** `est:2h`
  Expected executor skills/frontmatter: rust-development, rust-skills, bevy, tdd, write-docs, verify-before-complete.
  - Files: `src/windowed/digimon/renamon/mod.rs`, `src/windowed/digimon/mod.rs`, `assets/digimon/renamon/stance.ron`, `assets/digimon/renamon/clip.ron`, `assets/digimon/renamon/anim_graph.ron`, `tests/windowed_only/renamon_extension_contract.rs`
  - Verify: cargo test --features windowed --test windowed_only renamon_extension_contract -- --nocapture

- [ ] **T03: Harden Renamon zero-engine-edit source contracts** `est:1h`
  Expected executor skills/frontmatter: rust-development, rust-testing, bevy, tdd, write-docs, verify-before-complete.
  - Files: `tests/windowed_only/renamon_extension_contract.rs`, `tests/windowed_only/agumon_module_extraction.rs`, `tests/windowed_only.rs`
  - Verify: cargo test --features windowed --test windowed_only -- --nocapture

- [ ] **T04: Run full slice verification gates** `est:1h`
  Expected executor skills/frontmatter: rust-development, cargo-nextest if desired, bevy, verify-before-complete.
  - Verify: cargo test
cargo test --features windowed --test windowed_only
cargo test --test dependency_gating
RUSTFLAGS='-D warnings' cargo build --features windowed

## Files Likely Touched

- src/windowed/render.rs
- src/windowed/mod.rs
- src/windowed/demo.rs
- src/windowed/digimon/agumon/mod.rs
- tests/windowed_only.rs
- tests/windowed_only/renamon_extension_contract.rs
- src/windowed/digimon/renamon/mod.rs
- src/windowed/digimon/mod.rs
- assets/digimon/renamon/stance.ron
- assets/digimon/renamon/clip.ron
- assets/digimon/renamon/anim_graph.ron
- tests/windowed_only/agumon_module_extraction.rs
