---
estimated_steps: 12
estimated_files: 3
skills_used: []
---

# T03: Harden Renamon zero-engine-edit source contracts

Expected executor skills/frontmatter: rust-development, rust-testing, bevy, tdd, write-docs, verify-before-complete.

Why: S05's key proof is structural: after the generic seam repair, adding Renamon must not introduce Renamon-specific branches in engine/core files. Because src/windowed/ is binary-only and K001 live visuals require a human, source-contract tests are the durable automated proof.

Do:
1. Finish tests/windowed_only/renamon_extension_contract.rs so it include_str! checks source/assets only, never .git or .gsd. It should assert that src/windowed/render.rs and src/windowed/mod.rs do not contain renamon, RENAMON_, diamond_storm, or other Renamon-owned identifiers.
2. Assert src/windowed/digimon/mod.rs declares mod renamon and calls renamon::register(app), while src/windowed/digimon/renamon/mod.rs owns SpritePresentationRegistry, SkillStartNodeRegistry, AnimationStancePaths, digimon/renamon_atlas.png, renamon_stance, renamon_skill, and diamond_storm tokens.
3. Assert Renamon assets contain the required contracts: stance.ron id renamon_stance, clip.ron all range, anim_graph.ron ReleaseKernel cue, and graph id renamon_skill.
4. Extend or adjust the S04 Agumon extraction test only if T01's generic renames require token updates; do not weaken the original promise that Agumon-specific data belongs under src/windowed/digimon/agumon/.
5. Run the full windowed-only integration test target so existing S01-S04 contracts still pass.

Done when: the Renamon contract fails on any engine-level Renamon hardcoding and passes with Renamon confined to its module/assets plus the aggregator registration.

Failure Modes (Q5): Static tests can become brittle if they pin formatting or numeric values; assert semantic tokens and forbidden ownership boundaries only. Tests must not inspect git diff or ignored planning directories.

Load Profile (Q6): Test-only include_str! reads a small fixed set of source/assets at compile time; no runtime shared-resource impact.

Negative Tests (Q7): The test should catch absent Renamon module registration, missing ReleaseKernel, missing all clip range, accidental Renamon token in engine files, and regression to single-entry presentation lookups.

## Inputs

- `tests/windowed_only/renamon_extension_contract.rs`
- `tests/windowed_only/agumon_module_extraction.rs`
- `tests/windowed_only.rs`
- `src/windowed/render.rs`
- `src/windowed/mod.rs`
- `src/windowed/digimon/mod.rs`
- `src/windowed/digimon/agumon/mod.rs`
- `src/windowed/digimon/renamon/mod.rs`
- `assets/digimon/renamon/stance.ron`
- `assets/digimon/renamon/clip.ron`
- `assets/digimon/renamon/anim_graph.ron`

## Expected Output

- `tests/windowed_only/renamon_extension_contract.rs`
- `tests/windowed_only/agumon_module_extraction.rs`
- `tests/windowed_only.rs`

## Verification

cargo test --features windowed --test windowed_only -- --nocapture

## Observability Impact

Adds an automated CI-visible boundary signal for future species work: engine recoupling to Renamon or single-entry lookups fails immediately with a targeted assertion.
