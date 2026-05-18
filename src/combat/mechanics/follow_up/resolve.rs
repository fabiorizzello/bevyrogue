use bevy::prelude::*;

use crate::combat::{
    StatusBag,
    buffs::DrBag,
    energy::{Energy, RoundEnergyTracker},
    events::{ActionIntentKind, CombatEvent, CombatEventKind},
    kernel::{CombatBeatId, CombatKernelRegistry},
    log::ActionLog,
    round_flags::RoundFlags,
    sp::SpPool,
    state::CombatState,
    toughness::Toughness,
    turn_order::TurnOrder,
    turn_system::{ActionIntent, emit_combat_beat, emit_combat_event, step_app, step_declaration},
    ultimate::UltimateCharge,
    unit::{BasicStreak, Commander, Ko, SlotIndex, Unit},
};
use crate::data::{SkillBookHandle, skills_ron::SkillBook};

use super::types::{FollowUpIntent, FollowUpOriginKind};

type ResolveActorsQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Team,
        &'static mut Unit,
        Option<&'static UnitSkills>,
        Option<&'static mut UltimateCharge>,
        Option<&'static mut Toughness>,
        Option<&'static crate::combat::counterplay::EnemyCounterplayKit>,
        Option<&'static Ko>,
        Option<&'static Stunned>,
        Option<&'static Commander>,
        Option<&'static mut StatusBag>,
        Option<&'static mut BasicStreak>,
        Option<&'static mut RoundFlags>,
        Option<&'static SlotIndex>,
        Option<&'static mut DrBag>,
    ),
>;

use crate::combat::{
    kit::UnitSkills,
    stun::Stunned,
    team::Team,
};

#[allow(clippy::too_many_arguments)]
pub fn resolve_follow_up_action_system(
    mut commands: Commands,
    mut intents: MessageReader<FollowUpIntent>,
    mut state: ResMut<CombatState>,
    mut sp: ResMut<SpPool>,
    mut log: ResMut<ActionLog>,
    mut turn_order: ResMut<TurnOrder>,
    time: Res<Time>,
    skill_books: Res<Assets<SkillBook>>,
    skill_book_handle: Option<Res<SkillBookHandle>>,
    mut event_writer: MessageWriter<CombatEvent>,
    registry: Option<Res<CombatKernelRegistry>>,
    mut actors: ResolveActorsQuery,
    mut combat_rng: Option<ResMut<crate::combat::rng::CombatRng>>,
    mut energy_q: Query<(&mut Energy, Option<&mut RoundEnergyTracker>)>,
    mut cast_id_gen: Option<ResMut<crate::combat::runtime::intent::CastIdGen>>,
) {
    if let Some(intent) = intents.read().next() {
        #[cfg(debug_assertions)]
        let _combat_follow_up_span = bevy::log::info_span!(
            target: "combat.follow_up",
            "combat.follow_up.resolve",
            follower = ?intent.attacker,
            skill_id = ?intent.skill_id,
            defender = ?intent.target,
            origin_kind = ?intent.origin.kind,
            origin_source = ?intent.origin.source,
            origin_target = ?intent.origin.target,
        )
        .entered();

        debug!(
            target: "combat.follow_up",
            follower = ?intent.attacker,
            skill_id = ?intent.skill_id,
            target = ?intent.target,
            origin_kind = ?intent.origin.kind,
            origin_source = ?intent.origin.source,
            origin_target = ?intent.origin.target,
            "resolving scheduled follow-up"
        );

        let action = ActionIntent::Skill {
            attacker: intent.attacker,
            skill_id: intent.skill_id.clone(),
            target: intent.target,
        };

        let Some(inflight) = step_declaration(
            &mut commands,
            &action,
            intent.origin.follow_up_depth + 1,
            &mut state,
            intent.origin_kind,
            &skill_books,
            skill_book_handle.as_ref(),
            &mut log,
            &mut event_writer,
            &mut actors,
        ) else {
            return;
        };

        emit_combat_event(
            &mut event_writer,
            CombatEventKind::OnActionDeclared {
                intent_kind: ActionIntentKind::Skill,
            },
            inflight.action.source,
            inflight.action.target,
            inflight.follow_up_depth,
            crate::combat::runtime::intent::CastId::ROOT,
        );
        emit_combat_beat(
            &mut event_writer,
            registry.as_deref(),
            CombatBeatId::Declared,
            inflight.action.source,
            inflight.action.target,
            inflight.follow_up_depth,
            crate::combat::runtime::intent::CastId::ROOT,
        );
        emit_combat_event(
            &mut event_writer,
            CombatEventKind::OnActionPreApp,
            inflight.action.source,
            inflight.action.target,
            inflight.follow_up_depth,
            crate::combat::runtime::intent::CastId::ROOT,
        );
        emit_combat_beat(
            &mut event_writer,
            registry.as_deref(),
            CombatBeatId::PreApp,
            inflight.action.source,
            inflight.action.target,
            inflight.follow_up_depth,
            crate::combat::runtime::intent::CastId::ROOT,
        );

        let follow_up_cast_id = cast_id_gen
            .as_deref_mut()
            .map(|g| g.next())
            .unwrap_or(crate::combat::runtime::intent::CastId::ROOT);

        step_app(
            &mut commands,
            &inflight,
            &mut state,
            &mut sp,
            &mut log,
            &mut turn_order,
            &time,
            &mut event_writer,
            registry.as_deref(),
            &mut actors,
            &mut combat_rng,
            &mut energy_q,
            follow_up_cast_id,
        );

        emit_combat_event(
            &mut event_writer,
            CombatEventKind::OnActionApplied,
            inflight.action.source,
            inflight.action.target,
            inflight.follow_up_depth,
            crate::combat::runtime::intent::CastId::ROOT,
        );
        emit_combat_beat(
            &mut event_writer,
            registry.as_deref(),
            CombatBeatId::Applied,
            inflight.action.source,
            inflight.action.target,
            inflight.follow_up_depth,
            crate::combat::runtime::intent::CastId::ROOT,
        );
        emit_combat_event(
            &mut event_writer,
            CombatEventKind::OnActionResolved,
            inflight.action.source,
            inflight.action.target,
            inflight.follow_up_depth,
            crate::combat::runtime::intent::CastId::ROOT,
        );
        emit_combat_beat(
            &mut event_writer,
            registry.as_deref(),
            CombatBeatId::Resolved,
            inflight.action.source,
            inflight.action.target,
            inflight.follow_up_depth,
            crate::combat::runtime::intent::CastId::ROOT,
        );

        if intent.origin_kind == FollowUpOriginKind::FormIdentity {
            for (_, _, unit, _, _, _, _, _, _, _, _, _, mut round_flags, _, _) in actors.iter_mut()
            {
                if unit.id == intent.attacker {
                    if let Some(ref mut flags) = round_flags {
                        flags.form_identity_used = true;
                    }
                    break;
                }
            }
        }
    }
}
