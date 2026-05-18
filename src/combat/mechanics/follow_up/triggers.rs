use bevy::prelude::*;

use crate::combat::{
    events::{CombatEvent, CombatEventKind},
    kit::{FollowUpConfig, FollowUpTrigger, UnitSkills},
    stun::Stunned,
    team::Team,
    types::UnitId,
    unit::{Ko, Unit},
};

use super::types::{
    FollowUpDecision, FollowUpIntent, FollowUpOriginKind, FollowUpSkipReason, FollowUpTrace,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FollowerSnapshot {
    pub(super) id: UnitId,
    pub(super) team: Team,
    pub(super) hp_current: i32,
    pub(super) follow_up: Option<FollowUpConfig>,
    pub(super) is_ko: bool,
    pub(super) is_stunned: bool,
}

type FollowUpRosterQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Unit,
        &'static Team,
        &'static UnitSkills,
        Option<&'static Ko>,
        Option<&'static Stunned>,
    ),
>;

fn trigger_matches(trigger: &FollowUpTrigger, event_kind: &CombatEventKind) -> bool {
    matches!(
        (trigger, event_kind),
        (
            FollowUpTrigger::OnEnemyBreak,
            CombatEventKind::OnBreak { .. }
        ) | (FollowUpTrigger::OnAllyLowHp, CombatEventKind::OnAllyLowHp)
            | (FollowUpTrigger::OnEnemyKill, CombatEventKind::OnEnemyKill)
    )
}

fn team_for(id: UnitId, roster: &[FollowerSnapshot]) -> Option<Team> {
    roster
        .iter()
        .find(|unit| unit.id == id)
        .map(|unit| unit.team)
}

pub(super) fn is_alive_enemy(
    candidate: UnitId,
    follower_team: Team,
    roster: &[FollowerSnapshot],
) -> bool {
    roster
        .iter()
        .find(|unit| unit.id == candidate)
        .is_some_and(|unit| unit.team != follower_team && !unit.is_ko && unit.hp_current > 0)
}

pub(super) fn select_follow_up_target(
    follower_team: Team,
    event: &CombatEvent,
    roster: &[FollowerSnapshot],
) -> Option<UnitId> {
    if is_alive_enemy(event.target, follower_team, roster) {
        return Some(event.target);
    }
    if is_alive_enemy(event.source, follower_team, roster) {
        return Some(event.source);
    }

    roster
        .iter()
        .filter(|unit| unit.team != follower_team && !unit.is_ko && unit.hp_current > 0)
        .map(|unit| unit.id)
        .min_by_key(|id| id.0)
}

fn follower_is_allied_to_trigger(
    follower: &FollowerSnapshot,
    event: &CombatEvent,
    roster: &[FollowerSnapshot],
) -> bool {
    match event.kind {
        CombatEventKind::OnAllyLowHp => team_for(event.target, roster) == Some(follower.team),
        CombatEventKind::OnBreak { .. } | CombatEventKind::OnEnemyKill => {
            let source_team = team_for(event.source, roster);
            let target_team = team_for(event.target, roster);
            source_team == Some(follower.team)
                && target_team.is_some_and(|team| team != follower.team)
        }
        _ => false,
    }
}

pub(super) fn evaluate_follow_up(
    follower: &FollowerSnapshot,
    event: &CombatEvent,
    roster: &[FollowerSnapshot],
) -> Result<UnitId, FollowUpSkipReason> {
    let Some(config) = follower.follow_up.as_ref() else {
        return Err(FollowUpSkipReason::TriggerMismatch);
    };

    if !trigger_matches(&config.trigger, &event.kind) {
        return Err(FollowUpSkipReason::TriggerMismatch);
    }

    if !follower_is_allied_to_trigger(follower, event, roster) {
        return Err(FollowUpSkipReason::WrongTeam);
    }

    if follower.is_ko || follower.hp_current <= 0 {
        return Err(FollowUpSkipReason::FollowerKo);
    }

    if follower.is_stunned {
        return Err(FollowUpSkipReason::FollowerStunned);
    }

    select_follow_up_target(follower.team, event, roster).ok_or(FollowUpSkipReason::MissingTarget)
}

fn emit_follow_up_trace(
    trace_writer: &mut MessageWriter<FollowUpTrace>,
    follower: &FollowerSnapshot,
    config: &FollowUpConfig,
    event: &CombatEvent,
    follow_up_target: Option<UnitId>,
    decision: FollowUpDecision,
) {
    trace_writer.write(FollowUpTrace {
        follower: follower.id,
        trigger: config.trigger.clone(),
        action: config.action.clone(),
        origin_kind: event.kind.clone(),
        origin_source: event.source,
        origin_target: event.target,
        follow_up_target,
        decision,
    });
}

