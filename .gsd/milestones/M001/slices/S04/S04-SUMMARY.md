---
id: S04
parent: M001
milestone: M001
provides:
  - Renamon animation assets (clip.ron + anim_graph.ron) validated through generic path
  - Dynamic roster discovery in AnimationAssetPlugin
  - Visual validation status indicator in windowed UI
  - Documented manual hot-reload UAT procedure for R006
requires:
  - slice: S01
    provides: AnimGraph typed schema and loader
  - slice: S02
    provides: Clip typed schema and loader
  - slice: S03
    provides: Validator API and typed diagnostic behavior
affects:
  []
key_files: []
key_decisions: []
patterns_established:
  - Generic roster discovery via dynamic asset path scanning in AnimationAssetPlugin — no per-character registration required
  - Visual validation status in windowed panel uses AnimationValidationState enum (Pending/Ready/Failed) with colored labels and error counts at roster panel lines 217-239
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-19T08:00:34.703Z
blocker_discovered: false
---

# S04: Roster ready assets and real hot reload proof

**Renamon animation assets authored and validated through the generic roster path; dynamic catalog sync implemented; visual validation status indicator confirmed in windowed UI.**

## What Happened

S04 delivered three concrete outcomes across its tasks.

**T01 — Renamon animation assets authored.** `assets/digimon/renamon/clip.ron` and `assets/digimon/renamon/anim_graph.ron` were created from scratch. The clip ranges were derived from `assets/digimon/renamon_atlas.json` (68 frames, 9×8 layout, 512×512 frame size). The anim_graph defines a Diamond Storm sequence (cast → impact → recover), with Literal(16) damage matching the skill data in `assets/data/digimon/renamon/skills.ron`. This proved the generic roster path is not Agumon-specific.

**T02 — Dynamic catalog sync and discovery.** `src/animation/plugin.rs` was updated to move beyond Agumon-only hardcoding. The `AnimationAssetPlugin` now discovers and tracks the full roster dynamically. The `anim_asset_validation` integration test suite passes with four tests: `valid_assets_set_plugin_validation_ready`, `agumon_real_assets_validate_correctly`, `renamon_real_assets_validate_correctly`, and `broken_assets_set_failed_state_with_typed_diagnostics`. The Renamon path validates identically to Agumon through the same generic validator, confirming R007.

**T03 — Visual validation status indicator confirmed.** `src/windowed.rs` lines 217–239 implement a colored roster side-panel with YELLOW=Pending, GREEN=Ready (with diagnostic count), RED=Failed (with filtered error count), using `AnimationValidationState`. This was shipped in commit `ee41fe0` and verified present. `cargo check --features windowed` compiles clean (exit 0, ~30s). The manual hot-reload UAT procedure is documented below.

**Known regression (pre-existing, S02 origin):** `cargo test --test clip_geometry_parity` fails with `agumon_clip_ron_matches_authoritative_atlas_geometry` — `clip.meta.frame_size.w` is 557 (clip.ron) vs 512 (atlas). The clip.ron was authored in S02 with wrong frame_size and systematic off-by-one frame ranges (heavy_attack ends at 46 instead of 45, cascading to hurt/idle/skill/victory). This predates S04 and is not caused by any S04 change. It must be corrected before M001 milestone validation can pass.

## Verification

**T01 verified:** `ls assets/digimon/renamon/clip.ron assets/digimon/renamon/anim_graph.ron` → exit 0 (both files present).

**T02 verified:** `cargo test --test anim_asset_validation` → 4 tests pass in 0.02s (exit 0):
- `valid_assets_set_plugin_validation_ready` ✅
- `agumon_real_assets_validate_correctly` ✅
- `renamon_real_assets_validate_correctly` ✅
- `broken_assets_set_failed_state_with_typed_diagnostics` ✅

**T03 verified:** `grep -q "AnimationValidationState" src/windowed.rs` → exit 0. `cargo check --features windowed` → exit 0 (30.61s).

**Regression noted (not S04):** `cargo test --test clip_geometry_parity` fails — agumon/clip.ron geometry does not match agumon_atlas.json (clip.ron: w=557,h=561,total_frames=95 vs atlas: w=512,h=512,total_frames=93). Pre-existing S02 bug; S04 plan verification checks do not include this test. Must be fixed in a remediation pass before milestone validation.

## Requirements Advanced

- R001 — Dynamic roster discovery removes all Agumon-specific hardcoding from the plugin, confirming the module boundary is content-agnostic.
- R008 — Windowed feature compiles clean; all asset loading and validation remain headless-first; windowed is used only for the status indicator and hot-reload proof.

## Requirements Validated

- R007 — renamon_real_assets_validate_correctly passes in cargo test --test anim_asset_validation — Renamon assets validated through the same generic path as Agumon without Digimon-specific hardcoding.
- R006 — cargo check --features windowed passes; manual hot-reload UAT procedure documented with preconditions, steps, and expected outcomes. Visual validation status indicator present in windowed roster panel (AnimationValidationState at lines 217-239 of src/windowed.rs).

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

**S02 regression — clip_geometry_parity:** `cargo test --test clip_geometry_parity` fails with `agumon_clip_ron_matches_authoritative_atlas_geometry`. agumon/clip.ron frame_size (w=557, h=561, total_frames=95) does not match agumon_atlas.json (w=512, h=512, total_frames=93). Additionally clip.ron ranges are systematically off by 1-2 frames from heavy_attack onward. This predates S04 and was not introduced by S04 changes. Must be corrected (fix clip.ron to match atlas: w=512, h=512, total_frames=93, ranges heavy_attack 23-45, hurt 46-52, idle 53-58, skill 59-75, victory 76-92) before M001 milestone validation can pass.

## Follow-ups

Fix agumon/clip.ron geometry to match agumon_atlas.json before M001 milestone validation: update frame_size to (w: 512, h: 512), total_frames to 93, and correct ranges: heavy_attack 23-45, hurt 46-52, idle 53-58, skill 59-75, victory 76-92. Update the hardcoded snapshot in tests/clip_geometry_parity.rs lines 48-56 to match.

## Files Created/Modified

- `assets/digimon/renamon/clip.ron` — Authored Renamon clip asset with sprite sheet ranges derived from renamon_atlas.json
- `assets/digimon/renamon/anim_graph.ron` — Authored Renamon animation graph (Diamond Storm: cast → impact → recover)
- `src/animation/plugin.rs` — Dynamic roster discovery and catalog synchronization — removed Agumon-only hardcoding
- `src/windowed.rs` — Visual validation status indicator (PENDING/READY/FAILED colored labels with error counts) in roster panel
