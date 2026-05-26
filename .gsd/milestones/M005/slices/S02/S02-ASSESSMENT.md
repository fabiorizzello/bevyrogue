---
sliceId: S02
uatType: artifact-driven
verdict: PASS
date: 2026-05-26T00:00:00.000Z
---

# UAT Result — S02

## Checks

| Check | Mode | Result | Notes |
|-------|------|--------|-------|
| `cargo test --lib` (headless, 21 tests) | runtime | PASS | `test result: ok. 21 passed; 0 failed` |
| `cargo test --features windowed --lib` (29 tests) | runtime | PASS | `test result: ok. 29 passed; 0 failed` — includes `is_death_reaction_only_matches_unit_died` and `fade_alpha_lerps_full_to_zero` |
| `cargo test --features windowed --bins` (22+2 tests) | runtime | PASS | `test result: ok. 22 passed` (windowed binary) + `ok. 2 passed` (combat_cli) — exit 0 |
| Full integration test suite headless | runtime | PASS | All suites green, exit 0 |
| Full integration test suite windowed (33 windowed-only) | runtime | PASS | `test result: ok. 33 passed; 0 failed` in `windowed_only.rs` |
| `cargo build --features windowed` | runtime | PASS | `Finished dev profile` — exit 0 (0.19s incremental) |
| `cargo build` (headless) | runtime | PASS | `Finished dev profile` — exit 0 (0.14s incremental) |
| dep-leak grep (`bevy::render\|wgpu\|winit\|egui\|bevy_render` in render.rs) | artifact | PASS | `grep` returned no matches — exit 0; no banned dep leaked into windowed crate |
| K001 — visible death-frames-then-fade in `cargo winx` | human-follow-up | NEEDS-HUMAN | Auto-mode stops at the build/test boundary. Human must run `cargo winx`, wait for a unit to reach 0 HP, and confirm: (1) death frames play in full, (2) fade starts after death node exits, (3) entity despawns at alpha 0, (4) non-KO'd sprites unaffected. |

## Overall Verdict

PASS — all 8 automatable checks passed (builds, lib/bin/integration test suites, dep-gating); K001 visual sign-off remains as NEEDS-HUMAN.

## Notes

- Fresh re-run on current HEAD confirms every automated check in the UAT table.
- `cargo test --features windowed --bins` confirmed as 22 (windowed binary) + 2 (combat_cli) = 24 total, matching the UAT's "22+2" claim.
- The 33 windowed-only integration tests in `tests/windowed_only.rs` all pass — covers `windowed_twin_core_badge`, `windowed_preview_cache`, and related suites.
- dep-leak grep against `src/windowed/render.rs` produced zero lines — the windowed crate remains clean of banned render/wgpu/winit symbols at the lib boundary.
- K001 (visual death+fade in cargo winx) is the sole remaining open item. It requires a human to launch the windowed binary and observe the sprite lifecycle. No visual claim is made by auto-mode.

### K001 Manual Sign-off Instructions

1. Run `cargo winx` (or `cargo run --features windowed`)
2. Let the encounter proceed until one unit's HP reaches 0
3. Verify:
   - Death interrupts any in-flight skill animation immediately
   - Death frames (stances 14–22 per authored graph) play completely before fade begins
   - Sprite alpha decreases over ~8 animation ticks (~0.67 s at 12 fps) after death node exits
   - Entity disappears from field when alpha reaches 0 (no ghost sprite remains)
   - The other (surviving) sprite continues idle/hurt/skill animations normally
