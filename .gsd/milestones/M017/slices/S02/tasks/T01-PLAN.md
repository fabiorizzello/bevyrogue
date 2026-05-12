---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T01: StatusBag + BuffKind types and policy

Add `BuffKind { Buff, Debuff }` enum, `classify_buff_kind(StatusEffectKind) -> BuffKind` (total: Blessed=Buff, all others=Debuff including reserved Burn/Shock), `StatusInstance { kind, duration_remaining }`, and `StatusBag(Vec<StatusInstance>)` as a Bevy Component (derive `Default`, `Component`). Methods: `apply(kind, dur)` upserts with `max(old, new)` duration; `tick_all()` decrements every instance and returns the kinds that expired (so the tick system can emit `OnStatusExpired`); `cleanse_debuffs() -> Vec<StatusEffectKind>` drains every Debuff-classified instance and returns the kinds removed; `has(kind) -> bool`; `get_dur(kind) -> Option<u32>`. Keep the inline `#[cfg(test)] mod tests` style established in the file: add unit tests for refresh-max-dur math (apply 2 then 1 -> 2; apply 2 then 5 -> 5), multi-kind coexistence at the unit level, classify_buff_kind totality, and cleanse_debuffs leaving Blessed intact. Lock the doc-comment policy decision: a re-apply that fails the accuracy roll does NOT refresh — the apply pipeline gates entry to `StatusBag::apply` behind the existing `roll_pct(threshold)` check at `src/combat/turn_system/pipeline.rs:725-729`, so `StatusBag::apply` itself does not see resisted re-applies. Remove the old single-component `StatusEffect` shape (do not keep as alias — S01's policy was delete-and-rewrite-fresh). Note: this task only adds the types — call-sites are migrated in T02/T03/T04; expect compile errors at those sites until those tasks land.

## Inputs

- `.gsd/milestones/M017/M017-CONTEXT.md`
- `.gsd/milestones/M017/M017-ROADMAP.md`
- `.gsd/milestones/M017/slices/S02/S02-RESEARCH.md`
- `src/combat/status_effect.rs`

## Expected Output

- `src/combat/status_effect.rs`

## Verification

Inline `#[cfg(test)] mod tests` in `src/combat/status_effect.rs` covers: refresh-max-dur math, multi-kind coexistence, classify_buff_kind totality, cleanse_debuffs leaving Blessed intact. Run `cargo test --lib combat::status_effect` (the rest of the tree will not compile until T02-T04, which is expected).

## Observability Impact

Defines the surface that tick and apply paths will use to emit `OnStatusApplied`/`OnStatusExpired` per instance.
