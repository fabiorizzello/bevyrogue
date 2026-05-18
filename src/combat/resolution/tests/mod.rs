use super::*;
use crate::combat::{
    team::Team,
    turn_system::ActionIntent,
    types::{Attribute, DamageTag, EvoStage, SkillId, UnitId},
    ultimate::UltAccumulationTrigger,
};
use crate::combat::events::CombatEventKind;
use crate::combat::kit::UnitSkills;

use crate::combat::ultimate::UltimateCharge;
use crate::combat::unit::Unit;
use crate::data::skills_ron::{
    Effect, SelfTargetRule, SkillBook, SkillDef, SkillImplementation,
    SkillTargeting, TargetLife, TargetShape, TargetSide,
};

mod resolve_apply;
mod streak;
mod targets;
mod bounce;

fn grant_free_skill_def(id: &str, grant_count: usize) -> SkillDef {
    SkillDef {
        id: SkillId(id.into()),
        name: id.into(),
        damage_tag: DamageTag::Light,
        sp_cost: 0,
        targeting: SkillTargeting {
            shape: TargetShape::Single,
            side: TargetSide::Enemy,
            life: TargetLife::Alive,
            self_rule: SelfTargetRule::Forbid,
            ..Default::default()
        },
        implementation: SkillImplementation::Implemented,
        legacy_ops: vec![
            Effect::Damage {
                amount: 30,
                target: TargetShape::Single,
                per_hop: Default::default(),
            },
            Effect::ToughnessHit(15),
            Effect::GrantFreeSkill { count: grant_count },
        ],

        custom_signals: vec![],
        animation_sequence: None,
        qte: None,
        ..Default::default()
    }
}

fn grant_free_skill_events(count: usize, ally_basics: &[SkillId]) -> Vec<CombatEventKind> {
    ally_basics
        .iter()
        .take(count)
        .cloned()
        .map(|skill_id| CombatEventKind::OnSkillCast { skill_id })
        .collect()
}

fn unit(id: u32, attribute: Attribute, hp_current: i32) -> Unit {
    Unit {
        id: UnitId(id),
        name: format!("Unit{id}"),
        hp_max: 100,
        hp_current,
        attribute,
        resists: vec![],
        evo_stage: EvoStage::Adult,
    }
}

fn child_unit(id: u32, attribute: Attribute, hp_current: i32) -> Unit {
    Unit {
        id: UnitId(id),
        name: format!("ChildUnit{id}"),
        hp_max: 100,
        hp_current,
        attribute,
        resists: vec![],
        evo_stage: EvoStage::Child,
    }
}

fn skill(
    id: &str,
    damage_tag: DamageTag,
    damage: i32,
    sp_cost: i32,
    toughness_damage: i32,
) -> SkillDef {
    SkillDef {
        id: SkillId(id.into()),
        name: id.into(),
        damage_tag,
        sp_cost,
        targeting: SkillTargeting {
            shape: TargetShape::Single,
            side: TargetSide::Enemy,
            life: TargetLife::Alive,
            self_rule: SelfTargetRule::Forbid,
            ..Default::default()
        },
        implementation: SkillImplementation::Implemented,
        legacy_ops: vec![
            Effect::Damage {
                amount: damage,
                target: TargetShape::Single,
                per_hop: Default::default(),
            },
            Effect::ToughnessHit(toughness_damage),
        ],

        custom_signals: vec![],
        animation_sequence: None,
        qte: None,
        ..Default::default()
    }
}

fn revive_skill(id: &str, pct: i32, sp_cost: i32) -> SkillDef {
    SkillDef {
        id: SkillId(id.into()),
        name: id.into(),
        damage_tag: DamageTag::Light,
        sp_cost,
        targeting: SkillTargeting {
            shape: TargetShape::Single,
            side: TargetSide::Ally,
            life: TargetLife::Ko,
            self_rule: SelfTargetRule::Forbid,
            ..Default::default()
        },
        implementation: SkillImplementation::Implemented,
        legacy_ops: vec![Effect::Revive(pct)],

        custom_signals: vec![],
        animation_sequence: None,
        qte: None,
        ..Default::default()
    }
}

fn resolved(intent: &ActionIntent, skill: SkillDef) -> crate::combat::state::ResolvedAction {
    let book = SkillBook(vec![skill.clone()]);
    let kit = UnitSkills {
        basic: skill.id.clone(),
        skills: vec![skill.id.clone()],
        ultimate: skill.id,
        follow_up: None,
    };
    resolve_action(intent, &kit, Some(&book)).expect("skill should resolve")
}

fn basic_intent() -> ActionIntent {
    ActionIntent::Basic {
        attacker: UnitId(1),
        target: UnitId(2),
    }
}

fn default_ult() -> UltimateCharge {
    UltimateCharge {
        current: 0,
        trigger: 100,
        cap: 150,
        trigger_type: UltAccumulationTrigger::OnBasicAttack,
        charge_per_event: 25,
    }
}

fn snap(entries: Vec<(UnitId, Team, u8, bool)>) -> TargetableSnapshot {
    TargetableSnapshot {
        entries: entries
            .into_iter()
            .map(|(id, team, slot_index, alive)| TargetEntry {
                id,
                team,
                slot_index,
                alive,
                hp_per_mille: 1000, // full HP default for shape tests that don't use HP selector
            })
            .collect(),
    }
}

/// Build a snapshot with explicit HP percentages (per-mille: 0–1000).
fn snap_hp(entries: Vec<(UnitId, Team, u8, bool, u32)>) -> TargetableSnapshot {
    TargetableSnapshot {
        entries: entries
            .into_iter()
            .map(|(id, team, slot_index, alive, hp_per_mille)| TargetEntry {
                id,
                team,
                slot_index,
                alive,
                hp_per_mille,
            })
            .collect(),
    }
}
