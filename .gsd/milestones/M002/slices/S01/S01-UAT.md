# S01: Runtime player + sprite render + Stance FSM foundation — UAT

**Milestone:** M002
**UAT Type:** Headless integration + build verification + windowed soak
**Written:** 2026-05-19

## Preconditions

1. `bevyrogue` repository at HEAD of M002/S01 work (all 5 tasks committed)
2. Rust toolchain installed (`cargo`, `rustc`)
3. `cargo-nextest` available
4. Windowed dependencies (wgpu, winit, egui) available for `--features windowed`

## Steps and Expected Outcomes

| # | Step | Command | Expected |
|---|------|---------|----------|
| 1 | Full headless suite | `cargo nextest run --profile agent` | 601 tests, 0 failures |
| 2 | Windowed build | `cargo build --features windowed` | Clean compile, no errors |
| 3 | AnimGraph player FSM | `cargo nextest run --test anim_player_fsm` | 8 tests pass; idle frame indices 54–59; Loop/Hold/SpeedMul honored |
| 4 | Registry hit/miss | `cargo nextest run --test anim_registry` | 5 tests pass; resolve() returns handle on hit, None on miss; registries independent |
| 5 | Stance asset load | `cargo nextest run --test anim_stance_asset` | 3 tests pass; stance.ron parses; agumon_stance resolves via StanceGraphRegistry |
| 6 | Clip↔atlas parity | `cargo nextest run --test clip_atlas_parity` | 2 tests pass; clip.ron matches atlas JSON; `all` range present (0–94) |
| 7 | Gameplay command gate | `cargo nextest run --test anim_gameplay_command_forbidden` | 1 test passes; zero gameplay commands in production Agumon graph |
| 8 | Windowed idle soak | `cargo run --features windowed` | Agumon sprite cycles idle (frames 54–59) via stance graph autonomously; no panic; no hardcoded frame index |

## Edge Cases

- **Idle loop rollover:** Player resets to frame 54 after frame 59 (Loop modifier with count:0)
- **Hold on non-looping animations:** Last frame stays held at sequence end
- **Registry miss:** `resolve()` on unknown id returns None, not a panic
- **Registries are independent:** Stance entries do not appear in SkillGraphRegistry and vice versa
- **Gameplay command rejection:** EmitDamage/EmitStatus/EmitHeal in `on_enter` or `cues` produce Error diagnostics and block validation
- **Clip parity:** `all` range (0–94) is derived from `total_frames` in the tracked atlas JSON; clip ranges for individual animations match atlas exactly

## Not Proven By This UAT

- ReleaseKernelCue integration and KernelCue predicate transitions (S02 scope)
- Multi-unit rendering and dummy opponent (S05 scope)
- Hot-reload correctness mid-skill (S06 scope)
- Renamon stance graph (only Agumon authored in this slice)
- RON editor workflow (future milestone)
- FrameCue dispatch timing at specific animation frames (S02 scope)
