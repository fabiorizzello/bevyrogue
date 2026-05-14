---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T03: Heated DoT emission at turn-end (bypasses stun)

In `src/combat/turn_system/mod.rs` around lines 465-513, emit Heated DoT BEFORE `tick_all` and BEFORE the stun-skip early-return. Canon §H.1: Heated ticks 4 HP Fire damage on the affected unit at its own turn-end, and bypasses Paralyzed/Stunned skip (DoT does not require the unit to act). Concretely: restructure the per-turn block so the StatusBag is inspected first — for each `StatusEffectKind::Heated` instance, mutate `unit.hp_current -= 4` (clamped, skip if already KO), then push `CombatEventKind::OnDamageDealt { amount: 4, kind: DamageKind::Neutral, damage_tag: DamageTag::Fire, tag_mod_pct: 100, triangle_mod_pct: 100 }` via `emit_combat_event`. Emit `OnKO` if hp_current ≤ 0 post-tick. Run this DoT pass UNCONDITIONALLY (above stun continue) so a Heated+Stunned unit still burns. Then proceed with the existing stun continue and the pre-existing OnStatusTick/tick_all flow. Audit `src/combat/follow_up.rs` to confirm OnDamageDealt listeners do not require a preceding OnSkillCast (research risk note). Skills: bevy-ecs-expert, verify-before-complete.

## Inputs

- `src/combat/turn_system/mod.rs`
- `src/combat/status_effect.rs`
- `src/combat/events.rs`
- `src/combat/follow_up.rs`
- `src/combat/types.rs`

## Expected Output

- `src/combat/turn_system/mod.rs`

## Verification

cargo check && cargo test --test combat_coherence && cargo test --test follow_up_chains
