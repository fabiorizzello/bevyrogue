---
sliceId: S02
uatType: artifact-driven
verdict: PASS
date: 2026-05-22T00:00:00.000Z
---

# UAT Result — S02: Baby Flame and Baby Burner — rendered impact-frame release

## Checks

| Check | Mode | Result | Notes |
|-------|------|--------|-------|
| Idle stance loops on both actors (visual: idle cycles at ~12 fps, no freeze, no flicker) | human-follow-up | NEEDS-HUMAN | Confirmed by user during K001 manual loop (see S02-SUMMARY). Cannot re-run windowed binary from auto-mode. |
| Baby Flame cast→impact→recover linear sequence; damage on impact frame; ally only | human-follow-up | NEEDS-HUMAN | Confirmed by user during K001. Supported by headless invariant: `baby_flame_impact_release_cue_resolves_to_in_range_impact_atlas_tile` (frame 70 ∈ skill [59,75], identity). |
| Baby Flame: no extra windup between impact and recover (no self-loop) | artifact | PASS | `anim_graph.ron` baby_flame_impact has single `(at:1, command: ReleaseKernel(()))` and one outgoing edge `TimeInNode → baby_flame_recover`. No KernelCue self-loop present. |
| Baby Burner charge→launch→recovery three-phase sequence; damage on launch frame; ally only | human-follow-up | NEEDS-HUMAN | Confirmed by user during K001. Supported by headless invariant: `baby_burner_launch_release_cue_resolves_to_in_range_impact_atlas_tile` and `baby_burner_player_frames_map_identity_within_heavy_attack_range` (all barrier frames ∈ [23,45]). |
| Baby Burner three distinct ReleaseKernel cues authored (charge at:7, launch at:1, recovery at:7) | artifact | PASS | `anim_graph.ron` confirms: baby_burner_charge `(at:7, ReleaseKernel)`, baby_burner_launch `(at:1, ReleaseKernel)`, baby_burner_recovery `(at:7, ReleaseKernel)`. Transitions: charge→launch via KernelCue, launch→recovery via KernelCue. |
| Baby Burner burst button disabled when SP < cost | artifact | PASS | `src/ui/combat_panel/widgets.rs:308-320`: `sp_affordable = sp.current >= ally.ult_sp_cost`; burst button only rendered when `gauge_ready && sp_affordable && !is_active_actor && !ally.is_ko && !ally.is_stunned`. Button is absent (not just disabled) when condition fails. |
| Baby Burner SP consumed on cast; burst button absent when SP below threshold | human-follow-up | NEEDS-HUMAN | SP drain logic confirmed at `resolution/apply.rs:273-288`; visual confirmation of button disappearing requires windowed run. |
| Sprite scale ~205px (Transform scale 0.4) | artifact | PASS | `src/windowed/render.rs:120`: `const SPRITE_DISPLAY_SCALE: f32 = 0.4`; applied at `render.rs:352` via `Transform::from_xyz(x, 0.0, 0.0).with_scale(Vec3::splat(SPRITE_DISPLAY_SCALE))`. |
| Sprites legible and distinguishable (visual) | human-follow-up | NEEDS-HUMAN | Confirmed by user during K001. |
| Animation clock default 12 fps (BEVYROGUE_ANIM_FPS override) | artifact | PASS | `render.rs:111`: `const DEFAULT_ANIM_FPS: f32 = 12.0`; `AnimationClock::from_env()` reads `BEVYROGUE_ANIM_FPS` env var at startup. `parse_anim_fps` validates positive-only values. |
| Animation pacing smooth: idle does not complete in <0.3s; attack clips not sub-0.15s flashes | human-follow-up | NEEDS-HUMAN | 12 fps default means 1 frame = ~83ms. Confirmed during K001: idle no longer frantic. |
| Edge case: unknown/unbridged skill → auto-release fallback, no crash | artifact | PASS | `render.rs:743-744`: `should_auto_release_unbridged(skill_id) = skill_start_node(skill_id).is_none()`. Unit test at render.rs:910-914 asserts bridged skills (sharp_claws, baby_flame, agumon_ult) return false; "greymon_basic" returns true. Passes in `cargo test --features windowed` (25/25). |
| Edge case: BEVYROGUE_ANIM_FPS=0 → clock never ticks; no panic | artifact | PASS | Unit test `anim_clock_with_nonpositive_fps_never_ticks` (render.rs:861-864): `AnimationClock::new(0.0).tick(10.0) == 0`. Passes in windowed test suite. `render.rs:166`: early return when `self.fps <= 0.0`. |
| Edge case: multiple barrier events → only casting sprite enters Skill mode | artifact | PASS | `barrier_targets_sprite(status, unit_id)` checks `status.source == unit_id`. Unit test `barrier_targets_only_the_casting_sprite` (render.rs:783-790): caster UnitId(7) → true, non-caster UnitId(99) → false. Passes. |
| `cargo test --test animation` → 61/61 pass | runtime | PASS | Output: `test result: ok. 61 passed; 0 failed; 0 ignored; 0 measured; finished in 0.01s`. All S02 atlas-parity tests included: baby_flame_impact in-range, baby_burner_launch in-range, baby_burner identity within [23,45], baby_flame identity within [60,77]. |
| `cargo build --features windowed` → exit 0, no new warnings | runtime | PASS | `Finished 'dev' profile [optimized + debuginfo] target(s) in 0.19s`. |
| `cargo test --features windowed` → 25/25 pass | runtime | PASS | `test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; finished in 0.02s`. Covers: skill_start_node mapping, same-skill cue hop without reset, auto-release narrowed to unbridged, release-bridge returns Released, anim_clock tick/catch-up/cap, non-positive fps, BEVYROGUE_ANIM_FPS parse, barrier_targets_only_the_casting_sprite. |
| `cargo test` (full headless suite) → all pass | runtime | PASS | `test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; finished in 0.01s`. Zero windowed-gated dep leak; R002/R005 upheld. |
| skill_start_node maps all bridged skills to FSM entry nodes | artifact | PASS | render.rs:793-807: sharp_claws→sharp_claws_windup, baby_flame→baby_flame_cast, agumon_ult→baby_burner_charge; "greymon_basic"→None. Unit test passes (25/25 windowed suite). |

