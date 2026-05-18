# S06 — Research: Migrate 18 active skill canon + drop `enum Effect`

## Summary

S05 proved the bridge (`skills.ron` → `SkillTimeline` → `CompiledTimeline` → `BeatRunner`) but the repo is still in a mixed state: only **2/74** catalog skills are timeline-backed (`renamon_ult`, `petit_thunder`), while **all 74** still carry legacy `effects:` data and production runtime still branches on `ResolvedAction.timeline_backed`.

This slice is not just a data rewrite. Dropping `enum Effect` currently cuts through:

- `src/data/skills_ron.rs` serialization/validation
- `src/combat/resolution.rs` (`resolve_action`, bounce selection, `apply_effects`)
- `src/combat/state.rs` (`ResolvedAction` caches effect-derived fields)
- `src/combat/turn_system/{mod.rs,pipeline.rs}` (timeline-vs-legacy dispatch)
- a large integration-test surface that still constructs `SkillDef { effects: ... }`

Highest-risk gap: the current runtime has **no asset-backed canon skill exercising `BeatKind::Loop`**. The only loop proof is test-only (`tests/timeline_chain_bolt_port.rs`). If S06 must exit with "Loop tier-N" confidence, the first proof should be a real timeline-backed looped skill, not another straight-line single-target port.

## Requirement support

Supports **R001** directly: more canon skills must compile/load through the kernel timeline path, and dangling ids must continue to fail before runtime.

## Skills Discovered

Relevant installed skills already present:

- `bevy`
- `rust-best-practices`

No additional skill installation is needed for the core tech in this slice. The main work is local Bevy/Rust/runtime architecture, not an external service/library integration.

## Implementation Landscape

### Catalog / schema

- `assets/data/skills.ron`
  - Current live catalog has **74** skills.
  - Only `renamon_ult` and `petit_thunder` have `timeline: Some(...)`.
  - All skills still have legacy `effects: [...]`.
  - Current child-canon active skills are still mostly simple single-target effect lists (`baby_flame`, `bubble_blast`, `draconic_edge`, `diamond_storm`, `holy_breeze`, `agumon_ult`, `gabumon_ult`, `dorumon_ult`, `patamon_ult`, `patamon_revive`, `tentomon_basic`, `tentomon_ult`, etc.).
  - There are also post-MVP adult/enemy/fixture skills in the shipped asset set, so a global `Effect` removal has broader blast radius than the roadmap wording suggests.

- `src/data/skills_ron.rs`
  - Owns `enum Effect` and `SkillDef.effects: Vec<Effect>`.
  - `validate_skill_book()` assumes effect-backed semantics and tests currently assert canonical `skills.ron` parses with `!skill.effects.is_empty()`.
  - `parse_canonical_skills_ron()` hard-checks the 74-skill catalog and current ids.

- `src/data/skill_timeline.rs`
  - Already supports **incremental** migration via optional `SkillDef.timeline`.
  - Compiler/validator is ready; it compiles only timeline-backed skills and preserves skill-id + site context on failure.

### Runtime / dispatch

- `src/combat/turn_system/mod.rs`
  - `step_declaration()` still resolves actions through legacy `resolve_action()`.
  - Runtime dispatch is binary: `timeline_backed == true` goes to `run_timeline_backed_action()`, otherwise legacy path.

- `src/combat/turn_system/pipeline.rs`
  - `run_timeline_backed_action()` interns owned ids, runs `BeatRunner`, drains `IntentQueue`, then emits `OnActionApplied/Resolved`.
  - Legacy app path still handles SP, ult, bounce fan-out, AoE, revive/heal/cleanse, etc.
  - `tests/compiled_timeline_runtime_dispatch.rs` explicitly proves that unmigrated skills still fall back to the legacy effect resolver; that test will need to change once the branch disappears.

- `src/combat/state.rs`
  - `ResolvedAction` is legacy-shaped: `base_damage`, `toughness_damage`, `revive_pct`, `heal_pct`, `status_to_apply`, `delay_pct`, `grant_free_skill_count`, `cleanse_count`, `damage_curve`, `timeline_backed`, `custom_signals`, etc.
  - This struct is currently populated by scanning `effects` in `resolve_action()`.

- `src/combat/resolution.rs`
  - Owns both **valuable generic logic** and **legacy effect decoding**.
  - Keep: `resolve_targets()`, `select_bounce_hop()`, `compute_hop_damage()`, target-shape helpers.
  - Drop/replace candidate: `resolve_action()`’s effect scans and `apply_effects()` legacy execution path.
  - Important: legacy bounce coverage already exists here; S06 should reuse the existing bounce-targeting semantics instead of re-inventing them.

