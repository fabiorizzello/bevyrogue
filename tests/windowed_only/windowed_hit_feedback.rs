#![cfg(feature = "windowed")]

use bevy::prelude::*;
use bevyrogue::combat::{
    events::{CombatEvent, CombatEventKind},
    runtime::intent::CastId,
    state::CombatState,
    types::UnitId,
};
use bevyrogue::ui::hit_feedback::{
    FLASH_TICKS, HitFlashState, HitShakeState, SHAKE_TICKS, damage_number_kinematics, flash_tint,
    hit_damage_amount, observe_hit_feedback, shake_offset,
};

const ATTACKER: UnitId = UnitId(1);
const DEFENDER: UnitId = UnitId(2);

fn build_app() -> App {
    let mut app = App::new();
    app.add_message::<CombatEvent>()
        .init_resource::<HitFlashState>()
        .init_resource::<HitShakeState>()
        .init_resource::<CombatState>()
        .add_systems(Update, observe_hit_feedback);
    app
}

fn write_on_hit_taken(app: &mut App, target: UnitId, amount: i32) {
    app.world_mut().write_message(CombatEvent {
        source: ATTACKER,
        target,
        kind: CombatEventKind::OnHitTaken { amount },
        follow_up_depth: 0,
        cast_id: CastId::ROOT,
    });
}

fn write_event(app: &mut App, target: UnitId, kind: CombatEventKind) {
    app.world_mut().write_message(CombatEvent {
        source: ATTACKER,
        target,
        kind,
        follow_up_depth: 0,
        cast_id: CastId::ROOT,
    });
}

// (a) OnHitTaken arms both states for the target to full.
#[test]
fn on_hit_taken_arms_flash_and_shake_to_full() {
    let mut app = build_app();
    write_on_hit_taken(&mut app, DEFENDER, 15);
    app.update();

    let flash = app.world().resource::<HitFlashState>();
    let shake = app.world().resource::<HitShakeState>();
    assert_eq!(flash.remaining(DEFENDER), FLASH_TICKS);
    assert_eq!(shake.remaining(DEFENDER), SHAKE_TICKS);
}

// (b) decay_by drains to 0 with NO underflow when called past the budget.
#[test]
fn decay_by_past_budget_does_not_underflow_and_clears() {
    let mut flash = HitFlashState::default();
    flash.arm(DEFENDER);
    // Decay by more than the full window — must clamp to zero and remove.
    flash.decay_by(FLASH_TICKS + 5);
    assert_eq!(flash.remaining(DEFENDER), 0);
    assert!(!flash.remaining.contains_key(&DEFENDER));

    let mut shake = HitShakeState::default();
    shake.arm(DEFENDER);
    shake.decay_by(SHAKE_TICKS + 100);
    assert_eq!(shake.remaining(DEFENDER), 0);
    assert!(!shake.remaining.contains_key(&DEFENDER));
}

// decay_by below the budget leaves a partial countdown.
#[test]
fn decay_by_partial_leaves_remainder() {
    let mut flash = HitFlashState::default();
    flash.arm(DEFENDER);
    flash.decay_by(2);
    assert_eq!(flash.remaining(DEFENDER), FLASH_TICKS - 2);
}

// (c) Two OnHitTaken for the same target in one update arm once to full (dedup).
#[test]
fn repeated_hits_same_update_dedup_to_full() {
    let mut app = build_app();
    write_on_hit_taken(&mut app, DEFENDER, 10);
    write_on_hit_taken(&mut app, DEFENDER, 20);
    app.update();

    let flash = app.world().resource::<HitFlashState>();
    let shake = app.world().resource::<HitShakeState>();
    assert_eq!(flash.remaining(DEFENDER), FLASH_TICKS);
    assert_eq!(shake.remaining(DEFENDER), SHAKE_TICKS);
    assert_eq!(flash.remaining.len(), 1);
    assert_eq!(shake.remaining.len(), 1);
}

