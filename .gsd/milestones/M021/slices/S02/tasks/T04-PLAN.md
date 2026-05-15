---
estimated_steps: 22
estimated_files: 2
skills_used: []
---

# T04: Demo gates 1 & 3: fixture OnTurnStart kills target + chain_bolt CompiledTimeline port

Why: the two runner-driven demo gates from the roadmap. Keeping them in one task pays off because both build a hand-rolled `CompiledTimeline`, both wire a hook fn, and both drive `BeatRunner::run_to_completion` over an existing Bevy `App` — duplication of setup helpers is highest here.

Do:
1. Create `tests/timeline_onturnstart_kills.rs`. Setup:
   - Standard headless `App` with `CombatPlugin`.
   - Spawn 1 ally + 1 enemy via the existing `tests/common/` helpers (re-use whatever `damage_breakdown_log.rs` or `intent_applier_canary.rs` use — minimal entities with `Unit { hp: 1, max_hp: 1, .. }` for the enemy).
   - Pull a `CastId` from `world.resource_mut::<CastIdGen>().next()`.
   - Register a fn `pub fn ko_target(ev: &BeatEvent, ctx: &mut SkillCtx)` into `ExtRegistries.hooks` under id `"test/ko_target"` that calls `ctx.enqueue(Intent::DealDamage { caster: ctx.caster, target: ctx.primary_target, magnitude: 9999, cast_id: ctx.cast_id, .. })`.
   - Build `CompiledTimeline { entry: "impact", beats: [Beat { id: "impact", kind: Impact, hook: Some("test/ko_target"), .. }], edges: [] }`.
   - Drive `BeatRunner::run_to_completion`, then `app.update()` to let `intent_applier` drain the queue.
   - Assert: enemy `Unit::hp == 0` and an `OnDamageDealt` event with the matching `cast_id` is present.
2. Create `tests/timeline_chain_bolt_port.rs`. Setup:
   - Headless `App` with `CombatPlugin`.
   - Spawn 1 caster + 3 enemies with descending HP (e.g. 100, 80, 60) so lowest-HP-alive selection is unambiguous.
   - Register `selectors["chain/lowest_hp_pct_alive_norepeat"]` (returns the alive enemy with lowest hp_pct that is not in `ctx.cast_hit_set`; tie-break by UnitId asc).
   - Register `predicates["chain/pool_exhausted_or_max_hops"]` returning `hop_index >= 3 || cast_hit_set covers all alive enemies`.
   - Register `formulas["chain/falloff80"]` (placeholder reuse: until FormulaExt is refined, hard-code the falloff math inside the hook; document with a comment that S05's compiler will hoist it).
   - Register `hooks["chain/bolt_hop"]` that picks the selector target, computes `base * 0.8^hop_index`, and enqueues `Intent::DealDamage`.
   - Build `CompiledTimeline { entry: "loop", beats: [Beat { id: "loop", kind: Loop { body: vec![Beat { id: "hop", kind: Impact, hook: Some("chain/bolt_hop"), selector: Some("chain/lowest_hp_pct_alive_norepeat"), .. }], exit_when: "chain/pool_exhausted_or_max_hops" }, .. }], edges: [] }`.
   - Drive `BeatRunner::run_to_completion`, then `app.update()`.
   - Assertions: 3 `DealDamage` intents observed in the order [lowest_hp first, then next-lowest non-repeating, then next], with damage ladder satisfying the 80% per hop falloff (use integer-tolerant assertions), and no target hit twice.
3. Use a shared test helper module (`tests/common/timeline_fixtures.rs` if needed, otherwise duplicate the small setup blocks inline — research advises against premature abstractions; small duplication is fine).

Done-when: both new tests green; `cargo test` full suite still 0 failures; `cargo test timeline_onturnstart_kills timeline_chain_bolt_port` green.

## Inputs

- `src/combat/api/timeline.rs`
- `src/combat/api/runner.rs`
- `src/combat/api/registry.rs`
- `src/combat/api/skill_ctx.rs`
- `src/combat/api/applier.rs`
- `src/combat/api/intent.rs`
- `src/data/skills_ron.rs`
- `tests/intent_applier_canary.rs`
- `tests/cast_id_propagation.rs`
- `tests/common`

## Expected Output

- `tests/timeline_onturnstart_kills.rs`
- `tests/timeline_chain_bolt_port.rs`

## Verification

cargo test --test timeline_onturnstart_kills && cargo test --test timeline_chain_bolt_port && cargo test
