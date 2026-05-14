use bevy::prelude::*;
use bevyrogue::combat::api::intent::CastId;
use bevyrogue::combat::events::{CombatEvent, CombatEventKind};
use bevyrogue::combat::blueprints::patamon::{
    HolySupportRejectReason, HolySupportState, HolySupportStep, HolySupportTransition,
    apply_holy_support_transitions_system,
};
use bevyrogue::combat::kernel::CombatKernelTransition;
use bevyrogue::combat::log::ActionLog;
use bevyrogue::combat::observability::{capture_validation_snapshot, format_validation_snapshot};
use bevyrogue::combat::sp::SpPool;
use bevyrogue::combat::state::CombatState;
use bevyrogue::combat::blueprints::agumon::TwinCoreState;
use bevyrogue::combat::types::UnitId;

fn app_with_holy_support() -> App {
    let mut app = App::new();
    app.add_message::<CombatEvent>();
    app.insert_resource(CombatState::default())
        .insert_resource(SpPool::default())
        .insert_resource(ActionLog::default())
        .insert_resource(TwinCoreState::default())
        .insert_resource(HolySupportState::default())
        .add_systems(Update, apply_holy_support_transitions_system);
    app
}

fn app_without_holy_support() -> App {
    let mut app = App::new();
    app.insert_resource(CombatState::default())
        .insert_resource(SpPool::default())
        .insert_resource(ActionLog::default())
        .insert_resource(TwinCoreState::default());
    app
}

fn emit_holy_transition(app: &mut App, transition: HolySupportTransition) {
    app.world_mut().write_message(CombatEvent {
        kind: CombatEventKind::OnKernelTransition {
            transition: CombatKernelTransition::HolySupport(transition),
        },
        source: UnitId(1),
        target: UnitId(1),
        follow_up_depth: 0,
        cast_id: CastId::ROOT,
    });
    app.update();
}

#[test]
fn holy_support_snapshot_surfaces_grace_capacity_and_martyr_state() {
    let state = HolySupportState {
        grace: 3,
        martyr_light_marked_this_cycle: true,
        martyr_light_consumed_this_cycle: false,
        last_signal: Some(HolySupportTransition::build_grace(3)),
    };

    let snapshot = state.snapshot();

    assert_eq!(snapshot.grace, 3);
    assert_eq!(snapshot.grace_cap, 3);
    assert!(snapshot.martyr_light_marked_this_cycle);
    assert!(!snapshot.martyr_light_consumed_this_cycle);
    assert_eq!(
        snapshot.last_signal,
        Some(HolySupportTransition::build_grace(3))
    );
}

#[test]
fn holy_support_missing_resource_is_absent_from_validation_snapshot() {
    let mut app = app_without_holy_support();

    let snapshot = capture_validation_snapshot(app.world_mut()).expect("snapshot should build");

    assert!(snapshot.holy_support.is_none());
    assert!(format_validation_snapshot(&snapshot).contains("holy_support=none"));
}

#[test]
fn holy_support_transition_system_updates_snapshot_without_affordance_api() {
    let mut app = app_with_holy_support();
    emit_holy_transition(&mut app, HolySupportTransition::build_grace(2));
    emit_holy_transition(&mut app, HolySupportTransition::mark_martyr_light());

    let snapshot = capture_validation_snapshot(app.world_mut()).expect("snapshot should build");
    let holy_support = snapshot
        .holy_support
        .as_ref()
        .expect("holy support state should be captured");

    assert_eq!(holy_support.grace, 2);
    assert!(holy_support.martyr_light_marked_this_cycle);
    assert!(!holy_support.martyr_light_consumed_this_cycle);
    assert_eq!(
        holy_support.last_signal,
        Some(HolySupportTransition::mark_martyr_light())
    );

    let formatted = format_validation_snapshot(&snapshot);
    assert!(formatted.contains("holy_support=grace=2/3"));
    assert!(formatted.contains("martyr_marked=true"));
    assert!(formatted.contains("last=mark-martyr"));
}

#[test]
fn holy_support_invalid_spend_is_visible_as_rejected_snapshot_state() {
    let mut app = app_with_holy_support();
    emit_holy_transition(&mut app, HolySupportTransition::spend_grace(1));

    let snapshot = capture_validation_snapshot(app.world_mut()).expect("snapshot should build");
    let holy_support = snapshot
        .holy_support
        .as_ref()
        .expect("holy support state should be captured");

    assert_eq!(holy_support.grace, 0);
    assert_eq!(
        holy_support.last_signal,
        Some(HolySupportTransition::rejected(
            HolySupportStep::SpendGrace { amount: 1 },
            HolySupportRejectReason::GraceUnderflow,
        ))
    );

    let formatted = format_validation_snapshot(&snapshot);
    assert!(formatted.contains("holy_support=grace=0/3"));
    assert!(formatted.contains("last=rejected(spend(1);reason=GraceUnderflow)"));
}
