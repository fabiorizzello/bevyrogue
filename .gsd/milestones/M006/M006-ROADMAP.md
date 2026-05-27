# M006: M006: Extension-first presentation refactor + enoki-only VFX

**Vision:** Rework the windowed presentation layer so adding a new Digimon needs zero engine edits, the way the combat-logic blueprint layer already works. Retire the custom quad VFX system and make bevy_enoki the sole particle renderer. Replace the ad-hoc, Agumon-coupled flash/shake code with a data-driven CueRegistry of cosmetic primitives (flash/blink, target shake, camera shake, particle burst) modeled on UE5 GAS GameplayCues. Prove it by registering a second Digimon (Renamon) — combat logic and presentation — without touching a single core file.

## Success Criteria

- bevy_enoki is the only particle VFX renderer: the custom quad system (VfxAsset/VfxParticle/resolve_effect/advance_vfx_particles) is deleted and every Agumon effect (charge, ember, projectile, all three impacts) renders through enoki from .particle.ron
- Cosmetic feedback is a data-driven CueRegistry: flash/blink, target sprite-shake, camera-shake, and particle-burst are all parametrized cue primitives addressed by string id, with no hardcoded consts and no AgumonSprite coupling
- Camera-shake exists and is just another registered cue (writes Camera2d transform), not a special case
- The windowed engine contains zero Agumon-specific identifiers: no AGUMON_* consts, no closed on_enter_effect_ids match, no load_agumon_enoki_vfx; AgumonSprite/AgumonPlaybackMode are generalized to a data-carrying DigimonSprite
- Per-digimon presentation lives in src/windowed/digimon/<name>/ and per-digimon logic stays in src/combat/blueprints/<name>/; each registers all its bindings into the registries at startup
- A second Digimon (Renamon) is registered end-to-end — combat logic + presentation — with zero edits to any core/engine file (proven by grep/diff) and K001 manual visual sign-off
- Full cargo test (headless + windowed) and cargo build --features windowed stay green; the dep-gating test still proves no enoki/windowed leak into the headless build

## Slices

- [x] **S01: S01** `risk:high` `depends:[]`
  > After this: In cargo winx, the full Baby Flame sequence (charge, ember, projectile, impact) plus Sharp Claws and Baby Burner all render through enoki; the quad system code is gone; cargo build --features windowed is green and the dep-gating test still passes. (VFX quality is K001 manual.)

- [x] **S02: S02** `risk:medium` `depends:[]`
  > After this: cargo test (headless) covers flash/blink/shake/camera-shake param+decay math and CueRegistry lookup; all green. No windowed wiring yet — this is the generic seam downstream slices consume.

- [x] **S03: S03** `risk:high` `depends:[]`
  > After this: In cargo winx, hits still flash/shake the struck sprite, camera-shake fires on impact, and stance/skill/hurt/death playback still works — now driven by the generic DigimonSprite + cue dispatch instead of Agumon-named components and hit_feedback consts. Windowed regression suite green.

- [x] **S04: S04** `risk:medium` `depends:[]`
  > After this: In cargo winx, Agumon behaves exactly as before, but grep of the windowed engine files shows no AGUMON_* const, no closed on_enter_effect_ids match, and no load_agumon_enoki_vfx — all of it now lives in and is registered by src/windowed/digimon/agumon/. Windowed build/test green.

- [x] **S05: S05** `risk:high` `depends:[]`
  > After this: In cargo winx, Renamon appears as a combatant with working idle/skill/hurt/death presentation and cue-driven flash/shake; git diff shows the only changes are the two new renamon module trees plus their registration call — zero edits to engine/core files. Full cargo test green.

- [x] **S06: S06** `risk:high` `depends:[]`
  > After this: Headless registry test proves every queued graph asset populates; windowed run shows Renamon idle sprite present

- [x] **S07: S07** `risk:medium` `depends:[]`
  > After this: Headless test: Renamon action query returns its real skills, no false MissingSkill

- [x] **S08: S08** `risk:medium` `depends:[]`
  > After this: Renamon cast emits its enoki effect; Agumon cast-driven proof; warn-once on spawn miss

- [x] **S09: S09** `risk:medium` `depends:[]`
  > After this: render.rs imports from registries module; species imports repointed; tests green

- [ ] **S10: S10** `risk:high` `depends:[]`
  > After this: render decomposed; advance_digimon_presentation broken up; source-contract tests adjusted; windowed tests green

- [ ] **S11: Data-driven catalog discovery replacing DEFAULT_ANIM_GRAPH/CLIP/STANCE_PATHS** `risk:medium` `depends:[]`
  > After this: New Digimon discovered from data without editing path constants; headless catalog test

- [ ] **S12: Replace singleton effect registries with keyed-per-effect registries** `risk:medium` `depends:[S10,S11]`
  > After this: DetonateEffectRegistry and residual singletons become keyed; uniform roster presentation data

- [ ] **S13: Port one ranged and one aura/AoE Digimon end-to-end through the windowed seam** `risk:high` `depends:[S08,S11,S12]`
  > After this: Two more Digimon render and act with no engine control-flow edits — real scale proof

- [ ] **S14: VfxAsset to enoki compile/adapt layer making VfxAsset the runtime source of truth** `risk:medium` `depends:[S10,S12]`
  > After this: VfxAsset drives enoki spawn via adapter; decision VfxAsset-canonical recorded first

- [ ] **S15: Prune windowed VFX test churn per spike 4 recommendation** `risk:low` `depends:[S10,S13]`
  > After this: Tier-1 cut vfx_windowed_contracts.rs; Tier-2 thin source-token tests to absence-guards; anti-churn rule appended to DECISIONS.md; windowed tests green before and after

## Boundary Map

## Boundary Map

### S01 → S03/S04
Produces:
- Single enoki spawn path in spawn_effect_by_id (no quad fallback); enoki handle map keyed by effect id
Consumes:
- nothing (independent)

### S02 → S03
Produces:
- CueRegistry type (id → cue definition) and pure cue primitive math (flash/blink, sprite-shake, camera-shake) in the lib
Consumes:
- nothing (independent)

### S03 → S04
Produces:
- Generic DigimonSprite component (graph ids as data) replacing AgumonSprite; windowed cue dispatch reading CueRegistry; camera-shake cue writing Camera2d
Consumes:
- CueRegistry + cue math (S02); single enoki path (S01)

### S04 → S05
Produces:
- src/windowed/digimon/agumon/ register(app) pattern; engine consumes only registries (zero Agumon ids in engine)
Consumes:
- generic DigimonSprite + cue dispatch (S03)

### S05 (acceptance gate)
Produces:
- Renamon combat + presentation modules registered with zero engine edits
Consumes:
- the agumon module pattern + registry seams (S04)
