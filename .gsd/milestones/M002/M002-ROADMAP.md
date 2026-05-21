# M002: M002: M002: M002: M002: First on-screen combat (Agumon-only)

**Vision:** The first time bevyrogue's combat appears on screen: AnimGraph runtime player + wgpu sprite render + §9 UI core + two-clock impact sync, Agumon-only, ending in a playable cargo run --features windowed of Agumon vs an Agumon dummy at full kit, with damage landing on the visible impact frame, gated at closeout by a repomix architectural review. The animation/skill seam must be complete and extensible so M003-M007 and a future RON editor lean on it without rewrites.

## Slices

- [x] **S01: S01** `risk:high` `depends:[]`
  > After this: cargo run --features windowed shows Agumon cycling idle via the stance graph (not hardcoded); M001 headless tests green; clip-atlas geometry parity test present and passing.

- [x] **S02: S02** `risk:high` `depends:[]`
  > After this: Sharp Claws windup to strike to recovery on screen; damage falls on the impact frame via ReleaseKernelCue; telegraph chip visible; I3 extended green.

- [x] **S03: S03** `risk:medium` `depends:[]`
  > After this: §9 phase strip updates from EventReader<CombatEvent>; a structural test asserts the UI path never mutates combat state.

- [x] **S04: S04** `risk:medium` `depends:[]`
  > After this: Baby Burner reactive detonate with a flash VFX (Rust code, no RON/editor); zero non-determinism, R004 intact, headless tests unchanged.

- [x] **S05: S05** `risk:medium` `depends:[]`
  > After this: Agumon vs Agumon dummy at full kit; multi-hit loop visibly = kernel hop count; target blink/hurt driven by CombatEvent.

- [ ] **S06: Windowed smoke end-to-end + repomix review gate** `risk:low` `depends:[S03,S05]`
  > After this: A windowed session with no panic, stable FPS, hot-reload mid-skill not corrupting world state, captured console output; plus a repomix-grounded architectural review report.

## Boundary Map

## Boundary Map
