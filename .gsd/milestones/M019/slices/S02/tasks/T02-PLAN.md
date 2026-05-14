---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T02: Implement apply_heal_only helper and wire Heal into apply_effects (Single/SelfOnly)

Add `skill_heal_pct(&[Effect])` extractor next to `skill_revive_pct` (around resolution.rs:334). Add `heal_pct: u32` field to `ResolvedAction` (src/combat/state.rs) and populate it in `resolve_action` (resolution.rs:233). Add `apply_heal_only` helper mirroring `apply_damage_only` (resolution.rs:420): KO check first → if KO, return without state change and without emitting an event; otherwise compute `healed = min((hp_max * pct + 99)/100, hp_max - hp_current)` using i64 widening, increment hp_current, emit `OnHealed { amount: healed, hp_after: hp_current }`. Use floor division `(hp_max * pct) / 100` (not ceil) for predictable arithmetic; document the rounding choice in a one-line comment only if non-obvious. Extend `resolve_targets` (resolution.rs:84) with the `AllAllies` arm: filter `team == primary.team && alive`, sort by `slot_index` ascending. Add `AllAllies` to `target_shape_is_executable_now` (resolution.rs:402). In `apply_effects` (resolution.rs:533), insert a heal branch BEFORE the existing damage-path KO guard: if `resolved.heal_pct > 0` and target is KO, return with sp_ok=true (no events, no SP consumed — matches the no-op policy from S02 ROADMAP). For alive Single/SelfOnly, call `apply_heal_only` and propagate the event. This task covers single-target heal only — multi-target fan-out lives in T03.

## Inputs

- `src/combat/resolution.rs`
- `src/combat/state.rs`
- `src/combat/events.rs`
- `src/data/skills_ron.rs`
- `.gsd/milestones/M019/slices/S02/S02-RESEARCH.md`

## Expected Output

- `src/combat/resolution.rs`
- `src/combat/state.rs`

## Verification

cargo check passes. Write a temporary smoke test or run cargo test (existing suite must remain green; no Heal skills in baseline fixtures so trace identity is preserved). Integration test for heal behaviour is added in T03.

## Observability Impact

OnHealed events are now emitted by apply_heal_only via the existing CombatEvent bus — JSONL logger picks them up automatically.
