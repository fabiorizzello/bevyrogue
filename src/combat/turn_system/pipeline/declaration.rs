use bevy::prelude::*;

use crate::combat::events::{CombatEvent, CombatEventKind};
use crate::combat::log::{ActionLog, LogEntry};
use crate::combat::resolution::{resolve_action, target_shape_rejection_reason};
use crate::combat::runtime::intent::CastId;
use crate::combat::state::InFlightAction;
use crate::data::{
    SkillBookHandle,
    skills_ron::{SkillBook, TargetShape},
};

use super::super::{ActionIntent, ResolveActorsQuery, emit_combat_event};

pub(crate) fn step_declaration(
    _commands: &mut Commands,
    intent: &ActionIntent,
    follow_up_depth: u8,
    _state: &mut ResMut<crate::combat::state::CombatState>,
    follow_up_origin_kind: super::super::super::follow_up::FollowUpOriginKind,
    skill_books: &Res<Assets<SkillBook>>,
    skill_book_handle: Option<&Res<SkillBookHandle>>,
    log: &mut ResMut<ActionLog>,
    event_writer: &mut MessageWriter<CombatEvent>,
    actors: &mut ResolveActorsQuery,
) -> Option<InFlightAction> {
    let (attacker_id, _target_id) = match intent {
        ActionIntent::Basic { attacker, target }
        | ActionIntent::Skill {
            attacker, target, ..
        }
        | ActionIntent::Ultimate { attacker, target } => (*attacker, *target),
    };

    let (_entity, kit) =
        actors
            .iter()
            .find_map(|(entity, _, unit, kit, _, _, _, _, _, _, _, _, _, _, _)| {
                if unit.id == attacker_id {
                    Some((entity, kit))
                } else {
                    None
                }
            })?;

    let Some(kit) = kit else {
        return None;
    };
    let skill_book = skill_book_handle.and_then(|h| skill_books.get(&h.0));
    let mut action = resolve_action(intent, kit, skill_book)?;

    if follow_up_origin_kind == super::super::super::follow_up::FollowUpOriginKind::FormIdentity
        && action.target_shape == TargetShape::SelfOnly
        && action.base_damage == 0
        && action.toughness_damage == 0
        && action.revive_pct == 0
    {
        action.target = action.source;
    } else if follow_up_origin_kind
        != super::super::super::follow_up::FollowUpOriginKind::FormIdentity
        && let Some(reason) = target_shape_rejection_reason(action.target_shape)
    {
        log.push(LogEntry::ActionFailed {
            reason: reason.clone(),
        });
        emit_combat_event(
            event_writer,
            CombatEventKind::OnActionFailed { reason },
            action.source,
            action.target,
            follow_up_depth,
            CastId::ROOT,
        );
        return None;
    }

    let inflight = InFlightAction {
        action,
        interrupted: false,
        follow_up_depth,
    };

    Some(inflight)
}
