use bevy::prelude::World;

use crate::combat::{
    resolution::{TargetEntry, TargetableSnapshot, resolve_targets},
    runtime::{
        intent::Intent,
        registry::ExtRegistries,
        timeline::{BeatEvent, BeatPayload, SelectorCtx},
    },
    team::Team,
    types::UnitId,
    unit::{Ko, SlotIndex, Unit},
};
use crate::data::skills_ron::TargetShape;

/// Register the kernel's built-in timeline extension functions.
///
/// These are the canonical ids that asset-backed compiled timelines can rely on
/// without any blueprint-specific registration.
pub fn register_kernel_builtins(regs: &mut ExtRegistries) {
    regs.hooks.register("core/deal_damage", deal_damage);
    regs.hooks.register("core/break_toughness", break_toughness);
    regs.hooks.register("core/apply_status", apply_status);
    regs.hooks.register("core/delay_turn", delay_turn);
    regs.hooks.register("core/advance_turn", advance_turn);
    regs.hooks.register("core/revive", revive);
    regs.hooks
        .register("core/grant_free_skill", grant_free_skill);
    regs.hooks.register("core/grant_energy", grant_energy);
    regs.hooks.register("core/self_advance", self_advance);
    regs.hooks.register("core/apply_effect", apply_effect);
    regs.selectors
        .register("core/primary", select_primary_target);
    regs.selectors.register("core/caster", select_caster_target);
    regs.predicates.register("core/always", always_true);
    regs.predicates.register("core/never", always_false);
}

fn select_primary_target(ctx: &SelectorCtx<'_>) -> Vec<UnitId> {
    vec![ctx.primary_target]
}

fn select_caster_target(ctx: &SelectorCtx<'_>) -> Vec<UnitId> {
    vec![ctx.caster]
}

fn always_true(_: &BeatEvent, _: &crate::combat::runtime::SkillCtx<'_>) -> bool {
    true
}

fn always_false(_: &BeatEvent, _: &crate::combat::runtime::SkillCtx<'_>) -> bool {
    false
}

fn deal_damage(evt: &BeatEvent, ctx: &mut crate::combat::runtime::SkillCtx<'_>) {
    let Some(BeatPayload::DealDamage {
        amount,
        tag,
        target,
    }) = ctx.beat_payload()
    else {
        unreachable!(
            "core/deal_damage requires BeatPayload::DealDamage at beat `{}`",
            evt.beat_id
        );
    };

    let caster = ctx.caster;
    let cast_id = ctx.cast_id;
    enqueue_targets(evt, ctx, *target, move |target_id| Intent::DealDamage {
        source: caster,
        target: target_id,
        amount: *amount,
        tag: *tag,
        cast_id,
    });
}

fn break_toughness(evt: &BeatEvent, ctx: &mut crate::combat::runtime::SkillCtx<'_>) {
    let Some(BeatPayload::BreakToughness {
        amount,
        tag,
        target,
    }) = ctx.beat_payload()
    else {
        unreachable!(
            "core/break_toughness requires BeatPayload::BreakToughness at beat `{}`",
            evt.beat_id
        );
    };

    let caster = ctx.caster;
    let cast_id = ctx.cast_id;
    enqueue_targets(evt, ctx, *target, move |target_id| Intent::BreakToughness {
        source: caster,
        target: target_id,
        amount: *amount,
        tag: *tag,
        cast_id,
    });
}

fn apply_status(evt: &BeatEvent, ctx: &mut crate::combat::runtime::SkillCtx<'_>) {
    let Some(BeatPayload::ApplyStatus {
        kind,
        duration,
        target,
    }) = ctx.beat_payload()
    else {
        unreachable!(
            "core/apply_status requires BeatPayload::ApplyStatus at beat `{}`",
            evt.beat_id
        );
    };

    let caster = ctx.caster;
    let cast_id = ctx.cast_id;
    enqueue_targets(evt, ctx, *target, move |target_id| Intent::ApplyStatus {
        source: caster,
        target: target_id,
        kind: kind.clone(),
        duration_turns: *duration,
        cast_id,
    });
}

fn delay_turn(evt: &BeatEvent, ctx: &mut crate::combat::runtime::SkillCtx<'_>) {
    let Some(BeatPayload::DelayTurn { amount_pct, target }) = ctx.beat_payload() else {
        unreachable!(
            "core/delay_turn requires BeatPayload::DelayTurn at beat `{}`",
            evt.beat_id
        );
    };

    let cast_id = ctx.cast_id;
    enqueue_targets(evt, ctx, *target, move |target_id| Intent::DelayTurn {
        target: target_id,
        amount_pct: (*amount_pct).min(50),
        cast_id,
    });
}

fn advance_turn(evt: &BeatEvent, ctx: &mut crate::combat::runtime::SkillCtx<'_>) {
    let Some(BeatPayload::AdvanceTurn { amount_pct, target }) = ctx.beat_payload() else {
        unreachable!(
            "core/advance_turn requires BeatPayload::AdvanceTurn at beat `{}`",
            evt.beat_id
        );
    };

    let cast_id = ctx.cast_id;
    enqueue_targets(evt, ctx, *target, move |target_id| Intent::AdvanceTurn {
        target: target_id,
        amount_pct: (*amount_pct).min(50),
        cast_id,
    });
}

