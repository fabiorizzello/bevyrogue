use crate::combat::{
    api::{
        intent::Intent,
        registry::ExtRegistries,
        skill_ctx::SkillCtx,
        timeline::{BeatEvent, BeatPayload, SelectorCtx},
    },
    types::UnitId,
};

/// Register the kernel's built-in timeline extension functions.
///
/// These are the canonical ids that asset-backed compiled timelines can rely on
/// without any blueprint-specific registration.
pub fn register_kernel_builtins(regs: &mut ExtRegistries) {
    regs.hooks.register("core/deal_damage", deal_damage);
    regs.selectors.register("core/primary", select_primary_target);
    regs.predicates.register("core/always", always_true);
    regs.predicates.register("core/never", always_false);
}

fn select_primary_target(ctx: &SelectorCtx<'_>) -> Vec<UnitId> {
    vec![ctx.primary_target]
}

fn always_true(_: &BeatEvent, _: &SkillCtx<'_>) -> bool {
    true
}

fn always_false(_: &BeatEvent, _: &SkillCtx<'_>) -> bool {
    false
}

fn deal_damage(evt: &BeatEvent, ctx: &mut SkillCtx<'_>) {
    let Some(BeatPayload::DealDamage { amount, tag }) = ctx.beat_payload() else {
        panic!(
            "core/deal_damage requires BeatPayload::DealDamage at beat `{}`",
            evt.beat_id
        );
    };

    let target = evt
        .beat_targets
        .first()
        .copied()
        .unwrap_or(ctx.primary_target);

    ctx.enqueue(Intent::DealDamage {
        source: ctx.caster,
        target,
        amount: *amount,
        tag: *tag,
        cast_id: ctx.cast_id,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::api::{intent::CastId, timeline::BeatPayload};
    use bevy::prelude::World;
    use std::{collections::{HashSet, VecDeque}, num::NonZeroU32};

    #[test]
    fn register_kernel_builtins_installs_core_ids() {
        let mut regs = ExtRegistries::default();
        register_kernel_builtins(&mut regs);

        assert!(regs.hooks.get("core/deal_damage").is_some());
        assert!(regs.selectors.get("core/primary").is_some());
        assert!(regs.predicates.get("core/always").is_some());
        assert!(regs.predicates.get("core/never").is_some());
    }

    #[test]
    fn deal_damage_builtin_uses_payload_and_targets() {
        let mut regs = ExtRegistries::default();
        register_kernel_builtins(&mut regs);
        let hook = regs.hooks.get("core/deal_damage").expect("builtin hook registered");

        let world = World::new();
        let mut cast_hit_set = HashSet::new();
        let mut pending = VecDeque::new();
        let payload = BeatPayload::DealDamage { amount: 17, tag: crate::combat::types::DamageTag::Physical };
        let mut ctx = SkillCtx::new(
            UnitId(1),
            UnitId(2),
            CastId(NonZeroU32::new(7).unwrap()),
            crate::combat::api::skill_ctx::SkillCtxMode::Execute,
            &regs,
            &world,
            &mut cast_hit_set,
            &mut pending,
            Some(&payload),
        );
        let evt = BeatEvent {
            cast_id: ctx.cast_id,
            beat_id: "core/deal_damage",
            hop_index: 0,
            beat_targets: vec![UnitId(9)],
        };

        hook(&evt, &mut ctx);

        match pending.pop_front().expect("hook should enqueue one intent") {
            Intent::DealDamage { source, target, amount, .. } => {
                assert_eq!(source, UnitId(1));
                assert_eq!(target, UnitId(9));
                assert_eq!(amount, 17);
            }
            other => panic!("unexpected intent: {other:?}"),
        }
    }
}
