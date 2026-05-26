# M004: Per-Digimon data-driven VFX (owned, extension-first)

**Vision:** Replace the hardcoded three-phase Baby Flame VFX polish in src/windowed/render.rs (the VfxParticleKind enum, kind_from_name string-match, and per-skill offset fns) with the owned per-Digimon vfx.ron seam decided in D033/D034, before the M005+ roster multiplies the hardcoding. The engine exposes primitives (placement/appearance/variation verbs in the existing Registry<E>, curve eval, predicates) plus extension points; a Digimon brings its own assets/digimon/<name>/vfx.ron and, when a motion is novel, its own register() in its blueprint. The schema is editor-ready from S01 (typed introspectable verb params + Reflect + round-trip), so the future single GUI editor for anim_graph + vfx never forces a schema refactor. All verb math is pure and headless-testable (R004); only rendering is windowed-gated (R002/R005).

## Success Criteria

- Zero hardcoded VFX-kind paths remain in src/windowed/render.rs (VfxParticleKind enum and kind_from_name string-match removed)
- Every Agumon effect — Baby Flame charge/launch/impact and Baby Burner detonate — is expressed in assets/digimon/agumon/vfx.ron
- Adding an effect that reuses existing verbs is writing RON only; a novel motion is one register("ns/name", fn) in the Digimon blueprint, with no core change
- All placement/appearance verb math is headless-tested and deterministic (R004)
- The vfx.ron schema is editor-ready: verb parameters are typed and introspectable (Serialize+Deserialize+Reflect), not a stringly-typed map

## Slices

- [x] **S01: S01** `risk:high` `depends:[]`
  > After this: Headless test loads assets/digimon/agumon/vfx.ron into a typed VfxAsset and evaluates appearance scale/color keyframe curves deterministically; Baby Flame impact fan-out renders from the data path in cargo winx.

- [x] **S02: S02** `risk:medium` `depends:[]`
  > After this: cargo winx shows Baby Flame charge ember-swirl and fast launch rendered through Registry-resolved placement verbs; a static grep confirms VfxParticleKind and kind_from_name no longer exist in render.rs.

- [x] **S03: S03** `risk:medium` `depends:[]`
  > After this: Headless test maps a VfxContext (e.g. a skill-tree unlock) to a selected effect-tree variant deterministically; cargo winx shows Baby Burner detonate rendered from assets/digimon/agumon/vfx.ron with no hardcoded VFX paths left in render.rs.

- [x] **S04: S04** `risk:medium` `depends:[]`
  > After this: After this: M004 states exactly which requirements it validates, the boundary map is populated, S03 declares its consumed S01 and S02 contracts, and validation no longer flags documentation or scope ambiguity.

- [x] **S05: S05** `risk:high` `depends:[]`
  > After this: After this: Sharp Claws VFX is either authored and tested in assets/digimon/agumon/vfx.ron or explicitly rescoped, HDR bloom additive rendering criteria are implemented or rescoped, and automated evidence covers the chosen outcome.

- [x] **S06: S06** `risk:medium` `depends:[]`
  > After this: After this: human cargo winx UAT for Baby Flame, Baby Burner, and Sharp Claws is captured or formally waived, with no remaining unchecked visual signoff requirements for M004.

- [x] **S07: S07** `risk:medium` `depends:[]`
  > After this: After this: M004 validation can be rerun with scoped requirement coverage, an inline or canonical boundary map, variant seam disposition, S06 assessment evidence, and human visual UAT signed off or formally waived.

## Boundary Map

Not provided.