// (d) A non-hit event does NOT arm any state; hit_damage_amount returns None
// for it and Some for OnHitTaken.
#[test]
fn non_hit_event_does_not_arm_and_amount_is_none() {
    let mut app = build_app();
    write_event(&mut app, DEFENDER, CombatEventKind::OnRevive { hp_after: 30 });
    app.update();

    let flash = app.world().resource::<HitFlashState>();
    let shake = app.world().resource::<HitShakeState>();
    assert_eq!(flash.remaining(DEFENDER), 0);
    assert_eq!(shake.remaining(DEFENDER), 0);

    assert_eq!(
        hit_damage_amount(&CombatEventKind::OnRevive { hp_after: 30 }),
        None
    );
    assert_eq!(
        hit_damage_amount(&CombatEventKind::OnDamageDealt {
            amount: 12,
            kind: bevyrogue::combat::toughness::DamageKind::Normal,
            tag_mod_pct: 0,
            triangle_mod_pct: 0,
            damage_tag: bevyrogue::combat::types::DamageTag::Physical,
        }),
        None
    );
    assert_eq!(
        hit_damage_amount(&CombatEventKind::OnHitTaken { amount: 7 }),
        Some(7)
    );
}

// (e) damage_number_kinematics: age 0 → alpha≈1.0, rise≈0; age==total →
// alpha≈0, rise>0; monotonic in both directions.
#[test]
fn damage_number_kinematics_endpoints_and_monotonic() {
    const TOTAL: u32 = 12;
    let (rise0, alpha0) = damage_number_kinematics(0, TOTAL);
    assert!(rise0.abs() < 1e-6, "rise at age 0 should be ~0, got {rise0}");
    assert!(
        (alpha0 - 1.0).abs() < 1e-6,
        "alpha at age 0 should be ~1.0, got {alpha0}"
    );

    let (rise_end, alpha_end) = damage_number_kinematics(TOTAL, TOTAL);
    assert!(rise_end > 0.0, "rise at age==total should be > 0");
    assert!(
        alpha_end.abs() < 1e-6,
        "alpha at age==total should be ~0, got {alpha_end}"
    );

    let mut prev_rise = -1.0;
    let mut prev_alpha = f32::INFINITY;
    for age in 0..=TOTAL {
        let (rise, alpha) = damage_number_kinematics(age, TOTAL);
        assert!(rise >= prev_rise, "rise must be monotonic non-decreasing");
        assert!(alpha <= prev_alpha, "alpha must be monotonic non-increasing");
        prev_rise = rise;
        prev_alpha = alpha;
    }
}

// (f) flash_tint(total,total) != WHITE; flash_tint(0,total) == WHITE.
#[test]
fn flash_tint_endpoints() {
    const TOTAL: u32 = FLASH_TICKS;
    assert_ne!(
        flash_tint(TOTAL, TOTAL),
        Color::WHITE,
        "peak flash must differ from WHITE"
    );
    assert_eq!(
        flash_tint(0, TOTAL),
        Color::WHITE,
        "no flash must be exactly WHITE"
    );
}

// (g) shake_offset(0,total) == Vec2::ZERO.
#[test]
fn shake_offset_zero_when_not_shaking() {
    assert_eq!(shake_offset(0, SHAKE_TICKS), Vec2::ZERO);
    // And it produces a non-zero displacement at peak.
    assert_ne!(shake_offset(SHAKE_TICKS, SHAKE_TICKS), Vec2::ZERO);
}

// R010: feedback projection never mutates CombatState.
#[test]
fn no_combat_state_mutation_from_feedback_projection() {
    let mut app = build_app();
    let before = app.world().resource::<CombatState>().clone();
    write_on_hit_taken(&mut app, DEFENDER, 42);
    app.update();
    let after = app.world().resource::<CombatState>().clone();
    assert_eq!(before, after, "hit feedback must never mutate CombatState");
}
