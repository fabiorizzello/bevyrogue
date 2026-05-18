use super::*;
use crate::combat::runtime::intent::CastId;
use crate::combat::enemy_ai;
use crate::combat::{
    kit::UnitSkills,
    preview::{summarize_preview_damage, try_query_skill_preview},
    team::Team,
    toughness::Toughness,
    types::{SkillId, UnitId},
    ultimate::UltimateCharge,
    unit::{Commander, Ko, Unit},
};
use bevy::prelude::*;

pub fn resolve_enemy_turn_action_system(world: &mut World) {
    let requests = {
        let Some(mut queue) = world.get_resource_mut::<EnemyTurnRequestQueue>() else {
            return;
        };
        std::mem::take(&mut queue.0)
    };

    if requests.is_empty() {
        return;
    }

    #[derive(Clone)]
    struct Snapshot {
        id: UnitId,
        team: Team,
        is_commander: bool,
        toughness_current: i32,
        toughness_max: i32,
        hp_current: i32,
        hp_max: i32,
        skills: Option<UnitSkills>,
        ult_ready: bool,
        alive: bool,
    }

    let mut snapshots = Vec::new();
    let mut query = world.query::<(
        &Unit,
        &Team,
        Option<&Toughness>,
        Option<&UnitSkills>,
        Option<&UltimateCharge>,
        Option<&Ko>,
        Option<&Commander>,
    )>();
    for (unit, team, toughness, skills, ult, ko, commander) in query.iter(world) {
        snapshots.push(Snapshot {
            id: unit.id,
            team: *team,
            is_commander: commander.is_some(),
            toughness_current: toughness.map(|value| value.current).unwrap_or(0),
            toughness_max: toughness.map(|value| value.max).unwrap_or(1),
            hp_current: unit.hp_current,
            hp_max: unit.hp_max,
            skills: skills.cloned(),
            ult_ready: ult.map(|value| value.ready()).unwrap_or(false),
            alive: ko.is_none() && unit.hp_current > 0,
        });
    }

    for attacker_id in requests {
        let Some(attacker) = snapshots.iter().find(|snapshot| {
            snapshot.id == attacker_id && snapshot.team == Team::Enemy && snapshot.alive
        }) else {
            continue;
        };

        let fallback_skills;
        let skills_ref: &UnitSkills = if let Some(skills) = attacker.skills.as_ref() {
            skills
        } else {
            fallback_skills = UnitSkills {
                basic: SkillId(String::new()),
                skills: Vec::new(),
                ultimate: SkillId(String::new()),
                follow_up: None,
            };
            &fallback_skills
        };

        let ally_targets: Vec<enemy_ai::TargetInfo> = snapshots
            .iter()
            .filter(|snapshot| {
                snapshot.team == Team::Ally && !snapshot.is_commander && snapshot.alive
            })
            .map(|snapshot| enemy_ai::TargetInfo {
                id: snapshot.id,
                toughness_current: snapshot.toughness_current,
                toughness_max: snapshot.toughness_max,
                hp_current: snapshot.hp_current,
                hp_max: snapshot.hp_max,
            })
            .collect();

        let ctx = enemy_ai::EnemyTurnContext {
            attacker_id,
            attacker_skills: skills_ref,
            attacker_ult_ready: attacker.ult_ready,
            targets: &ally_targets,
        };

        if let Some(intent) = enemy_ai::pick_enemy_action_with_preview(&ctx, |skill_id, target| {
            let pending =
                try_query_skill_preview(world, skill_id, CastId::ROOT, attacker_id, target)?;
            Some(summarize_preview_damage(&pending))
        }) {
            world.write_message(intent);
        }
    }
}

