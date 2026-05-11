use bevy::prelude::*;

use crate::combat::{
    StatusEffect,
    energy::{Energy, RoundEnergyTracker},
    events::{ActionIntentKind, CombatEvent, CombatEventKind},
    kernel::{CombatBeatId, CombatKernelRegistry},
    kit::{
        FollowUpConfig, FollowUpTrigger, FormIdentityConfig, FormIdentityKit, FormIdentityTrigger,
        UnitSkills,
    },
    stun::Stunned,
    team::Team,
    turn_system::{ActionIntent, emit_combat_beat, emit_combat_event, step_app, step_declaration},
    types::{Attribute, DamageTag, SkillId, UnitId},
    unit::{BasicStreak, Commander, Ko, Unit},
};
use crate::combat::{
    log::ActionLog, round_flags::RoundFlags, sp::SpPool, state::CombatState, toughness::Toughness,
    turn_order::TurnOrder, ultimate::UltimateCharge,
};
use crate::data::{SkillBookHandle, skills_ron::SkillBook};

/// Distinguishes standard follow-ups from Form Identity reactive actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FollowUpOriginKind {
    #[default]
    FollowUp,
    FormIdentity,
}

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct FollowUpIntent {
    pub attacker: UnitId,
    pub skill_id: SkillId,
    pub target: UnitId,
    pub origin: CombatEvent,
    pub origin_kind: FollowUpOriginKind,
}

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct FollowUpTrace {
    pub follower: UnitId,
    pub trigger: FollowUpTrigger,
    pub action: SkillId,
    pub origin_kind: CombatEventKind,
    pub origin_source: UnitId,
    pub origin_target: UnitId,
    pub follow_up_target: Option<UnitId>,
    pub decision: FollowUpDecision,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FollowUpDecision {
    Scheduled,
    Suppressed { reason: FollowUpSkipReason },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FollowerSnapshot {
    id: UnitId,
    team: Team,
    hp_current: i32,
    follow_up: Option<FollowUpConfig>,
    is_ko: bool,
    is_stunned: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FollowUpSkipReason {
    TriggerMismatch,
    WrongTeam,
    FollowerKo,
    FollowerStunned,
    MissingTarget,
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
        Option<&'static crate::combat::enemy_counterplay::EnemyCounterplayKit>,
        Option<&'static Ko>,
        Option<&'static Stunned>,
        Option<&'static Commander>,
        Option<&'static mut StatusEffect>,
        Option<&'static mut BasicStreak>,
        Option<&'static mut RoundFlags>,
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

fn is_alive_enemy(candidate: UnitId, follower_team: Team, roster: &[FollowerSnapshot]) -> bool {
    roster
        .iter()
        .find(|unit| unit.id == candidate)
        .is_some_and(|unit| unit.team != follower_team && !unit.is_ko && unit.hp_current > 0)
}

fn select_follow_up_target(
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

fn evaluate_follow_up(
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

struct FormIdentitySnapshot {
    id: UnitId,
    team: Team,
    form_identity: Option<FormIdentityConfig>,
    form_identity_used: bool,
    is_ko: bool,
    is_stunned: bool,
}

type FormIdentityRosterQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Unit,
        &'static Team,
        Option<&'static FormIdentityKit>,
        Option<&'static Ko>,
        Option<&'static Stunned>,
        Option<&'static RoundFlags>,
    ),
>;

fn skill_damage_tag(skill_book: Option<&SkillBook>, skill_id: &SkillId) -> Option<DamageTag> {
    skill_book
        .and_then(|book| book.0.iter().find(|skill| &skill.id == skill_id))
        .map(|skill| skill.damage_tag)
}

fn evaluate_form_identity_trigger(
    config: &FormIdentityConfig,
    event: &CombatEvent,
    follower_id: UnitId,
    attribute_map: &std::collections::HashMap<UnitId, Attribute>,
    skill_book: Option<&SkillBook>,
) -> bool {
    match &config.trigger {
        FormIdentityTrigger::OnFirstHitVsTagThisRound(tag) => {
            matches!(&event.kind, CombatEventKind::OnDamageDealt { amount, damage_tag, .. }
                if *amount > 0 && damage_tag == tag)
                && event.source == follower_id
        }
        FormIdentityTrigger::OnStatusApplied(trigger_kind) => {
            // Match by discriminant so inner field values (e.g. speed_reduction) are ignored.
            matches!(&event.kind, CombatEventKind::OnStatusApplied { kind }
                if std::mem::discriminant(kind) == std::mem::discriminant(trigger_kind))
                && event.source == follower_id // unit must be the applier, not the target
        }
        FormIdentityTrigger::OnFirstSkillCastWithTag(tag) => {
            let matches_tag = match &event.kind {
                CombatEventKind::OnSkillCast { skill_id } => skill_damage_tag(skill_book, skill_id)
                    .is_some_and(|damage_tag| damage_tag == *tag),
                CombatEventKind::OnDamageDealt {
                    amount, damage_tag, ..
                } => *amount > 0 && *damage_tag == *tag,
                _ => false,
            };
            matches_tag && event.source == follower_id
        }
        FormIdentityTrigger::OnAttackVsAttribute(attr) => {
            // Fires when this unit deals damage to an enemy whose Attribute matches.
            matches!(&event.kind, CombatEventKind::OnDamageDealt { amount, .. } if *amount > 0)
                && event.source == follower_id
                && attribute_map.get(&event.target) == Some(attr)
        }
    }
}

pub fn form_identity_listener_system(
    mut events: MessageReader<CombatEvent>,
    mut follow_up_writer: MessageWriter<FollowUpIntent>,
    skill_books: Res<Assets<SkillBook>>,
    skill_book_handle: Option<Res<SkillBookHandle>>,
    roster: FormIdentityRosterQuery,
) {
    let fi_snapshots: Vec<FormIdentitySnapshot> = roster
        .iter()
        .map(
            |(unit, team, fi_kit, ko, stunned, flags)| FormIdentitySnapshot {
                id: unit.id,
                team: *team,
                form_identity: fi_kit.map(|k| k.config.clone()),
                form_identity_used: flags.map(|f| f.form_identity_used).unwrap_or(false),
                is_ko: ko.is_some(),
                is_stunned: stunned.is_some(),
            },
        )
        .collect();

    let skill_book = skill_book_handle.and_then(|handle| skill_books.get(&handle.0));

    // Reuse the FollowerSnapshot type for target selection (reuse select_follow_up_target).
    let target_snapshots: Vec<FollowerSnapshot> = roster
        .iter()
        .map(|(unit, team, _, ko, stunned, _)| FollowerSnapshot {
            id: unit.id,
            team: *team,
            hp_current: unit.hp_current,
            follow_up: None,
            is_ko: ko.is_some(),
            is_stunned: stunned.is_some(),
        })
        .collect();

    // Attribute lookup used by OnAttackVsAttribute trigger.
    let attribute_map: std::collections::HashMap<UnitId, Attribute> = roster
        .iter()
        .map(|(unit, _, _, _, _, _)| (unit.id, unit.attribute))
        .collect();

    // Guard: each unit fires form identity at most once per listener invocation.
    let mut triggered_this_frame: std::collections::HashSet<UnitId> =
        std::collections::HashSet::new();

    for event in events.read() {
        for follower in &fi_snapshots {
            let Some(config) = follower.form_identity.as_ref() else {
                continue;
            };
            if follower.form_identity_used || triggered_this_frame.contains(&follower.id) {
                continue;
            }
            if follower.is_ko {
                continue;
            }

            if !evaluate_form_identity_trigger(
                config,
                event,
                follower.id,
                &attribute_map,
                skill_book,
            ) {
                continue;
            }

            let Some(target) = select_follow_up_target(follower.team, event, &target_snapshots)
            else {
                continue;
            };

            info!(
                target: "combat.form_identity",
                trigger = ?config.trigger,
                follower = ?follower.id,
                origin_kind = ?event.kind,
                origin_source = ?event.source,
                follow_up_target = ?target,
                action = ?config.action,
                "form identity scheduled"
            );

            triggered_this_frame.insert(follower.id);
            follow_up_writer.write(FollowUpIntent {
                attacker: follower.id,
                skill_id: config.action.clone(),
                target,
                origin: event.clone(),
                origin_kind: FollowUpOriginKind::FormIdentity,
            });
        }
    }
}

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
) {
    if let Some(intent) = intents.read().next() {
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
        );
        emit_combat_beat(
            &mut event_writer,
            registry.as_deref(),
            CombatBeatId::Declared,
            inflight.action.source,
            inflight.action.target,
            inflight.follow_up_depth,
        );
        emit_combat_event(
            &mut event_writer,
            CombatEventKind::OnActionPreApp,
            inflight.action.source,
            inflight.action.target,
            inflight.follow_up_depth,
        );
        emit_combat_beat(
            &mut event_writer,
            registry.as_deref(),
            CombatBeatId::PreApp,
            inflight.action.source,
            inflight.action.target,
            inflight.follow_up_depth,
        );

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
        );

        emit_combat_event(
            &mut event_writer,
            CombatEventKind::OnActionApplied,
            inflight.action.source,
            inflight.action.target,
            inflight.follow_up_depth,
        );
        emit_combat_beat(
            &mut event_writer,
            registry.as_deref(),
            CombatBeatId::Applied,
            inflight.action.source,
            inflight.action.target,
            inflight.follow_up_depth,
        );
        emit_combat_event(
            &mut event_writer,
            CombatEventKind::OnActionResolved,
            inflight.action.source,
            inflight.action.target,
            inflight.follow_up_depth,
        );
        emit_combat_beat(
            &mut event_writer,
            registry.as_deref(),
            CombatBeatId::Resolved,
            inflight.action.source,
            inflight.action.target,
            inflight.follow_up_depth,
        );

        if intent.origin_kind == FollowUpOriginKind::FormIdentity {
            for (_, _, unit, _, _, _, _, _, _, _, _, _, mut round_flags) in actors.iter_mut() {
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

#[cfg(test)]
#[path = "follow_up_tests.rs"]
mod follow_up_tests;
