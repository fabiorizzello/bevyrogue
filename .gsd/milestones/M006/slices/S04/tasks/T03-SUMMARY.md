---
id: T03
parent: S04
milestone: M006
key_files: []
key_decisions:
  - SkillStartNodeRegistry and SpritePresentationRegistry are the engine-generic seam for per-Digimon FSM entry-node lookup and sprite/atlas presentation wiring; species-specific data stays in src/windowed/digimon/agumon/mod.rs.
  - Before adding Renamon-specific windowed presentation code, refactor oversized non-test windowed files first, especially src/windowed/render.rs (D050).
duration: 
verification_result: passed
completed_at: 2026-05-26T13:48:15.670Z
blocker_discovered: false
---

# T03: Verified and recorded the completed Agumon registry migration: skill start-node and sprite presentation data now live in src/windowed/digimon/agumon/mod.rs, and the windowed engine files are free of AGUMON_* consts, residual skill_start_node helpers, and hardcoded atlas-path wiring.

**Verified and recorded the completed Agumon registry migration: skill start-node and sprite presentation data now live in src/windowed/digimon/agumon/mod.rs, and the windowed engine files are free of AGUMON_* consts, residual skill_start_node helpers, and hardcoded atlas-path wiring.**

## What Happened

When I entered T03, the planned refactor was already present on disk but its canonical completion artifact had not been written. I verified that src/windowed/render.rs now defines and initializes the generic SkillStartNodeRegistry and SpritePresentationRegistry, and that the windowed engine reads those registries in the skill sync / auto-release path, atlas construction path, and sprite spawn path instead of consulting Agumon-specific constants or a closed skill_start_node helper. I also verified that src/windowed/digimon/agumon/mod.rs owns the moved stance/skill graph ids, skill/node vocabulary, atlas image path, clip index, and registry-population startup systems. No code edits were required in this recovery pass; the work here was fresh verification plus writing the missing DB-backed task completion artifact.

## Verification

Fresh verification passed in this run: RUSTFLAGS='-D warnings' cargo build --features windowed; cargo test --features windowed --test windowed_only; cargo test --test dependency_gating; and a direct grep-gate assertion proving the engine files contain no AGUMON_* tokens, no fn skill_start_node helper, and no hardcoded digimon/agumon_atlas.png path while the Agumon module still owns the registry registration entry points.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `RUSTFLAGS='-D warnings' cargo build --features windowed` | 0 | ✅ pass | 334ms |
| 2 | `cargo test --features windowed --test windowed_only` | 0 | ✅ pass | 471ms |
| 3 | `cargo test --test dependency_gating` | 0 | ✅ pass | 586ms |
| 4 | `python: assert no AGUMON_* consts or fn skill_start_node in src/windowed/render.rs/src/windowed/mod.rs, no digimon/agumon_atlas.png in src/windowed/render.rs, and registration entry points remain in src/windowed/digimon/agumon/mod.rs` | 0 | ✅ pass | 22ms |

## Deviations

The implementation was already present on disk before this execution turn, so this pass performed verification and state recording rather than additional source edits.

## Known Issues

src/windowed/render.rs remains a large non-test source file; per user direction captured in D050/MEM121, the next step before Renamon work should be a structural refactor that splits generic presentation responsibilities into smaller modules.

## Files Created/Modified

None.