pub fn follow_up_listener_system(
    mut events: MessageReader<CombatEvent>,
    mut follow_up_writer: MessageWriter<FollowUpIntent>,
    mut trace_writer: MessageWriter<FollowUpTrace>,
    roster: FollowUpRosterQuery,
) {
    let snapshots: Vec<FollowerSnapshot> = roster
        .iter()
        .map(|(unit, team, skills, ko, stunned)| FollowerSnapshot {
            id: unit.id,
            team: *team,
            hp_current: unit.hp_current,
            follow_up: skills.follow_up.clone(),
            is_ko: ko.is_some(),
            is_stunned: stunned.is_some(),
        })
        .collect();

    for event in events.read() {
        for follower in &snapshots {
            let Some(config) = follower.follow_up.as_ref() else {
                continue;
            };

            match evaluate_follow_up(follower, event, &snapshots) {
                Ok(target) => {
                    info!(
                        target: "combat.follow_up",
                        trigger = ?config.trigger,
                        follower = ?follower.id,
                        origin_kind = ?event.kind,
                        origin_source = ?event.source,
                        origin_target = ?event.target,
                        follow_up_target = ?target,
                        action = ?config.action,
                        "follow-up scheduled"
                    );
                    emit_follow_up_trace(
                        &mut trace_writer,
                        follower,
                        config,
                        event,
                        Some(target),
                        FollowUpDecision::Scheduled,
                    );
                    follow_up_writer.write(FollowUpIntent {
                        attacker: follower.id,
                        skill_id: config.action.clone(),
                        target,
                        origin: event.clone(),
                        origin_kind: FollowUpOriginKind::FollowUp,
                    });
                }
                Err(reason) => {
                    debug!(
                        target: "combat.follow_up",
                        trigger = ?config.trigger,
                        follower = ?follower.id,
                        origin_kind = ?event.kind,
                        origin_source = ?event.source,
                        origin_target = ?event.target,
                        reason = ?reason,
                        "follow-up suppressed"
                    );
                    emit_follow_up_trace(
                        &mut trace_writer,
                        follower,
                        config,
                        event,
                        None,
                        FollowUpDecision::Suppressed { reason },
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{FollowerSnapshot, evaluate_follow_up};
    use crate::combat::events::{CombatEvent, CombatEventKind};
    use crate::combat::kit::{FollowUpConfig, FollowUpTrigger};
    use crate::combat::runtime::intent::CastId;
    use crate::combat::team::Team;
    use crate::combat::types::{DamageTag, SkillId, UnitId};

    use super::super::types::FollowUpSkipReason;

    #[test]
    fn follow_up_reports_ineligible_reasons() {
        let roster = vec![
            FollowerSnapshot {
                id: UnitId(1),
                team: Team::Ally,
                hp_current: 100,
                follow_up: Some(FollowUpConfig {
                    trigger: FollowUpTrigger::OnEnemyBreak,
                    action: SkillId("follow_up".into()),
                }),
                is_ko: false,
                is_stunned: false,
            },
            FollowerSnapshot {
                id: UnitId(2),
                team: Team::Enemy,
                hp_current: 100,
                follow_up: Some(FollowUpConfig {
                    trigger: FollowUpTrigger::OnEnemyBreak,
                    action: SkillId("follow_up".into()),
                }),
                is_ko: false,
                is_stunned: false,
            },
            FollowerSnapshot {
                id: UnitId(3),
                team: Team::Ally,
                hp_current: 0,
                follow_up: Some(FollowUpConfig {
                    trigger: FollowUpTrigger::OnEnemyBreak,
                    action: SkillId("follow_up".into()),
                }),
                is_ko: true,
                is_stunned: false,
            },
            FollowerSnapshot {
                id: UnitId(4),
                team: Team::Ally,
                hp_current: 100,
                follow_up: Some(FollowUpConfig {
                    trigger: FollowUpTrigger::OnEnemyBreak,
                    action: SkillId("follow_up".into()),
                }),
                is_ko: false,
                is_stunned: true,
            },
            FollowerSnapshot {
                id: UnitId(5),
                team: Team::Ally,
                hp_current: 100,
                follow_up: Some(FollowUpConfig {
                    trigger: FollowUpTrigger::OnEnemyKill,
                    action: SkillId("follow_up".into()),
                }),
                is_ko: false,
                is_stunned: false,
            },
            FollowerSnapshot {
                id: UnitId(6),
                team: Team::Enemy,
                hp_current: 100,
                follow_up: None,
                is_ko: false,
                is_stunned: false,
            },
        ];

        let root_break = CombatEvent {
            kind: CombatEventKind::OnBreak {
                damage_tag: DamageTag::Fire,
            },
            source: UnitId(1),
            target: UnitId(6),
            follow_up_depth: 0,
            cast_id: CastId::ROOT,
        };

        assert_eq!(
            evaluate_follow_up(&roster[1], &root_break, &roster),
            Err(FollowUpSkipReason::WrongTeam)
        );
        assert_eq!(
            evaluate_follow_up(&roster[2], &root_break, &roster),
            Err(FollowUpSkipReason::FollowerKo)
        );
        assert_eq!(
            evaluate_follow_up(&roster[3], &root_break, &roster),
            Err(FollowUpSkipReason::FollowerStunned)
        );
        assert_eq!(
            evaluate_follow_up(&roster[4], &root_break, &roster),
            Err(FollowUpSkipReason::TriggerMismatch)
        );
    }
}