### Timeline built-ins / current capability ceiling

- `src/combat/api/timeline.rs`
  - `BeatPayload` currently supports only:
    - `DealDamage`
    - `BreakToughness`
    - `ApplyStatus`
    - `DelayTurn`
    - `ApplyBuff`
    - `BlueprintSignal`

- `src/combat/api/builtins.rs`
  - Registered built-ins are minimal:
    - hooks: `core/deal_damage`, `core/apply_effect`
    - selectors: `core/primary`, `core/caster`
    - predicates: `core/always`, `core/never`
  - This is enough for S05’s straight-line proofs, but not enough to absorb the full active catalog if S06 truly drops `Effect`.

- `src/combat/api/skill_ctx.rs`
  - Hook context exposes `world`, `cast_hit_set`, `beat_payload`, `cast_id`, etc.
  - Prior memories apply here: when immutable world queries are needed inside predicate/hook helpers, use `World::try_query::<&T>()`, not `World::query()`.

## Key Gaps / Constraints

### 1. `Effect` removal is broader than “migrate 18 canon skills”

The shipped asset catalog and many tests still construct legacy `Effect` data directly. If S06 removes `Effect` from `SkillDef`, the planner must budget for:

- updating `skills.ron` schema
- rewriting integration fixtures that build `SkillDef` inline
- replacing `parse_canonical_skills_ron()` assumptions in `src/data/skills_ron.rs`
- deleting the legacy-fallback expectation from `tests/compiled_timeline_runtime_dispatch.rs`

This is a repo-wide refactor, not a narrow asset-only slice.

### 2. Current timeline payload/builtins do not cover all active verbs

Observed live verbs in `skills.ron` that are not fully represented in `BeatPayload`/built-ins today:

- `Revive` (`patamon_revive`, `first_aid`, `angemon_ult` hybrid)
- `AdvanceTurn` (`kyubimon_ult`)
- `GrantEnergy` / `SelfAdvance` (form-identity skills; maybe out of S06 if passive-owned)
- `GrantFreeSkill` (`brave_tri_strike`, compatibility surface)
- row / all-enemy fan-out semantics
- bounce/loop hop semantics as first-class asset-backed behavior

If S06 scope is truly only the 18 active canon skills, some of these may be deferred — but the global `Effect` drop cannot happen until every shipped runtime path depending on them is addressed.

### 3. Loop proof is still fixture-only

- `tests/timeline_chain_bolt_port.rs` shows a valid `BeatKind::Loop` pattern using `cast_hit_set` and `hop_index`.
- `tests/target_shape_bounce_chain.rs` proves current bounce gameplay semantics, but only through the **legacy** effect path.
- No asset-backed catalog skill currently uses `BeatKind::Loop`.

Therefore the planner should treat **real loop migration** as the first technical unblocker for this slice.

### 4. `petit_thunder` is a semantic fork

Current live asset:
- `assets/data/skills.ron` models `petit_thunder` as single-target damage + break + paralyze + signal.

Design doc:
- `docs/future_design_draft/digimon/tentomon/02_skill_petit_thunder.md` defines canon intent as **Bounce(3) + OnHit3 Paralyzed + self DR**.

So S06 has an explicit choice to make:

- preserve the current shipped simplified semantics and prove loop using another skill/fixture, or
- use S06 to align `petit_thunder` with the intended bounce canon and let it become the real loop proof.

That choice affects gameplay, tests, and slice sizing.

### 5. Headless-first still constrains all runtime work

Per `CLAUDE.md`, any new runner/builtin/selector logic must stay headless-safe. No windowed-only assumptions should leak into timeline execution or validation.

## Recommendation

### Recommended execution order

1. **Prove one real asset-backed loop skill first.**
   - This is the only part not already proven by S05.
   - Reuse existing bounce targeting semantics from `resolution.rs`.
   - After this, most remaining single-target actives become repetitive data migration.

2. **Expand timeline payload/builtin coverage only for active-skill verbs actually needed in S06.**
   - Straight-line single-target damage/break/status/delay/signal are already done.
   - Add only the minimum additional primitives required by the active set chosen for this slice.

3. **Batch-migrate the rest of the active catalog in `skills.ron`.**
   - Group by shape/behavior, not by Digimon name.
   - Example batches: straight-line single-target, row/all-target, revive/support, loop/bounce.

