---
estimated_steps: 18
estimated_files: 2
skills_used: []
---

# T02: Refine ExtPoint::Fn signatures (Hook/Selector/Predicate/Cue) + extend SkillCtx<'a>

Why: T01's `BeatEvent` and timeline types need real callable signatures on the four graph-referenced axes; T03's `BeatRunner` needs `SkillCtx<'a>` to carry the borrows the spike used thread-locals for (F7). This task does the surgical signature promotion and adds the borrows — nothing more.

Do:
1. Edit `src/combat/api/registry.rs`:
   - `HookExt::Fn = for<'a> fn(&BeatEvent, &mut SkillCtx<'a>)`.
   - `SelectorExt::Fn = for<'a> fn(&SelectorCtx<'a>) -> Vec<UnitId>`.
   - `PredicateExt::Fn = for<'a> fn(&BeatEvent, &SkillCtx<'a>) -> bool`.
   - `CueExt::Fn = for<'a> fn(&CueCtx<'a>) -> &'static str` (or `CueId` alias — match spike; use `&'static str` if no CueId type yet).
   - Leave `FormulaExt`, `TickExt`, `AiUtilityExt` as `fn()` placeholders (out of scope for S02 — explicit comment noting S05/S07 will refine).
   - Import `BeatEvent`, `SelectorCtx`, `CueCtx` from `super::timeline`.
2. Edit `src/combat/api/skill_ctx.rs`:
   - Add `pub registries: &'a ExtRegistries` field.
   - Add a state borrow handle — use `pub world: &'a bevy::prelude::World` as the simplest S02 choice (the F7 promotion). Gate the import behind nothing — `bevy::prelude::World` is core, not winit/render.
   - Add `pub cast_hit_set: &'a mut std::collections::HashSet<UnitId>` (used by the chain_bolt NoRepeat selector / runner).
   - Update `SkillCtx::new` and existing call sites (only `intent_applier` and its canary test) to pass the new borrows; for the canary, supply a temp `HashSet` and the world reference at construction. Keep the existing `pending: &'a mut VecDeque<Intent>` channel intact.
3. Update the inline tests in `registry.rs` if they break (only `NumExt` test pattern is independent — should still compile).
4. Update inline test fixtures in `skill_ctx.rs` if any (the file currently has none).
5. Update `tests/intent_applier_canary.rs` only if its `SkillCtx` construction changes — the applier itself doesn't build SkillCtx so likely a no-op.

Done-when: `cargo check` headless and `cargo check --features windowed` both exit 0; `cargo test` full suite has 0 failures; the four `ExtPoint::Fn` types compile against `BeatEvent`/`SelectorCtx`/`CueCtx` from T01.

## Inputs

- `src/combat/api/registry.rs`
- `src/combat/api/skill_ctx.rs`
- `src/combat/api/timeline.rs`
- `src/combat/api/applier.rs`
- `.gsd/workflows/spikes/M021-timeline-fsm/src/lib.rs`

## Expected Output

- `src/combat/api/registry.rs`
- `src/combat/api/skill_ctx.rs`

## Verification

cargo check && cargo check --features windowed && cargo test --lib combat::api:: && cargo test --test intent_applier_canary && cargo test --test cast_id_propagation
