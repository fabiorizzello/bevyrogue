//! Unit-level coverage for `UltimateCharge::try_add` return-value semantics and
//! the `matches_trigger` pure predicate. Clamp invariants are covered by
//! `tests/properties.rs::ultimate_charge_try_add_clamps_in_range`.
//!
//! Relocated from `src/combat/mechanics/ultimate.rs` per R003.

use bevyrogue::combat::events::{CombatEvent, CombatEventKind};
use bevyrogue::combat::runtime::intent::CastId;
use bevyrogue::combat::team::Team;
use bevyrogue::combat::toughness::DamageKind;
use bevyrogue::combat::types::{DamageTag, SkillId, UnitId};
use bevyrogue::combat::ultimate::{UltAccumulationTrigger, UltimateCharge, matches_trigger};
use rstest::rstest;

// ── try_add return-value semantics ──────────────────────────────────────────

/// `try_add` returns true exactly when this call crosses the trigger threshold
/// from below. Already-ready charges return false; mere increases that don't
/// cross return false. Covers what the properties.rs clamp-invariant does not:
/// the boolean "newly ready" signal that drives ult-fire eligibility.
#[rstest]
#[case::crosses_from_below(0, 100, 150, 100, true)]
#[case::already_ready_no_new_cross(100, 100, 150, 10, false)]
#[case::below_and_stays_below(0, 100, 150, 10, false)]
fn try_add_crossed_flag(
    #[case] start: i32,
    #[case] trigger: i32,
    #[case] cap: i32,
    #[case] amount: i32,
    #[case] expected: bool,
) {
    let mut uc = UltimateCharge {
        current: start,
        trigger,
        cap,
        trigger_type: UltAccumulationTrigger::OnBasicAttack,
        charge_per_event: 25,
    };
    let crossed = uc.try_add(amount);
    assert_eq!(crossed, expected);
    if expected {
        assert!(uc.ready());
    }
}

/// Delta = new_current - old_current. UltGain events emit this delta; it must
/// equal the actual clamped change, never the requested amount.
#[rstest]
#[case::full_increase(0, 100, 150, 10, 10)]
#[case::clamps_at_cap(145, 100, 150, 10, 5)]
#[case::zero_when_at_cap(150, 100, 150, 10, 0)]
fn ult_gain_delta_matches_actual_increase(
    #[case] start: i32,
    #[case] trigger: i32,
    #[case] cap: i32,
    #[case] amount: i32,
    #[case] expected_delta: i32,
) {
    let mut uc = UltimateCharge {
        current: start,
        trigger,
        cap,
        trigger_type: UltAccumulationTrigger::OnBasicAttack,
        charge_per_event: 25,
    };
    let before = uc.current;
    uc.try_add(amount);
    assert_eq!(uc.current - before, expected_delta);
}

// ── matches_trigger dispatch ────────────────────────────────────────────────

fn damage_event(source: UnitId, target: UnitId) -> CombatEvent {
    CombatEvent {
        kind: CombatEventKind::OnDamageDealt {
            amount: 50,
            kind: DamageKind::Normal,
            tag_mod_pct: 100,
            triangle_mod_pct: 100,
            damage_tag: DamageTag::Fire,
        },
        source,
        target,
        follow_up_depth: 0,
        cast_id: CastId::ROOT,
    }
}

fn hit_taken_event(source: UnitId, target: UnitId, amount: i32) -> CombatEvent {
    CombatEvent {
        kind: CombatEventKind::OnHitTaken { amount },
        source,
        target,
        follow_up_depth: 0,
        cast_id: CastId::ROOT,
    }
}

fn skill_cast_event(source: UnitId, depth: u8) -> CombatEvent {
    CombatEvent {
        kind: CombatEventKind::OnSkillCast {
            skill_id: SkillId("s".into()),
        },
        source,
        target: source,
        follow_up_depth: depth,
        cast_id: CastId::ROOT,
    }
}

