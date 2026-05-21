#![cfg(feature = "windowed")]
//! Twin Core synergy badge: windowed-only projection of `twin_core`
//! `OnKernelTransition::Blueprint` signals into a frame-countdown chip.

use bevy::prelude::*;
use bevyrogue::combat::{
    events::{CombatEvent, CombatEventKind},
    kernel::CombatKernelTransition,
    runtime::{SignalPayload, intent::CastId},
    state::CombatState,
    types::UnitId,
};
use bevyrogue::ui::combat_panel::{
    TWIN_CORE_BADGE_FRAMES, TwinCoreBadgeState, observe_twin_core_badge, tick_twin_core_badge,
    twin_core_badge_chip,
};

const ACTOR: UnitId = UnitId(1);
const TARGET: UnitId = UnitId(2);

fn build_app() -> App {
    let mut app = App::new();
    app.add_message::<CombatEvent>()
        .init_resource::<TwinCoreBadgeState>()
        .init_resource::<CombatState>()
        .add_systems(Update, (observe_twin_core_badge, tick_twin_core_badge).chain());
    app
}

fn twin_core_event(name: &str) -> CombatEvent {
    CombatEvent {
        source: ACTOR,
        target: TARGET,
        kind: CombatEventKind::OnKernelTransition {
            transition: CombatKernelTransition::Blueprint {
                owner: "twin_core".to_string(),
                name: name.to_string(),
                payload: SignalPayload::Amount(1),
            },
        },
        follow_up_depth: 0,
        cast_id: CastId::ROOT,
    }
}

fn unrelated_blueprint_event() -> CombatEvent {
    CombatEvent {
        source: ACTOR,
        target: TARGET,
        kind: CombatEventKind::OnKernelTransition {
            transition: CombatKernelTransition::Blueprint {
                owner: "agumon".to_string(),
                name: "detonate".to_string(),
                payload: SignalPayload::UnitTarget(TARGET),
            },
        },
        follow_up_depth: 0,
        cast_id: CastId::ROOT,
    }
}

#[test]
fn signal_primes_badge_with_lifetime_minus_tick() {
    let mut app = build_app();
    app.world_mut().write_message(twin_core_event("twin_burst"));
    app.update();

    let state = app.world().resource::<TwinCoreBadgeState>().clone();
    assert_eq!(state.primed_for_frames, TWIN_CORE_BADGE_FRAMES - 1);
    assert_eq!(state.last_signal_name.as_deref(), Some("twin_burst"));
    assert!(state.is_primed());
    let chip = twin_core_badge_chip(&state).expect("chip primed");
    assert!(chip.label.contains("Twin Core"));
    assert!(chip.tooltip.contains("signal=twin_burst"));
}

#[test]
fn unrelated_blueprint_signals_do_not_prime() {
    let mut app = build_app();
    app.world_mut().write_message(unrelated_blueprint_event());
    app.update();

    let state = app.world().resource::<TwinCoreBadgeState>().clone();
    assert_eq!(state.primed_for_frames, 0);
    assert!(state.last_signal_name.is_none());
    assert!(twin_core_badge_chip(&state).is_none());
}

#[test]
fn countdown_decrements_each_frame_and_clears_at_zero() {
    let mut app = build_app();
    app.world_mut().write_message(twin_core_event("build_cross_resonance"));
    app.update();
    // After first update: primed = LIFETIME - 1.
    for _ in 0..(TWIN_CORE_BADGE_FRAMES - 1) {
        app.update();
    }
    let state = app.world().resource::<TwinCoreBadgeState>().clone();
    assert_eq!(state.primed_for_frames, 0);
    assert!(state.last_signal_name.is_none());
    assert!(twin_core_badge_chip(&state).is_none());
}

#[test]
fn multiple_signals_in_one_ultimate_only_prime_once() {
    let mut app = build_app();
    // Simulate an Ultimate fanning out three Twin Core signals in the same frame.
    app.world_mut().write_message(twin_core_event("build_cross_resonance"));
    app.world_mut().write_message(twin_core_event("thermal_spark"));
    app.world_mut().write_message(twin_core_event("twin_burst"));
    app.update();

    let state = app.world().resource::<TwinCoreBadgeState>().clone();
    // First signal primed; subsequent same-frame signals must not re-prime.
    assert_eq!(state.primed_for_frames, TWIN_CORE_BADGE_FRAMES - 1);
    assert_eq!(
        state.last_signal_name.as_deref(),
        Some("build_cross_resonance"),
        "first signal's name latched, later signals ignored while primed"
    );
}

#[test]
fn signals_while_primed_do_not_extend_lifetime() {
    let mut app = build_app();
    app.world_mut().write_message(twin_core_event("twin_burst"));
    app.update();
    // Tick a few frames, then fire another signal — must not refresh.
    for _ in 0..5 {
        app.update();
    }
    let mid = app.world().resource::<TwinCoreBadgeState>().primed_for_frames;
    app.world_mut().write_message(twin_core_event("shatter"));
    app.update();
    let after = app.world().resource::<TwinCoreBadgeState>().primed_for_frames;
    assert_eq!(
        after,
        mid - 1,
        "re-prime while active must be a no-op; only the tick decrements"
    );
}

#[test]
fn no_combat_state_mutation_from_badge_projection() {
    let mut app = build_app();
    let before = app.world().resource::<CombatState>().clone();
    app.world_mut().write_message(twin_core_event("twin_burst"));
    app.update();
    let after = app.world().resource::<CombatState>().clone();
    assert_eq!(before, after, "TwinCoreBadgeState must never mutate CombatState");
}
