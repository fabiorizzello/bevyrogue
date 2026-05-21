# S04: Baby Burner reactive detonate + flash VFX — UAT

**Milestone:** M002
**Written:** 2026-05-21T06:41:40.878Z

# UAT Type
Agent-driven deterministic regression / closeout UAT.

# Preconditions
- Repository is at the slice closeout HEAD in `/home/fabio/dev/bevyrogue`.
- Rust toolchain and Cargo dependencies are available.
- No desktop compositor or GPU access is required for this UAT.

# Steps
1. Run `cargo test --test agumon_baby_burner_reactive`.
2. Run `cargo test --test unit_died_payload`.
3. Run `cargo test --test timeline_cue_barrier_pipeline` and `cargo test --test timeline_two_clock_parity`.
4. Run `cargo test --features windowed --test windowed_preview_cache`.
5. Run `cargo test --test anim_player_fsm --test anim_graph_asset --test anim_gameplay_command_forbidden --test clip_atlas_parity`.
6. Run `cargo test --lib`.
7. Run `cargo build --no-default-features` and `cargo build --features windowed`.

# Expected Outcomes
1. A lethal Baby Burner hit on a Heated primary target detonates adjacent alive enemies exactly once.
2. The dead primary target is excluded from detonate damage, and already-KO or non-adjacent enemies are excluded.
3. Non-lethal Baby Burner hits, lethal non-Baby-Burner hits, zero-Heated cases, and duplicate update/release paths do not create extra detonate damage.
4. `UnitDied` retains the Heated snapshot needed by the reaction seam.
5. Generic `OnKernelTransition::Blueprint(owner="agumon", name="baby_burner_detonate", ...)` events are observable for real detonation targets.
6. In `windowed` test mode, the Baby Burner flash appears, counts down for a deterministic fixed number of frames, then expires without changing HP or mutating `CombatState`.
7. Animation graph, gameplay-command, and clip-atlas parity regressions stay green.
8. Both headless and `windowed` builds succeed.

# Edge Cases
- Heated primary survives the Baby Burner hit.
- A different killing skill lands the lethal blow.
- Heated remaining is zero at KO time.
- Extra update or release ticks occur after the reactive damage has already resolved.
- Adjacent candidates are already dead or are outside the blast neighborhood.

# Not Proven By This UAT
- A real compositor/GPU-backed live window smoke session; prior attempts inside `gsd_exec` hit `NoCompositor` and missing-GPU sandbox limits.
- Any RON/editor-authored VFX or particle tooling; this slice proves the Rust-only reactive flash path.
- S05 full-kit assembly concerns such as multi-hit loop count or target blink/hurt polish.
