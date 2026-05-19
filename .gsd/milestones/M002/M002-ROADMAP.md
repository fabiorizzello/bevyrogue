# M002: First on-screen combat (Agumon-only)

**Vision:** The first time bevyrogue's combat appears on screen: AnimGraph runtime player + wgpu sprite render + §9 UI core + two-clock impact sync, Agumon-only, ending in a playable cargo run --features windowed of Agumon vs an Agumon dummy at full kit, with damage landing on the visible impact frame, gated at closeout by a repomix architectural review. The animation/skill seam must be complete and extensible so M003-M007 and a future RON editor lean on it without rewrites.

## Success Criteria

- cargo run --features windowed runs Agumon vs Agumon dummy at full kit with no panic and stable (qualitative) FPS
- Damage lands on the visible impact frame via the player-side two-clock barrier; invariant I3 extended to the cue handshake stays green
- Zero gameplay numbers in anim_graph.ron, enforced by an executable anti-DRY test; the M001 mul:18 duplicate remediated behind it
- Multi-hit loop visibly equals the kernel hop count with no N authored in the anim graph
- All M001 headless Agumon tests stay green; R002/R004/R005/R006 hold
- Repomix architectural review report produced at closeout and findings triaged

## Slices

- [ ] **S01: Runtime player + sprite render + Stance FSM foundation** `risk:high` `depends:[]`
  > After this: cargo run --features windowed shows Agumon cycling idle via the stance graph (not hardcoded); M001 headless tests green; clip-atlas geometry parity test present and passing.

- [ ] **S02: Basic attack + two-clock impact barrier + telegraph chip** `risk:high` `depends:[S01]`
  > After this: Sharp Claws windup to strike to recovery on screen; damage falls on the impact frame via ReleaseKernelCue; telegraph chip visible; I3 extended green (identical Intent stream headless vs windowed, only timing differs).

- [ ] **S03: Section 9 phase strip live (event-driven)** `risk:medium` `depends:[S01]`
  > After this: §9 phase strip updates from EventReader<CombatEvent>; a structural test asserts the UI path never mutates combat state (D008).

- [ ] **S04: Baby Burner reactive detonate + flash VFX** `risk:medium` `depends:[S02]`
  > After this: Baby Burner reactive detonate with a flash VFX (Rust code, no RON/editor); zero non-determinism, R004 intact, headless tests unchanged.

- [ ] **S05: Full kit: Agumon vs Agumon dummy** `risk:medium` `depends:[S02,S04]`
  > After this: Agumon vs Agumon dummy at full kit (Sharp Claws + Baby Flame + Baby Burner + Twin Core fire side via placeholder ally); multi-hit loop visibly = kernel hop count (no N authored in the anim graph); target blink/hurt driven by CombatEvent.

- [ ] **S06: Windowed smoke end-to-end + repomix review gate** `risk:low` `depends:[S03,S05]`
  > After this: A windowed session with no panic, stable FPS, hot-reload mid-skill not corrupting world state, captured console output; plus a repomix-grounded architectural review report.

## Boundary Map

### S01 to S02
Produces:
- `RenderPlugin`/`UiPlugin` split gated `#[cfg(feature="windowed")]`; pre-split `animation/plugin.rs`
- `AnimNode.cues: Vec<FrameCue>`, `FrameCue{at,command}`, `ReleaseKernelCue`, `Predicate::KernelCue` (closed enums, `#[serde(default)]`, no untagged)
- `SkillGraphRegistry`+`StanceGraphRegistry` (skill-id to graph), `AnimGraph` id field
- AnimGraph runtime player driving a wgpu sprite from the stance graph
- `GameplayCommandForbidden` validation check + test; `mul:18` remediated behind it
Consumes:
- nothing (first slice)

### S01 to S03
Produces:
- `RenderPlugin`/`UiPlugin` split; egui surface for the §9 strip
Consumes:
- nothing (first slice)

### S02 to S04
Produces:
- Two-clock barrier: `Clock` wired into the turn pipeline (per-frame `step()` suspending `CombatPhase` on `AwaitingCue`); player calls `resume_cue()` at the authored frame
- Typed graph input (`Role` enum, read-only, pure-function evaluation)
Consumes:
- S01 player + `cues`/`ReleaseKernelCue` + registries

### S02 to S05
Produces:
- Per-hit cue handshake (`when: KernelCue` self-loop on the strike node)
Consumes:
- S01 player + closed-enum extensions; S02 barrier

### S04 to S05
Produces:
- `SpawnVfx` opaque-`Id` VFX entity (Rust-configured), self-clocked fire-and-forget
Consumes:
- S02 cue handshake

### S03,S05 to S06
Produces:
- Assembled windowed runtime: full Agumon kit vs dummy, live §9 strip, target reactions
Consumes:
- S03 phase strip; S05 full kit + multi-hit loop
