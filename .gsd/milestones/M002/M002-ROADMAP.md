# M002: M002: First on-screen combat (Agumon-only)

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

- [x] **S06: S06** `risk:low` `depends:[]`
  > After this: A windowed session with no panic, stable FPS, hot-reload mid-skill not corrupting world state, captured console output; plus a repomix-grounded architectural review report.

- [x] **S07: S07** `risk:medium` `depends:[]`
  > After this: cargo run --features windowed --bin bevyrogue — barra ult Agumon sale solo da energy, Ultimate si abilita esattamente quando energy=max, fire ult azzera la barra

- [ ] **S08: Remediate graph purity and failure visibility** `risk:high` `depends:[]`
  > After this: After this: R009 has executable proof of typed pure graph input with no world globals or mutable graph context; R013 has structured failure visibility for cue timeout, missing skill-id, hot reload at next spawn, and dead target mid-loop.

- [ ] **S09: Remediate validation evidence and operational closeout** `risk:medium` `depends:[S08]`
  > After this: After this: M002 has an explicit producer consumer boundary map, evidence for stance return and skill graph mapping and VFX handle seam, captured console output, and measured frame-time baseline comparison for the windowed soak.

## Boundary Map

## Boundary Map

Not provided.
