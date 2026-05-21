---
sliceId: S04
uatType: artifact-driven
verdict: PASS
date: 2026-05-21T00:00:00.000Z
---

# UAT Result — S04

## Checks

| Check | Mode | Result | Notes |
|-------|------|--------|-------|
| `cargo test --test agumon_baby_burner_reactive` | runtime | PASS | 5/5 tests: lethal Heated Baby Burner detonates adjacent alive enemies exactly once; non-lethal, non-Baby-Burner, and zero-Heated cases produce no detonation; KO context properly mapped to adjacent targets |
| `cargo test --test unit_died_payload` | runtime | PASS | 2/2 tests: `UnitDied` carries `heated_remaining` snapshot; not emitted on survival |
| `cargo test --test timeline_cue_barrier_pipeline` | runtime | PASS | 5/5 tests: suspension/resume contract preserved; duplicate-release is no-op; spam ignored while barrier resolves; windowed equals headless at completion |
| `cargo test --test timeline_two_clock_parity` | runtime | PASS | 2/2 tests: stray `resume_cue()` without awaiting is harmless; headless auto equals windowed manual cue handshake |
| `cargo test --features windowed --test windowed_preview_cache` | runtime | PASS | 4/4 tests: `BabyBurnerFlashState` projects detonate transitions for fixed frames without touching HP; windowed preview uses Sharp Claws damage data; cache tracks shared preview summary; telegraph chip helpers surface/hide barriers |
| `cargo test --test anim_player_fsm --test anim_graph_asset --test anim_gameplay_command_forbidden --test clip_atlas_parity` | runtime | PASS | 18/18 tests across 4 suites: animation FSM, graph asset parse (Agumon + Renamon), gameplay command forbidden paths, clip-atlas parity all green |
| `cargo test --lib` | runtime | PASS | 0 lib unit tests registered; clean exit (no regressions introduced) |
| `cargo build --no-default-features` | runtime | PASS | Headless build compiles clean |
| `cargo build --features windowed` | runtime | PASS | Windowed feature build compiles clean |

## Overall Verdict

PASS — All 9 automated checks passed across the full S04 regression/build matrix: reactive detonate behavior, payload semantics, timeline barrier contracts, windowed flash projection, animation/atlas parity, lib tests, and both build targets.

## Notes

- **Step 1 (agumon_baby_burner_reactive):** Confirmed adjacency-only one-shot detonate behavior and negative cases (non-lethal hit, non-Baby-Burner kill, zero-Heated, duplicate update/release). Generic `OnKernelTransition::Blueprint(owner="agumon", name="baby_burner_detonate")` observability verified.
- **Step 2 (unit_died_payload):** R002 preserved — `UnitDied { heated_remaining }` carries the post-KO reaction context needed by the seam.
- **Step 3 (timeline tests):** R004 preserved — deterministic suspension/resume and exact-once semantics hold; duplicate-release and stray `resume_cue()` paths are safe no-ops.
- **Step 4 (windowed_preview_cache):** R005 preserved — `BabyBurnerFlashState` fixed-frame flash lifecycle is projection-only; `CombatState` and HP unchanged. Feature gate is correctly `#[cfg(feature = "windowed")]`.
- **Step 5 (anim/atlas tests):** R003 preserved — animation graph, gameplay command, and clip-atlas parity regressions stay green.
- **Step 6 (lib):** Clean; no lib-level unit test regressions.
- **Step 7 (builds):** Both headless (`--no-default-features`) and windowed (`--features windowed`) builds succeed, confirming dependency gating is correct.
- **Real-window smoke:** Not automated (recorded as not proven in the UAT preconditions). Wayland has no compositor and X11 lacks a GPU in the `gsd_exec` sandbox. This is an execution-environment limitation, not a product failure. The deterministic test/build matrix is the canonical closeout evidence.