fn kill_event(source: UnitId, target: UnitId) -> CombatEvent {
    CombatEvent {
        kind: CombatEventKind::OnEnemyKill,
        source,
        target,
        follow_up_depth: 0,
        cast_id: CastId::ROOT,
    }
}

#[test]
fn trigger_on_basic_attack_never_matches() {
    let event = damage_event(UnitId(1), UnitId(2));
    assert!(!matches_trigger(
        &event,
        UnitId(1),
        Team::Ally,
        UltAccumulationTrigger::OnBasicAttack,
        Some(Team::Ally)
    ));
}

#[test]
fn trigger_on_hit_taken_matches_correct_target() {
    let event = hit_taken_event(UnitId(1), UnitId(2), 30);
    assert!(matches_trigger(
        &event,
        UnitId(2),
        Team::Ally,
        UltAccumulationTrigger::OnHitTaken,
        Some(Team::Ally)
    ));
    assert!(!matches_trigger(
        &event,
        UnitId(99),
        Team::Ally,
        UltAccumulationTrigger::OnHitTaken,
        Some(Team::Ally)
    ));
}

#[test]
fn trigger_on_ally_follow_up_matches_depth_and_team() {
    let event = skill_cast_event(UnitId(1), 1);
    assert!(matches_trigger(
        &event,
        UnitId(5),
        Team::Ally,
        UltAccumulationTrigger::OnAllyFollowUp,
        Some(Team::Ally)
    ));
    let root_event = skill_cast_event(UnitId(1), 0);
    assert!(!matches_trigger(
        &root_event,
        UnitId(5),
        Team::Ally,
        UltAccumulationTrigger::OnAllyFollowUp,
        Some(Team::Ally)
    ));
}

#[test]
fn trigger_on_kill_matches_source_unit_only() {
    let event = kill_event(UnitId(3), UnitId(7));
    assert!(matches_trigger(
        &event,
        UnitId(3),
        Team::Ally,
        UltAccumulationTrigger::OnKill,
        Some(Team::Ally)
    ));
    assert!(!matches_trigger(
        &event,
        UnitId(7),
        Team::Ally,
        UltAccumulationTrigger::OnKill,
        Some(Team::Ally)
    ));
}

#[test]
fn trigger_on_offensive_party_event_matches_ally_source_only() {
    let event = damage_event(UnitId(1), UnitId(5));
    assert!(matches_trigger(
        &event,
        UnitId(99),
        Team::Ally,
        UltAccumulationTrigger::OnOffensivePartyEvent,
        Some(Team::Ally)
    ));
    assert!(!matches_trigger(
        &event,
        UnitId(99),
        Team::Ally,
        UltAccumulationTrigger::OnOffensivePartyEvent,
        Some(Team::Enemy)
    ));
}

/// Q7 negative test: ally-vs-ally damage — Taichi charges on OnDamageDealt, not
/// on OnHitTaken; Hackmon charges on OnHitTaken when it is the target.
#[test]
fn ally_vs_ally_semantics_taichi_and_hackmon() {
    let attacker = UnitId(1);
    let defender = UnitId(2);

    let damage_evt = damage_event(attacker, defender);
    let hit_evt = hit_taken_event(attacker, defender, 30);

    assert!(matches_trigger(
        &damage_evt,
        UnitId(99),
        Team::Ally,
        UltAccumulationTrigger::OnOffensivePartyEvent,
        Some(Team::Ally)
    ));
    assert!(!matches_trigger(
        &hit_evt,
        UnitId(99),
        Team::Ally,
        UltAccumulationTrigger::OnOffensivePartyEvent,
        Some(Team::Ally)
    ));
    assert!(matches_trigger(
        &hit_evt,
        defender,
        Team::Ally,
        UltAccumulationTrigger::OnHitTaken,
        Some(Team::Ally)
    ));
    assert!(!matches_trigger(
        &hit_evt,
        UnitId(99),
        Team::Ally,
        UltAccumulationTrigger::OnHitTaken,
        Some(Team::Ally)
    ));
}
