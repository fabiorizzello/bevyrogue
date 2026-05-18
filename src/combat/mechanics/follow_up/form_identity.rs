use bevy::prelude::*;

use crate::combat::{
    events::{CombatEvent, CombatEventKind},
    kit::{FormIdentityConfig, FormIdentityKit, FormIdentityTrigger},
    round_flags::RoundFlags,
    stun::Stunned,
    team::Team,
    types::{Attribute, DamageTag, SkillId, UnitId},
    unit::{Ko, Unit},
};
use crate::data::{SkillBookHandle, skills_ron::SkillBook};

use super::triggers::{FollowerSnapshot, select_follow_up_target};
use super::types::{FollowUpIntent, FollowUpOriginKind};

struct FormIdentitySnapshot {
    id: UnitId,
    team: Team,
    form_identity: Option<FormIdentityConfig>,
    form_identity_used: bool,
    is_ko: bool,
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
            |(unit, team, fi_kit, ko, _stunned, flags)| FormIdentitySnapshot {
                id: unit.id,
                team: *team,
                form_identity: fi_kit.map(|k| k.config.clone()),
                form_identity_used: flags.map(|f| f.form_identity_used).unwrap_or(false),
                is_ko: ko.is_some(),
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
