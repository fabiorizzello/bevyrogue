---
estimated_steps: 1
estimated_files: 4
skills_used: []
---

# T02: SkillCtx<'a> + intent_applier dispatcher (DealDamage canary wired)

Aggiungere SkillCtx + dispatcher Intent. In S01 il dispatcher è scheletro: route per variant esiste, ma solo DealDamage è wired al damage system esistente come canary. Altre variant: log::warn! + delega alla code-path attuale. SkillCtxMode {DryRun, Execute, Preview} (Default=Execute). SkillCtx<'a> con caster, primary_target, cast_id, pending VecDeque<Intent>. Resource IntentQueue + system intent_applier exclusive. Test canary tests/intent_applier_canary.rs: spawn 2 unit, enqueue DealDamage, tick, asserisce HP ridotto + CombatEvent::OnDamageDealt + cast_id propagato (finalizzato dopo T03).

## Inputs

- `src/combat/api/intent.rs`
- `src/combat/damage.rs`
- `src/combat/resolution.rs`
- `src/combat/events.rs`

## Expected Output

- `src/combat/api/skill_ctx.rs`
- `src/combat/api/applier.rs`
- `src/combat/api/mod.rs`
- `tests/intent_applier_canary.rs`

## Verification

cargo check (headless + windowed) puliti. cargo test --test intent_applier_canary verde (finalizzato dopo T03). rg 'fn intent_applier' src/combat/api/applier.rs → 1. rg 'use bevy::winit|use bevy::render|use bevy_egui' src/combat/api/skill_ctx.rs src/combat/api/applier.rs → 0.

## Observability Impact

log::warn! per Intent variant non-wired (rumoroso v0). CI imposta log level a error. Canary emette CombatEvent::OnDamageDealt esistente.
