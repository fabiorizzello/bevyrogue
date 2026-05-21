# M002: First on-screen combat (Agumon-only)

**Vision:** The first time bevyrogue's combat appears on screen: AnimGraph runtime player + wgpu sprite render + §9 UI core + two-clock impact sync, Agumon-only, ending in a playable cargo run --features windowed of Agumon vs an Agumon dummy at full kit, with damage landing on the visible impact frame, gated at closeout by a repomix architectural review. The animation/skill seam must be complete and extensible so M003-M007 and a future RON editor lean on it without rewrites.

## Slices

- [x] **S01: Runtime player + sprite render + Stance FSM foundation** `risk:high` `depends:[]`
  > After this: cargo run --features windowed shows Agumon cycling idle via the stance graph (not hardcoded); M001 headless tests green; clip-atlas geometry parity test present and passing.

- [x] **S02: Basic attack + two-clock impact barrier + telegraph chip** `risk:high` `depends:[]`
  > After this: Sharp Claws windup to strike to recovery on screen; damage falls on the impact frame via ReleaseKernelCue; telegraph chip visible; I3 extended green.

- [x] **S03: Section 9 phase strip live (event-driven)** `risk:medium` `depends:[]`
  > After this: §9 phase strip updates from EventReader<CombatEvent>; a structural test asserts the UI path never mutates combat state.

- [x] **S04: Baby Burner reactive detonate + flash VFX** `risk:medium` `depends:[]`
  > After this: Baby Burner reactive detonate with a flash VFX (Rust code, no RON/editor); zero non-determinism, R004 intact, headless tests unchanged.

- [x] **S05: Full kit: Agumon vs Agumon dummy** `risk:medium` `depends:[]`
  > After this: Agumon vs Agumon dummy at full kit; multi-hit loop visibly = kernel hop count; target blink/hurt driven by CombatEvent.

- [x] **S06: S06** `risk:low` `depends:[]`
  > After this: A windowed session with no panic, stable FPS, hot-reload mid-skill not corrupting world state, captured console output; plus a repomix-grounded architectural review report.

## Boundary Map

| Producer → Consumer | Surface | Contract |
|---------------------|---------|----------|
| S01 → S02 | Agumon graph/registry/schema groundwork; clip↔atlas parity; baseline windowed stance presentation | `SkillGraphRegistry`, `StanceGraphRegistry`, `AnimationStancePaths`, `clip_atlas_parity` |
| S01 → S03 | `windowed.rs` egui boot + `UiPlugin` split | `EguiPrimaryContextPass` hook surface used by the phase strip |
| S01 → S05 | `AnimGraphPlayer` FSM + windowed render path | Agumon visuals during the full-kit encounter |
| S02 → S03 | `CombatEvent::OnCombatBeat` event stream from the two-clock cue-barrier pipeline | Read-only event source for `PhaseStripDisplay` |
| S02 → S04 | Two-clock cue-barrier contract + `UnitDied` payload semantics | Reaction seam preconditions for post-action dispatch |
| S02 → S05 | Sharp Claws timeline + per-hit cue handshake | Multi-hit loop hop-cue parity (`timeline_loop_hop_cue_parity`) |
| S02 → S06 | Two-clock cue-barrier contract | I3 parity verified by R016 invariant matrix |
| S03 → S05 | Read-only UI ingress pattern (`assert_is_read_only_system`) | Reused by HP bars, floating damage, twin-core badge |
| S03 → S06 | Phase strip live surface | Smoke session visual check |
| S04 → S05 | Owner-neutral post-action reaction seam + `OnKernelTransition::Blueprint` projection | Baby Burner reactive detonate + flash projection end-to-end under full kit |
| S04 → S06 | Reactive detonate + flash VFX path | Smoke session no-panic / no-corruption probe under reactions |
| S05 → S06 | Full-kit Agumon-vs-Agumon-dummy bootstrap (`AGUMON_DUMMY_ID = UnitId(9001)`), HUD HP bars, floating damage, twin-core badge | The session driven by `capture-windowed-smoke.sh` and the UAT runbook; consumed by R016 invariant gate |