4. **Only then collapse the legacy resolver.**
   - Remove `ResolvedAction.timeline_backed` branching.
   - Remove effect scanning from `resolve_action()`.
   - Remove `apply_effects()` once no production caller remains.
   - Drop `enum Effect` last, not first.

### Why this order

Because the first three steps are additive and keep the repo runnable. Deleting `Effect` early forces a wide partial rewrite with weak intermediate verification.

## Natural seams for planning

### Seam A — Timeline runtime surface

Files:
- `src/combat/api/timeline.rs`
- `src/combat/api/builtins.rs`
- `src/combat/api/skill_ctx.rs`
- possibly `src/combat/api/applier.rs`

Purpose:
- Add any missing payload variants / built-ins / selectors / predicates needed by the active catalog.
- Keep the API generic; no Digimon names in kernel code.

### Seam B — Asset migration batches

Files:
- `assets/data/skills.ron`
- `src/data/skill_timeline.rs` (only if schema changes)
- `src/data/skills_ron.rs` validation/tests

Purpose:
- Rewrite active skills from `effects:` semantics into `timeline:` semantics.
- Update validation/tests to assert the new catalog shape.

### Seam C — Resolver collapse

Files:
- `src/combat/resolution.rs`
- `src/combat/state.rs`
- `src/combat/turn_system/{mod.rs,pipeline.rs}`

Purpose:
- Stop deriving runtime behavior from `Effect` scans.
- Shrink/reshape `ResolvedAction` to only the metadata still needed outside the runner.
- Delete legacy effect execution branch.

### Seam D — Test migration

Files:
- `tests/compiled_timeline_runtime_dispatch.rs`
- `tests/target_shape_bounce_chain.rs`
- `tests/compiled_timeline_{petit_thunder,tohakken}.rs`
- any inline `SkillDef { effects: ... }` fixtures touched by schema removal

Purpose:
- Replace “legacy fallback still works” assertions with “all shipped active skills run through timeline path”.
- Preserve bounce parity evidence while changing execution backend.

## First proof

**Do this first:** migrate one asset-backed skill to a genuine looped timeline and prove parity against existing bounce semantics.

Best candidate if scope allows semantic alignment:
- `petit_thunder` → real `BeatKind::Loop` / bounce proof.

Fallback if gameplay change is not allowed in S06:
- add a temporary asset-backed loop skill or port an existing compatibility skill, but that is weaker evidence because it does not retire a shipped canon action.

Why this is the right first proof:
- straight-line single-target migration is already de-risked by S05
- loop/bounce is the only unproven runtime shape for S06’s acceptance text
- once loop works in shipped data, the rest is mostly batch conversion + cleanup

## Verification

Suggested verification ladder for the executor/planner:

1. **Loop/runtime proofs**
   - `cargo test --test timeline_chain_bolt_port`
   - `cargo test --test target_shape_bounce_chain`
   - plus the real migrated skill test (likely update/create around `petit_thunder`)

2. **Existing timeline regression**
   - `cargo test --test compiled_timeline_petit_thunder --test compiled_timeline_tohakken`
   - `cargo test --test compiled_timeline_runtime_dispatch`

3. **Catalog/boot validation**
   - `cargo test --test compiled_timeline_boot_validation --test compiled_timeline_builtin_validation`

4. **Full project checks**
   - `cargo test`
   - `cargo check`
   - `cargo check --features windowed`

5. **Structural greps for slice exit**
   - `rg "enum Effect" src/data/skills_ron.rs`
   - `rg "apply_effects\(" src/combat`
   - `rg "timeline_backed" src/combat`
   - if the schema is fully cut over, also `rg "effects:" assets/data/skills.ron`

## Watch-outs for the planner

- **Scope ambiguity:** roadmap says “18 active canon”, repo ships many more effect-backed skills. Decide explicitly whether S06 migrates only the 18, or all shipped actives needed to make global `Effect` deletion true.
- **Do not lose generic helpers:** bounce target resolution in `resolution.rs` is reusable even if `Effect` goes away.
- **Preserve load-time failure quality:** S05 established good error surfaces (`skill_id` + `site`). Do not regress to runtime-only failures.
- **Bevy borrow trap:** immutable world reads from hook/predicate helpers should use `World::try_query::<&T>()`.
- **Headless-first:** no UI/windowing assumptions in any new timeline helper.
