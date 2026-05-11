# S03 Research: Skill DSL targeting and legality metadata

## Summary

S03 owns the data/schema part of R084 and supports R085. The slice should not build the full pure query API yet; it should make skill intent explicit enough that S04 can query it without guessing from effect IDs or UI rules.

Current state is effect-inferred:

- `src/data/skills_ron.rs` defines `SkillDef { id, name, damage_tag, sp_cost, effects }`, `Effect`, and `TargetShape`.
- `TargetShape` currently appears only inside `Effect::Damage { target }`.
- `src/combat/resolution.rs::resolve_action` flattens effects into `ResolvedAction` and derives `target_shape` by scanning the first `Damage` effect; no-damage skills default to `Single`.
- S02 added `target_shape_rejection_reason()` and `step_declaration` rejection for non-`Single` shapes before mutation.
- There is no first-class side/life/self target policy in skill data. Revive legality is inferred from `Effect::Revive`, and engine code currently checks KO/live state but not team side.

Canonical `assets/data/skills.ron` currently has 72 skills: 59 `Single` damage entries, 6 `Row` damage entries, 3 revive-containing entries, 1 mixed damage+revive entry (`angemon_ult`), and 7 no-damage entries (`patamon_revive`, `first_aid`, four form-identity modifiers, one `ToughnessHit`-only form identity). This is the main migration surface.

## Requirements targeted

- **R084**: S03 must make action/target validity data-driven by adding skill targeting/legalities metadata to the DSL. S04/S06 will consume it, but S03 must stop relying on effect-shape inference as the only source of truth.
- **R085**: S03 must explicitly represent unsupported/non-truthful mechanics in data, especially non-single target shapes and mixed-effect target semantics, so UI-readiness work can return `Deferred`/`Disabled` instead of silently exposing false affordances.

Relevant loaded rules/memories:

- D053/MEM062: legality lives in `SkillDef` DSL plus a pure query API, not a separate UI rule table.
- MEM066: hard boundary — no CLI/windowed skill-ID-specific legality rules.
- MEM071/MEM069: S02 preserves `TargetShape` on `ResolvedAction` and rejects non-single shapes with stable `UnimplementedTargetShape:<Shape>` before mutation.
- `docs/skill_legality_contract.md`: stable statuses/reasons include `WrongSide`, `TargetKo`, `TargetNotKo`, `TargetFullHp`, `TargetNotDamaged`, `UnimplementedTargetShape`, `UnimplementedEffect`.

## Recommendation

Add explicit metadata to `SkillDef` in `src/data/skills_ron.rs` and migrate canonical `assets/data/skills.ron` in the same slice.

A practical S03 schema shape:

```rust
pub struct SkillDef {
    pub id: SkillId,
    pub name: String,
    pub damage_tag: DamageTag,
    pub sp_cost: i32,
    pub targeting: SkillTargeting,
    pub implementation: SkillImplementation,
    pub effects: Vec<Effect>,
}

pub struct SkillTargeting {
    pub shape: TargetShape,
    pub side: TargetSide,
    pub life: TargetLife,
    pub self_rule: SelfTargetRule,
}

pub enum TargetSide { Enemy, Ally, Any }
pub enum TargetLife { Alive, Ko, Damaged, Any }
pub enum SelfTargetRule { Forbid, Allow, Require }
pub enum SkillImplementation {
    Implemented,
    Deferred { reason: LegalityReasonCode },
    Hidden { reason: LegalityReasonCode },
}
```

The exact names can vary, but the schema should answer these questions without looking at the skill id:

1. What selected target shape does this skill require?
2. Which side may be selected relative to the actor?
3. What target HP/KO/damaged state is required?
4. Is self-targeting allowed, required, or forbidden?
5. Is this skill truthfully implemented now, or should consumers treat it as deferred/hidden with a stable reason?

Use `SkillImplementation::Deferred { reason: UnimplementedTargetShape }` for canonical `Row` skills while S02 still rejects them at execution time. Use `Implemented` for current `Single` offensive and revive skills. Use a deferred implementation status for mixed target semantics (`angemon_ult`) unless the planner chooses to split/re-author the skill data.

Do **not** create a sidecar legality registry. It would violate D053 and make future RON edits easy to desynchronize.

## Implementation landscape

### `src/data/skills_ron.rs`

Current responsibilities:

- Defines `TargetShape`, `Effect`, `SkillDef`, `SkillBook`.
- Contains serde roundtrip tests and canonical `skills.ron` parse test.