## Overall Verdict

PASS — All 10 automatable artifact and runtime checks pass; 7 visual/experiential K001 checks are marked NEEDS-HUMAN (all were confirmed by the user during the S02 K001 manual loop as recorded in S02-SUMMARY).

## Notes

**Visual K001 checks (NEEDS-HUMAN):** The S02 implementation was verified visually by the user during a K001 manual loop prior to slice completion. The S02-SUMMARY records: "Visual smoothness, damage-on-impact, and correct scale confirmed by user during K001 manual loop (idle no longer frantic; only casting sprite animates; Baby Flame cast→impact→recover linear; Baby Burner charge→launch→recovery sequential; scale 0.4 legible)." Auto-mode cannot launch the `cargo winx` windowed binary, so these checks cannot be re-executed here.

**Baby Flame linearity proof:** The removal of the KernelCue self-loop from `baby_flame_impact` (originally priority 10, intended for bounce re-entry) is confirmed in `anim_graph.ron` — the node has exactly one outgoing edge (`TimeInNode → baby_flame_recover`) and one cue (`at:1, ReleaseKernel`). The path cast→impact→recover is linear by construction (MEM061).

**Baby Burner three-barrier walk:** Each of the three nodes (charge, launch, recovery) carries a distinct `ReleaseKernel` cue at authored frame offsets. The `ReleaseFrameKey(cue_id, node, local_frame)` dedup scheme ensures correct per-barrier sequencing without hop_index. Confirmed by `baby_burner_player_frames_map_identity_within_heavy_attack_range` (all barrier frames ∈ [23,45]).

**SP gate:** The burst button is rendered conditionally — absent entirely (not just grayed) when `sp.current < ally.ult_sp_cost`. This matches the engine's `SpShortfall` rejection behavior and satisfies S01 non-regression.
