#[cfg(feature = "windowed")]
use bevy::prelude::*;

#[cfg(feature = "windowed")]
use crate::combat::{
    action_query::{ActionStatus, build_snapshot_from_ecs_with_sp, first_enabled_target_id},
    counterplay::EnemyCounterplayKit,
    energy::{Energy, RoundEnergyTracker},
    kit::UnitSkills,
    preview::{PreviewDamageSummary, query_skill_preview, summarize_preview_damage},
    runtime::intent::CastIdGen,
    sp::SpPool,
    state::CombatState,
    stun::Stunned,
    team::Team,
    toughness::Toughness,
    turn_order::TurnOrder,
    types::{SkillId, UnitId},
    ult_gauge::UltGaugeMetadata,
    ultimate::UltimateCharge,
    unit::{Commander, Ko, Unit},
};
#[cfg(feature = "windowed")]
use crate::data::{SkillBookHandle, skills_ron::SkillBook};

#[cfg(feature = "windowed")]
use super::{
    PendingAction, PreviewDamageCache, labels::query_pending_action_affordance,
    pending_kind_skill_id,
};

#[cfg(feature = "windowed")]
pub fn refresh_preview_damage_cache(world: &mut World) {
    let Some(active_actor_id) = world
        .get_resource::<TurnOrder>()
        .and_then(|order| order.active_unit)
    else {
        return;
    };

    let Some(pending_kind) = world
        .get_resource::<PendingAction>()
        .and_then(|pending| pending.kind.clone())
    else {
        return;
    };

    let Some((skill_id, target_id, summary)) =
        (|| -> Option<(SkillId, UnitId, PreviewDamageSummary)> {
            let skill_book = world
                .get_resource::<Assets<SkillBook>>()
                .and_then(|assets| {
                    world
                        .get_resource::<SkillBookHandle>()
                        .and_then(|handle| assets.get(&handle.0).cloned())
                })?;
            let Some(mut cast_id_gen) = world.get_resource_mut::<CastIdGen>() else {
                return None;
            };
            let cast_id = cast_id_gen.next();
            let combat_state = world.resource::<CombatState>().clone();
            let order = world.resource::<TurnOrder>().clone();
            let sp_current = world.resource::<SpPool>().current;

            let mut units_data = Vec::new();
            let mut active_kit: Option<UnitSkills> = None;
            let mut units_q = world.query::<(
                &'static Unit,
                &'static Team,
                Option<&'static Toughness>,
                Option<&'static EnemyCounterplayKit>,
                &'static UltimateCharge,
                &'static UnitSkills,
                Option<&'static Ko>,
                Option<&'static Commander>,
                Option<&'static Stunned>,
                Option<&'static Energy>,
                Option<&'static RoundEnergyTracker>,
                Option<&'static UltGaugeMetadata>,
            )>();
            for (
                unit,
                team,
                tough,
                counterplay,
                ult,
                kit,
                ko,
                commander,
                stunned,
                energy,
                tracker,
                gauge_meta,
            ) in units_q.iter(world)
            {
                if unit.id == active_actor_id {
                    active_kit = Some(kit.clone());
                }
                units_data.push((
                    unit.id,
                    *team,
                    unit,
                    Some(kit),
                    Some(ult),
                    tough,
                    counterplay,
                    ko.is_some(),
                    stunned.is_some(),
                    commander.is_some(),
                    energy,
                    tracker,
                    gauge_meta,
                ));
            }

            let kit = active_kit?;
            let snapshot = build_snapshot_from_ecs_with_sp(
                &combat_state,
                &order,
                sp_current,
                active_actor_id,
                active_actor_id,
                units_data,
            );

            let affordance = query_pending_action_affordance(
                &snapshot,
                &skill_book,
                active_actor_id,
                &pending_kind,
            );
            if !matches!(affordance.action, ActionStatus::Enabled) {
                return None;
            }

            let target_id = first_enabled_target_id(&affordance)?;
            let skill_id = pending_kind_skill_id(&pending_kind, &kit);
            let preview_pending =
                query_skill_preview(world, &skill_id, cast_id, active_actor_id, target_id);
            let summary = summarize_preview_damage(&preview_pending);
            Some((skill_id, target_id, summary))
        })()
    else {
        return;
    };

    let mut cache = world.resource_mut::<PreviewDamageCache>();
    cache.actor_id = Some(active_actor_id);
    cache.pending_kind = Some(pending_kind);
    cache.skill_id = Some(skill_id);
    cache.target_id = Some(target_id);
    cache.summary = Some(summary);
}