fn revive(evt: &BeatEvent, ctx: &mut crate::combat::runtime::SkillCtx<'_>) {
    let Some(BeatPayload::Revive { pct, target }) = ctx.beat_payload() else {
        unreachable!(
            "core/revive requires BeatPayload::Revive at beat `{}`",
            evt.beat_id
        );
    };

    let caster = ctx.caster;
    let cast_id = ctx.cast_id;
    enqueue_targets(evt, ctx, *target, move |target_id| Intent::Revive {
        source: caster,
        target: target_id,
        pct: *pct,
        cast_id,
    });
}

fn grant_free_skill(evt: &BeatEvent, ctx: &mut crate::combat::runtime::SkillCtx<'_>) {
    let Some(BeatPayload::GrantFreeSkill { count }) = ctx.beat_payload() else {
        unreachable!(
            "core/grant_free_skill requires BeatPayload::GrantFreeSkill at beat `{}`",
            evt.beat_id
        );
    };

    if *count == 0 {
        return;
    }

    ctx.enqueue(Intent::GrantFreeSkill {
        source: ctx.caster,
        count: *count,
        cast_id: ctx.cast_id,
    });
}

fn grant_energy(evt: &BeatEvent, ctx: &mut crate::combat::runtime::SkillCtx<'_>) {
    let Some(BeatPayload::GrantEnergy { amount }) = ctx.beat_payload() else {
        unreachable!(
            "core/grant_energy requires BeatPayload::GrantEnergy at beat `{}`",
            evt.beat_id
        );
    };

    if *amount == 0 {
        return;
    }

    ctx.enqueue(Intent::AddEnergy {
        target: ctx.caster,
        amount: *amount,
        cast_id: ctx.cast_id,
    });
}

fn self_advance(evt: &BeatEvent, ctx: &mut crate::combat::runtime::SkillCtx<'_>) {
    let Some(BeatPayload::SelfAdvance { amount_pct }) = ctx.beat_payload() else {
        unreachable!(
            "core/self_advance requires BeatPayload::SelfAdvance at beat `{}`",
            evt.beat_id
        );
    };

    let capped = (*amount_pct).clamp(0, 50) as u32;
    if capped == 0 {
        return;
    }

    ctx.enqueue(Intent::AdvanceTurn {
        target: ctx.caster,
        amount_pct: capped,
        cast_id: ctx.cast_id,
    });
}

fn apply_effect(evt: &BeatEvent, ctx: &mut crate::combat::runtime::SkillCtx<'_>) {
    match ctx.beat_payload() {
        Some(BeatPayload::DealDamage { .. }) => deal_damage(evt, ctx),
        Some(BeatPayload::BreakToughness { .. }) => break_toughness(evt, ctx),
        Some(BeatPayload::ApplyStatus { .. }) => apply_status(evt, ctx),
        Some(BeatPayload::DelayTurn { .. }) => delay_turn(evt, ctx),
        Some(BeatPayload::AdvanceTurn { .. }) => advance_turn(evt, ctx),
        Some(BeatPayload::Revive { .. }) => revive(evt, ctx),
        Some(BeatPayload::GrantFreeSkill { .. }) => grant_free_skill(evt, ctx),
        Some(BeatPayload::GrantEnergy { .. }) => grant_energy(evt, ctx),
        Some(BeatPayload::SelfAdvance { .. }) => self_advance(evt, ctx),
        Some(BeatPayload::ApplyBuff {
            kind,
            duration,
            target,
        }) => {
            let cast_id = ctx.cast_id;
            enqueue_targets(evt, ctx, *target, move |target_id| Intent::ApplyBuff {
                target: target_id,
                kind: kind.clone(),
                duration_turns: *duration,
                cast_id,
            });
        }
        Some(BeatPayload::BlueprintSignal {
            owner,
            name,
            payload,
        }) => {
            let owner: &'static str = Box::leak(owner.clone().into_boxed_str());
            let name: &'static str = Box::leak(name.clone().into_boxed_str());
            ctx.enqueue(Intent::BlueprintSignal {
                source: ctx.caster,
                owner,
                name,
                payload: payload.clone(),
                cast_id: ctx.cast_id,
            });
        }
        None => {
            unreachable!(
                "core/apply_effect requires a BeatPayload at beat `{}`",
                evt.beat_id
            );
        }
    }
}

fn enqueue_targets<F>(
    evt: &BeatEvent,
    ctx: &mut crate::combat::runtime::SkillCtx<'_>,
    target: TargetShape,
    mut build: F,
) where
    F: FnMut(UnitId) -> Intent,
{
    let targets = if !evt.beat_targets.is_empty() {
        evt.beat_targets.clone()
    } else {
        let snapshot = snapshot_targets(ctx.world);
        let resolved = resolve_targets(&target, ctx.primary_target, &snapshot);
        if resolved.is_empty() {
            vec![ctx.primary_target]
        } else {
            resolved
        }
    };

    for target_id in targets {
        ctx.enqueue(build(target_id));
    }
}

fn snapshot_targets(world: &World) -> TargetableSnapshot {
    let mut q = match world.try_query::<(&Unit, &Team, Option<&Ko>, Option<&SlotIndex>)>() {
        Some(q) => q,
        None => return TargetableSnapshot { entries: vec![] },
    };
    let entries = q
        .iter(world)
        .map(|(unit, team, ko, slot)| TargetEntry {
            id: unit.id,
            team: *team,
            slot_index: slot.map(|s| s.0).unwrap_or(0),
            alive: ko.is_none() && unit.hp_current > 0,
            hp_per_mille: if unit.hp_max > 0 {
                ((unit.hp_current.max(0) as u64 * 1000) / unit.hp_max as u64) as u32
            } else {
                0
            },
        })
        .collect();
    TargetableSnapshot { entries }
}

#[cfg(test)]
mod tests;