S03 changes likely belong here first:

- Add targeting/legalities enums and structs.
- Consider `#[serde(deny_unknown_fields)]` on `SkillDef`, `SkillTargeting`, and metadata structs so bad RON keys fail loudly. This is especially useful because S03 acceptance calls for invalid sample skills to fail loudly.
- Add semantic validation helpers, e.g. `validate_skill_book(&SkillBook) -> Result<(), SkillBookValidationError>`.
- Extend tests for:
  - roundtrip of `SkillTargeting` and `SkillImplementation`;
  - canonical `skills.ron` parses and validates;
  - missing `targeting` fails parse;
  - unknown targeting field fails parse if `deny_unknown_fields` is used;
  - contradictory metadata fails validation, e.g. `Damage(target: Row)` with `targeting.shape: Single`, `Revive` with `life: Alive`, or `Implemented` + non-single shape while non-single execution is still unsupported.

Important constraint: adding required fields to `SkillDef` will break Rust struct literals. `rg "SkillDef \{" src tests` finds 58 construction sites. Most are in test fixtures. The planner should allocate a mechanical migration task for those fixtures or introduce small test helper constructors before changing the struct.

### `assets/data/skills.ron`

Canonical catalog must be migrated. Existing patterns:

- Most offensive skills: `targeting: (shape: Single, side: Enemy, life: Alive, self_rule: Forbid)`, `implementation: Implemented`.
- Current Row/AoE skills: `targeting.shape: Row`, `side: Enemy`, `life: Alive`, `implementation: Deferred(reason: UnimplementedTargetShape)` until multi-target execution exists. Examples: `heat_viper`, `greymon_ult`, `mega_blaster_aoe`, `kabuterimon_ult`, `kyubimon_ult`, `angemon_ult`.
- Revive skills: `targeting: (shape: Single, side: Ally, life: Ko, self_rule: Allow or Forbid)`, `implementation: Implemented`. Examples: `patamon_revive`, `first_aid`.
- Form Identity modifier skills (`GrantEnergy`, `SelfAdvance`) are not normal UI-selectable actions today but still go through `ActionIntent::Skill` as follow-up/form-identity actions. To preserve current execution, give them a selected-target policy that matches current routing (usually alive enemy selected by `select_follow_up_target`) while noting effect application is source-side. S05 can refine resource/source affordances.
- `angemon_ult` is the biggest data smell: it combines `Damage(Row)`, `ToughnessHit`, and `Revive(20)`. The gap matrix already flags this as mixed-effect target semantics. S03 should not allow a consumer to present it as a normal implemented action unless the content is split or explicitly deferred.

### `src/combat/resolution.rs`

Current functions affected:

- `resolve_action()` fills `ResolvedAction.target_shape` by calling `skill_target_shape(&skill.effects)`.
- `skill_target_shape()` defaults no-damage skills to `Single`.
- `target_shape_rejection_reason()` only checks shape.

After S03, `ResolvedAction.target_shape` should come from `skill.targeting.shape`, not by scanning `Effect::Damage`. That makes no-damage revive/modifier skills explicit and gives S04 a single metadata source.

S03 probably should not move all action validation out of `apply_effects`; that is S04/S06. But S03 can safely update shape propagation and keep S2's rejection path working.

### `src/combat/state.rs`

`ResolvedAction` already has `target_shape: TargetShape` from S02. If S03 metadata includes more than shape, there is a design choice:

- Minimal S03: keep only `target_shape` in `ResolvedAction`; S04 will query `SkillDef` directly for side/life/self.
- More integrated S03: add `targeting: SkillTargeting` to `ResolvedAction` and remove/derive `target_shape` later.

I recommend minimal S03 to avoid widening the pipeline before pure query types exist. But ensure `target_shape` is sourced from `SkillDef.targeting.shape`.

### `src/data/mod.rs` / asset loading

`DataPlugin` loads `SkillBook` through `bevy_common_assets::ron::RonAssetPlugin`. Parse failures will surface through the asset system, but semantic validation will not run unless wired. For S03, unit tests can call `validate_skill_book()` directly. A later slice can decide whether runtime asset hydration should reject invalid skill books explicitly.

### Existing docs/tests

