//! Unit-level contract for `evaluate_follow_up`: the four skip-reason branches
//! (WrongTeam, FollowerKo, FollowerStunned, TriggerMismatch) are not asserted
//! by the broader follow-up integration tests, which only observe the
//! Scheduled / Suppressed decision at the trace level.
//!
//! Relocated from `src/combat/mechanics/follow_up/triggers.rs` per R003.

use bevyrogue::combat::{
    events::{CombatEvent, CombatEventKind},
    kit::{FollowUpConfig, FollowUpTrigger},
    mechanics::follow_up::{FollowUpSkipReason, FollowerSnapshot, evaluate_follow_up},
    runtime::CastId,
    team::Team,
    types::{DamageTag, SkillId, UnitId},
};

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
