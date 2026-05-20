# S02: Basic attack + two-clock impact barrier + telegraph chip — UAT

**Milestone:** M002
**Written:** 2026-05-20T16:26:23.754Z

# UAT Type
Engineering closeout / automated integration proof with optional local visual smoke.

# Preconditions
1. Repository is at the S02 closeout commit/state.
2. Rust/Cargo toolchain is installed.
3. For optional visual proof, the machine has a usable display session and GPU for Bevy window creation.

# Steps
1. Run `cargo test --test timeline_two_clock_parity`.
2. Run `cargo test --test anim_player_fsm --test anim_graph_asset --test anim_gameplay_command_forbidden --test clip_atlas_parity`.
3. Run `cargo test --test agumon_sharp_claws_asset`.
4. Run `cargo test --test timeline_cue_barrier_pipeline`.
5. Run `cargo test --features windowed --test windowed_preview_cache`.
6. Run `cargo test --lib`.
7. Run `cargo build --no-default-features`.
8. Run `cargo build --features windowed`.
9. If a display/GPU is available, run `BEVYROGUE_VALIDATION_WINDOWED=1 BEVYROGUE_VALIDATION_WINDOWED_SOAK_SECS=1 cargo run --features windowed --bin bevyrogue`.
10. In the windowed run, select Agumon Basic against a legal target and observe the playback.

# Expected Outcomes
1. Timeline parity proves headless auto mode and windowed manual cue release end with identical intent streams.
2. Animation tests prove KernelCue transitions do not fire before signal, do fire after signal, and consume once.
3. Asset tests prove Agumon Basic points to `sharp_claws`, the timeline carries the impact damage payload plus presentation cue metadata, and Baby Flame still parses.
4. Cue-barrier pipeline tests prove damage/event application is deferred while awaiting, releases resume exactly once, and queued actions are ignored while resolving.
5. Windowed preview tests prove the telegraph chip reports awaiting Sharp Claws impact and hides after release/resolution; Basic preview damage matches Sharp Claws data.
6. Both headless and windowed builds succeed.
7. In a display-capable visual run, Agumon visibly plays windup -> strike -> recovery; damage lands on the impact frame rather than before the strike; the telegraph chip is visible while impact is pending and clears after release.

# Edge Cases
1. Calling the release API before any timeline is suspended should be a harmless no-op.
2. Releasing the same barrier twice must not duplicate damage.
3. Windowed execution must not advance past the barrier without an explicit release.
4. Missing display/GPU capability is an environment limitation, not a product failure; rely on the passing windowed build/test suite in that case.

# Not Proven By This UAT
1. A live visual soak was not proven in this closeout environment because the verification lane had no display/GPU-capable session.
2. Later milestone behavior outside Sharp Claws first-hit flow (Section 9 event strip, Baby Burner reactive VFX, full-kit multi-hit combat, repomix review gate) is not covered here.