- `docs/skill_legality_contract.md` already defines the reason/status vocabulary. S03 should reuse those exact reason names where possible.
- `tests/skill_legality_contract_docs.rs` guards the doc vocabulary.
- `tests/target_shape_truthfulness.rs` asserts S02 behavior and should remain green after shape metadata replaces effect-derived shape.
- `tests/revive_semantics.rs` and `tests/patamon_revive.rs` prove current revive execution but do not enforce side legality. S03 can add data validation; S06 will enforce engine/query parity.

## Natural seams for planning

1. **Schema and validation seam**
   - Files: `src/data/skills_ron.rs`.
   - Add enums/structs, serde derives, deny-unknown-fields, validation function, focused tests.
   - Verify with `cargo test-dev --lib data::skills_ron` or the closest available package/test selector.

2. **Canonical RON migration seam**
   - Files: `assets/data/skills.ron`.
   - Add `targeting` and `implementation` to all 72 skills.
   - Keep comments explaining deferred Row/mixed cases, especially `angemon_ult`.
   - Verify canonical parse/validation test.

3. **Rust fixture migration seam**
   - Files: many tests and small helper constructors in `src/combat/*_tests.rs` and `tests/*.rs`.
   - Either update all `SkillDef { ... }` literals or introduce helper constructors local to test modules.
   - Do this as a mechanical task after the schema compiles.

4. **Resolution shape source seam**
   - Files: `src/combat/resolution.rs`, maybe `src/combat/state.rs` if carrying richer metadata.
   - Change `resolve_action` to use `skill.targeting.shape`.
   - Keep S02 `target_shape_rejection_reason()` behavior intact.
   - Verify `tests/target_shape_truthfulness.rs`.

5. **Contract regression seam**
   - Files: `tests/skill_legality_contract_docs.rs`, maybe a new `tests/skill_dsl_targeting_metadata.rs` if integration tests are preferred.
   - Add tests proving invalid sample skills fail loudly and canonical skills contain metadata.

## Risks and gotchas

- **Struct literal churn is real.** There are 58 `SkillDef {` construction sites across `src` and `tests`. Plan a dedicated mechanical migration task, or executors will lose time to compile errors.
- **Do not infer side policy only from effects.** Revive currently has no target shape field because it has no `Damage`; relying on effect inference is exactly what S03 is meant to fix.
- **Current engine does not reject wrong-side targets.** S03 metadata can say revive targets allies and offensive skills target enemies, but S06 must enforce it. Do not claim R084 is fully validated in S03.
- **Form Identity skills are selected-target actions even when effects apply to source.** Current routing chooses an alive enemy target for follow-up/form-identity actions. If S03 marks these as `SelfOnly` immediately, existing pipeline/tests may fail before S05/S06 can reconcile command/resource semantics.
- **`angemon_ult` should not be normalized by skill-id exception.** Either re-author/split it or mark its implementation deferred via metadata. This follows the gap matrix and MEM066.
- **Reason codes should be stable identifiers, not display strings.** Existing engine strings are human-ish (`"Target is KO"`, `"SP shortfall"`). S03 metadata should use contract names (`TargetKo`, `SpShortfall`, etc.) so S04/S06 can map/compare them.
- **RON invalidity has two layers.** Missing/unknown/unknown-enum fields can fail deserialization; contradictory-but-well-typed declarations need semantic validation tests.

## Verification recommendations

Targeted verification after S03 implementation:

```bash
cargo test-dev --test target_shape_truthfulness --test skill_legality_contract_docs
cargo test-dev --lib data::skills_ron
```

If the exact lib test selector is not accepted by the project alias, use:

```bash
cargo test-dev skills_ron
cargo test-dev parse_canonical_skills_ron
```

Before completing the slice, run at least:

```bash
cargo test-dev --test target_shape_truthfulness --test revive_semantics --test patamon_revive
cargo test-dev skills_ron
```

The full milestone criterion remains `cargo test-dev`; S03 does not need the windowed check unless it changes exported types used by windowed code.

## Skill discovery

Installed relevant skills: none specific to Rust/Bevy/RON in the prompt.

`npx skills find "Rust Bevy RON serde"` returned potentially relevant optional skills (do not install automatically):

- `mindrally/skills@rust` — 243 installs; general Rust guidance.
- `sickn33/antigravity-awesome-skills@bevy-ecs-expert` — 108 installs; Bevy ECS guidance.
- `existential-birds/beagle@serde-code-review` — 18 installs; serde review, relevant if the schema becomes complex.

No external library documentation lookup was necessary; the slice uses existing project patterns around serde/RON and Bevy asset loading.
