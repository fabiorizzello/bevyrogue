#![cfg(feature = "windowed")]

use bevy::prelude::*;
use bevyrogue::combat::{
    events::{CombatEvent, CombatEventKind},
    runtime::intent::CastId,
    state::CombatState,
    types::UnitId,
};
use bevyrogue::ui::combat_panel::{
    HURT_FRAMES, TargetHurtState, observe_target_hurt, tick_target_hurt_state,
};

const ATTACKER: UnitId = UnitId(1);
const DEFENDER: UnitId = UnitId(2);

fn build_app() -> App {
    let mut app = App::new();
    app.add_message::<CombatEvent>()
        .init_resource::<TargetHurtState>()
        .init_resource::<CombatState>()
        .add_systems(
            Update,
            (observe_target_hurt, tick_target_hurt_state).chain(),
        );
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

#[test]
fn on_hit_taken_seeds_entry_with_hurt_frames() {
    let mut app = build_app();
    write_on_hit_taken(&mut app, DEFENDER, 15);
    app.update();

    let state = app.world().resource::<TargetHurtState>().clone();
    // After one update: observe fires (sets HURT_FRAMES), then tick decrements by 1.
    let expected = HURT_FRAMES - 1;
    assert_eq!(
        state.entries.get(&DEFENDER).copied(),
        Some(expected),
        "entry should be HURT_FRAMES-1 after first update (tick ran after observe)"
    );
    assert!(state.is_hurt(DEFENDER));
}

#[test]
fn repeated_hits_same_frame_do_not_underflow_and_collapse_to_max_countdown() {
    let mut app = build_app();
    // Write two hits on the same target before the update.
    write_on_hit_taken(&mut app, DEFENDER, 10);
    write_on_hit_taken(&mut app, DEFENDER, 20);
    app.update();

    let state = app.world().resource::<TargetHurtState>().clone();
    // Both hits observed → entry set to HURT_FRAMES (idempotent max), then tick -1.
    let expected = HURT_FRAMES - 1;
    assert_eq!(
        state.entries.get(&DEFENDER).copied(),
        Some(expected),
        "repeated same-frame hits must collapse to one max-countdown entry"
    );
}

#[test]
fn countdown_decrements_each_frame_and_clears_at_zero() {
    let mut app = build_app();
    write_on_hit_taken(&mut app, DEFENDER, 8);
    app.update();

    // Run HURT_FRAMES - 1 more updates: after the first update the count was
    // already at HURT_FRAMES-1 (one tick happened), so we need HURT_FRAMES-1
    // more ticks to reach 0.
    for _ in 0..(HURT_FRAMES - 1) {
        app.update();
    }

    let state = app.world().resource::<TargetHurtState>().clone();
    assert!(
        !state.entries.contains_key(&DEFENDER),
        "entry must be cleared once the countdown reaches zero"
    );
    assert!(!state.is_hurt(DEFENDER));
}

#[test]
fn no_combat_state_mutation_from_hurt_projection() {
    let mut app = build_app();
    let before = app.world().resource::<CombatState>().clone();
    write_on_hit_taken(&mut app, DEFENDER, 42);
    app.update();

    let after = app.world().resource::<CombatState>().clone();
    assert_eq!(
        before, after,
        "TargetHurtState must never mutate CombatState"
    );
}
