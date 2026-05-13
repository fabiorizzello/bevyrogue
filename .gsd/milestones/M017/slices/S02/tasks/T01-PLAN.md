---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T01: StatusBag + BuffKind types and policy

Edit `src/combat/status_effect.rs`. Keep `StatusEffectKind` enum unchanged (already canon post-S01). Replace the `StatusEffect` struct with: `StatusInstance { kind: StatusEffectKind, duration_remaining: u32 }` (Serialize/Deserialize, Clone, PartialEq, Debug; not a Component). Add `BuffKind { Buff, Debuff }` enum (Copy, Eq) and free fn `pub fn classify_buff_kind(kind: &StatusEffectKind) -> BuffKind` returning `Buff` for `Blessed` and `Debuff` for all 6 other variants (Heated/Chilled/Paralyzed/Slowed/Burn/Shock — total over the enum). Add `pub struct StatusBag(Vec<StatusInstance>)` with `#[derive(Component, Default, Debug, Clone)]`. Methods: `apply(&mut self, kind: StatusEffectKind, dur: u32)` upserts with `max(old, new)`; `tick_all(&mut self) -> Vec<StatusEffectKind>` decrements every instance, returns kinds whose duration reached 0, removes them; `cleanse_debuffs(&mut self) -> Vec<StatusEffectKind>` drains every Debuff-classified instance, returns kinds removed; `has(&self, kind: &StatusEffectKind) -> bool`; `get_dur(&self, kind: &StatusEffectKind) -> Option<u32>`; `is_empty(&self) -> bool`; `iter(&self) -> impl Iterator<Item = &StatusInstance>`. Rewrite the existing inline `#[cfg(test)] mod tests` (lines ~80-138): drop the 7 single-component RON round-trip tests; replace with refresh-max-dur math (apply 2 then 1 → dur=2; apply 2 then 5 → dur=5), multi-kind coexistence at unit level, `classify_buff_kind` totality, `cleanse_debuffs` leaving Blessed intact, `tick_all` returns expired kinds and removes them. Lock policy as a doc-comment on `apply`: a re-apply that fails the accuracy roll does NOT call `apply` — the `roll_pct(threshold)` gate at `pipeline.rs:725-729` runs *before* `apply`, so resisted re-apply emits `OnStatusResisted` and leaves duration untouched. Update `src/combat/mod.rs:61` re-export: replace `StatusEffect` with `StatusBag` (add `StatusInstance`/`BuffKind`/`classify_buff_kind` only if external crates need them).

## Inputs

- `.gsd/milestones/M017/M017-CONTEXT.md`
- `.gsd/milestones/M017/M017-ROADMAP.md`
- `.gsd/milestones/M017/slices/S02/S02-RESEARCH.md`
- `src/combat/status_effect.rs`
- `src/combat/mod.rs`

## Expected Output

- `src/combat/status_effect.rs`
- `src/combat/mod.rs`

## Verification

Inline `#[cfg(test)] mod tests` in `src/combat/status_effect.rs` covers: refresh-max-dur math, multi-kind coexistence, `classify_buff_kind` totality (all 7 variants), `cleanse_debuffs` leaving Blessed intact, `tick_all` returning + removing expired kinds. Run `cargo test --lib combat::status_effect`. The rest of the tree will not compile until T02-T04 — expected.

## Observability Impact

Defines the surface that tick and apply paths will use to emit `OnStatusApplied`/`OnStatusExpired` per instance, and that M019's `Effect::EmitCleanse` will call.
