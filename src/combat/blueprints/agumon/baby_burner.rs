use crate::combat::{
    resolution::{TargetEntry, TargetableSnapshot, resolve_targets},
    runtime::{Intent, PostActionContext, PostActionQueue, SignalPayload},
    team::Team,
    types::{DamageTag, UnitId},
};
use crate::data::skills_ron::TargetShape;

use super::OWNER;

pub const DETONATE_SIGNAL_NAME: &str = "baby_burner_detonate";

const BABY_BURNER_SKILL_ID: &str = "agumon_ult";
const DETONATE_DAMAGE_PER_HEATED: i32 = 8;

pub fn enqueue_reactive_detonate(ctx: &PostActionContext, out: &mut PostActionQueue) {
    if ctx.skill_id.0 != BABY_BURNER_SKILL_ID {
        return;
    }

    let Some(unit_died) = &ctx.unit_died else {
        return;
    };
    if unit_died.heated_remaining == 0 {
        return;
    }

    let Some(source_team) = ctx.source_unit().map(|unit| unit.team) else {
        return;
    };
    let Some(primary_target) = ctx.primary_target_unit() else {
        return;
    };
    if primary_target.slot_index.is_none() || primary_target.team == source_team {
        return;
    }

    let Some(amount) = i32::try_from(unit_died.heated_remaining)
        .ok()
        .and_then(|heated| heated.checked_mul(DETONATE_DAMAGE_PER_HEATED))
    else {
        return;
    };

    for target in detonate_targets(ctx, source_team) {
        out.push_intent(Intent::DealDamage {
            source: ctx.source,
            target,
            amount,
            tag: DamageTag::Fire,
            cast_id: ctx.cast_id,
        });
        out.push_intent(Intent::BlueprintSignal {
            source: ctx.source,
            owner: OWNER,
            name: DETONATE_SIGNAL_NAME,
            payload: SignalPayload::UnitTarget(target),
            cast_id: ctx.cast_id,
        });
    }
}

fn detonate_targets(ctx: &PostActionContext, source_team: Team) -> Vec<UnitId> {
    resolve_targets(
        &TargetShape::Blast,
        ctx.primary_target,
        &targetable_snapshot(ctx),
    )
    .into_iter()
    .filter(|target_id| {
        *target_id != ctx.primary_target
            && ctx
                .unit(*target_id)
                .is_some_and(|unit| unit.alive && unit.team != source_team)
    })
    .collect()
}

fn targetable_snapshot(ctx: &PostActionContext) -> TargetableSnapshot {
    TargetableSnapshot {
        entries: ctx
            .roster
            .iter()
            .filter_map(|unit| {
                Some(TargetEntry {
                    id: unit.unit_id,
                    team: unit.team,
                    slot_index: unit.slot_index?,
                    alive: unit.alive,
                    hp_per_mille: hp_per_mille(unit.hp_current, unit.hp_max),
                })
            })
            .collect(),
    }
}

fn hp_per_mille(hp_current: i32, hp_max: i32) -> u32 {
    if hp_max <= 0 {
        return 0;
    }

    let hp_current = i64::from(hp_current.max(0));
    let hp_max = i64::from(hp_max);
    ((hp_current * 1000) / hp_max) as u32
}
