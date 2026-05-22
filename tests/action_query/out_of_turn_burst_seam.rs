//! T1 seam: `mark_unit_active` lifts ONLY the active-unit gate for an
//! out-of-turn ult burst. Every other legality check (gauge readiness, SP) must
//! still bite. Pure snapshot-level — no ECS. See small-feature 260522-1.

use bevyrogue::combat::action_query::{
    ActionQueryKind, ActionStatus, CombatQuerySnapshot, UnitQuerySnapshot, mark_unit_active,
    query_action_affordance,
};
use bevyrogue::combat::kit::UnitSkills;
use bevyrogue::combat::state::CombatPhase;
use bevyrogue::combat::team::Team;
use bevyrogue::combat::types::{DamageTag, SkillId, UnitId};
use bevyrogue::data::skills_ron::{
    Effect, LegalityReasonCode, SelfTargetRule, SkillBook, SkillDef, SkillImplementation,
    SkillTargeting, TargetHpRule, TargetLife, TargetShape, TargetSide,
};

const BURST: u32 = 1;
const ENEMY: u32 = 2;
const ULT_ID: &str = "burst_ult";
const ULT_SP_COST: i32 = 20;

/// A non-active ally with the given ult/sp state, plus an enemy to target.
/// `is_active` is false on the burst unit — it is acting out of turn.
fn snapshot(ult_ready: bool, sp: i32) -> CombatQuerySnapshot {
    let burst = UnitQuerySnapshot {
        id: UnitId(BURST),
        team: Team::Ally,
        is_active: false,
        sp,
        ultimate_current: if ult_ready { 100 } else { 0 },
        ultimate_trigger: 100,
        ultimate_ready: ult_ready,
        skills: Some(UnitSkills {
            basic: SkillId("basic".into()),
            skills: vec![],
            ultimate: SkillId(ULT_ID.into()),
            follow_up: None,
        }),
        ..Default::default()
    };
    let enemy = UnitQuerySnapshot {
        id: UnitId(ENEMY),
        team: Team::Enemy,
        is_active: false,
        hp_current: 100,
        hp_max: 100,
        ..Default::default()
    };

    CombatQuerySnapshot {
        phase: CombatPhase::WaitingAction,
        acting_unit: burst.clone(),
        target_unit: Some(enemy.clone()),
        units: vec![burst, enemy],
    }
}

fn ult_skill_book() -> SkillBook {
    SkillBook(vec![SkillDef {
        id: SkillId(ULT_ID.into()),
        name: "Burst Ultimate".into(),
        damage_tag: DamageTag::Fire,
        sp_cost: ULT_SP_COST,
        targeting: SkillTargeting {
            shape: TargetShape::Single,
            side: TargetSide::Enemy,
            life: TargetLife::Alive,
            self_rule: SelfTargetRule::Forbid,
            target_hp_rule: TargetHpRule::Any,
        },
        implementation: SkillImplementation::Implemented,
        legacy_ops: vec![Effect::Damage {
            amount: 50,
            target: TargetShape::Single,
            per_hop: Default::default(),
        }],
        custom_signals: vec![],
        animation_sequence: None,
        qte: None,
        timeline: None,
    }])
}

#[test]
fn off_turn_ult_rejected_as_not_active_without_seam() {
    let snap = snapshot(true, ULT_SP_COST);
    let aff = query_action_affordance(&snap, &ult_skill_book(), UnitId(BURST), ActionQueryKind::Ultimate);
    assert_eq!(
        aff.action,
        ActionStatus::Disabled {
            reason: LegalityReasonCode::NotActiveUnit
        },
        "a non-active unit must be rejected before the seam is applied"
    );
}

#[test]
fn seam_enables_ready_off_turn_ult() {
    let mut snap = snapshot(true, ULT_SP_COST);
    mark_unit_active(&mut snap, UnitId(BURST));
    let aff = query_action_affordance(&snap, &ult_skill_book(), UnitId(BURST), ActionQueryKind::Ultimate);
    assert_eq!(
        aff.action,
        ActionStatus::Enabled,
        "seam must enable a ready, funded off-turn ult"
    );
}

#[test]
fn seam_still_blocks_when_gauge_not_ready() {
    let mut snap = snapshot(false, ULT_SP_COST);
    mark_unit_active(&mut snap, UnitId(BURST));
    let aff = query_action_affordance(&snap, &ult_skill_book(), UnitId(BURST), ActionQueryKind::Ultimate);
    assert_eq!(
        aff.action,
        ActionStatus::Disabled {
            reason: LegalityReasonCode::UltimateNotReady
        },
        "the gauge-ready check must still bite after the seam"
    );
}

#[test]
fn seam_still_blocks_on_sp_shortfall() {
    let mut snap = snapshot(true, ULT_SP_COST - 1);
    mark_unit_active(&mut snap, UnitId(BURST));
    let aff = query_action_affordance(&snap, &ult_skill_book(), UnitId(BURST), ActionQueryKind::Ultimate);
    assert_eq!(
        aff.action,
        ActionStatus::Disabled {
            reason: LegalityReasonCode::SpShortfall
        },
        "the SP-cost check must still bite after the seam"
    );
}
