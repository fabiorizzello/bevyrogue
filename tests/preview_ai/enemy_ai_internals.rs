//! Pure-function unit contracts for `pick_enemy_action` that are NOT covered
//! by the full-pipeline integration tests in `tests/enemy_ai.rs`.
//!
//! Integration tests exercise the Bevy `advance_turn_system → pick_enemy_action`
//! path with `UltimateCharge` components and emit `ActionIntent` messages —
//! that covers Ultimate/Skill/lowest-ratio selection.
//!
//! These two cases poke `pick_enemy_action` directly:
//!   * basic-fallback when the unit's `UnitSkills.skills` slice is empty,
//!   * deterministic tie-break by lowest `UnitId` when toughness ratios tie.
//! Both are easier to assert in isolation than via the full pipeline.
use bevyrogue::combat::{
    enemy_ai::{EnemyTurnContext, TargetInfo, pick_enemy_action},
    kit::UnitSkills,
    turn_system::ActionIntent,
    types::{SkillId, UnitId},
};

fn skills_with(ids: &[&str]) -> UnitSkills {
    UnitSkills {
        basic: SkillId("basic".to_string()),
        skills: ids.iter().map(|s| SkillId(s.to_string())).collect(),
        ultimate: SkillId("ult".to_string()),
        follow_up: None,
    }
}

fn target(id: u32, tough_cur: i32, tough_max: i32) -> TargetInfo {
    TargetInfo {
        id: UnitId(id),
        toughness_current: tough_cur,
        toughness_max: tough_max,
        hp_current: 100,
        hp_max: 100,
    }
}

#[test]
fn pick_action_falls_back_to_basic_when_no_skills() {
    let skills = skills_with(&[]);
    let targets = vec![target(3, 10, 10)];
    let ctx = EnemyTurnContext {
        attacker_id: UnitId(10),
        attacker_skills: &skills,
        attacker_ult_ready: false,
        targets: &targets,
    };
    let intent = pick_enemy_action(&ctx).unwrap();
    assert!(
        matches!(
            intent,
            ActionIntent::Basic {
                attacker: UnitId(10),
                target: UnitId(3)
            }
        ),
        "expected Basic, got {:?}",
        intent
    );
}

#[test]
fn pick_action_breaks_tie_by_unit_id() {
    // both at 5/10 = 0.5 → tie broken by lowest UnitId
    let skills = skills_with(&[]);
    let targets = vec![target(5, 5, 10), target(2, 5, 10)];
    let ctx = EnemyTurnContext {
        attacker_id: UnitId(10),
        attacker_skills: &skills,
        attacker_ult_ready: false,
        targets: &targets,
    };
    let intent = pick_enemy_action(&ctx).unwrap();
    assert!(
        matches!(
            intent,
            ActionIntent::Basic {
                target: UnitId(2),
                ..
            }
        ),
        "expected target UnitId(2) (lowest id), got {:?}",
        intent
    );
}
