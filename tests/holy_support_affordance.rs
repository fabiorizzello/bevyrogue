use bevy::prelude::*;
use bevyrogue::combat::runtime::ExtRegistries;
use bevyrogue::combat::runtime::{SignalPayload, intent::CastId};
use bevyrogue::combat::blueprints::twin_core::TwinCoreState;
use bevyrogue::combat::blueprints::patamon::{
    HolySupportState, HolySupportTransition, apply_holy_support_transitions_system,
    register_validation_ext,
};
use bevyrogue::combat::blueprints::patamon::identity::HolySupportSignal;
use bevyrogue::combat::events::{CombatEvent, CombatEventKind};
use bevyrogue::combat::kernel::CombatKernelTransition;
use bevyrogue::combat::log::ActionLog;
use bevyrogue::combat::observability::{capture_validation_snapshot, format_validation_snapshot};
use bevyrogue::combat::sp::SpPool;
use bevyrogue::combat::state::CombatState;
use bevyrogue::combat::types::UnitId;

fn app_with_holy_support() -> App {
    let mut app = App::new();
    app.add_message::<CombatEvent>();
    app.insert_resource(ExtRegistries::default());
    {
        let mut regs = app.world_mut().resource_mut::<ExtRegistries>();
        register_validation_ext(&mut regs);
    }
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
    app.add_message::<CombatEvent>();
    app.insert_resource(ExtRegistries::default());
    {
        let mut regs = app.world_mut().resource_mut::<ExtRegistries>();
        register_validation_ext(&mut regs);
    }
    app.insert_resource(CombatState::default())
        .insert_resource(SpPool::default())
        .insert_resource(ActionLog::default())
        .insert_resource(TwinCoreState::default());
    app
}

fn emit_holy_transition(app: &mut App, transition: HolySupportTransition) {
    let (name, payload) = match transition.signal {
        HolySupportSignal::BuildGrace => (
            "build_holy_support_grace",
            SignalPayload::Amount(i64::from(transition.amount)),
        ),
        HolySupportSignal::SpendGrace => (
            "spend_holy_support_grace",
            SignalPayload::Amount(i64::from(transition.amount)),
        ),
        HolySupportSignal::MarkMartyrLight => ("mark_martyr_light", SignalPayload::Empty),
        HolySupportSignal::ConsumeMartyrLight => ("consume_martyr_light", SignalPayload::Empty),
        HolySupportSignal::CycleReset => ("cycle_reset", SignalPayload::Empty),
        HolySupportSignal::Rejected | HolySupportSignal::Ignored => {
            ("rejected", SignalPayload::Empty)
        }
    };

    app.world_mut().write_message(CombatEvent {
        kind: CombatEventKind::OnKernelTransition {
            transition: CombatKernelTransition::Blueprint {
                owner: "patamon".to_owned(),
                name: name.to_owned(),
                payload,
            },
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

fn holy_support_section<'a>(snapshot: &'a bevyrogue::combat::observability::ValidationSnapshot) -> &'a bevyrogue::combat::runtime::ValidationSection {
    snapshot
        .section("support")
        .expect("support section should be present when HolySupportState is registered")
}

#[test]
fn holy_support_missing_resource_is_absent_from_validation_snapshot() {
    let mut app = app_without_holy_support();

    let snapshot = capture_validation_snapshot(app.world_mut()).expect("snapshot should build");

    assert!(snapshot.section("support").is_none());
    assert!(!format_validation_snapshot(&snapshot).contains("support="));
    assert!(!format_validation_snapshot(&snapshot).contains("holy_support="));
}

#[test]
fn holy_support_transition_system_updates_snapshot_without_affordance_api() {
    let mut app = app_with_holy_support();
    emit_holy_transition(&mut app, HolySupportTransition::build_grace(2));
    emit_holy_transition(&mut app, HolySupportTransition::mark_martyr_light());

    let snapshot = capture_validation_snapshot(app.world_mut()).expect("snapshot should build");
    let holy_support = holy_support_section(&snapshot);

    assert_eq!(holy_support.field("grace"), Some("2"));
    assert_eq!(holy_support.field("grace_cap"), Some("3"));
    assert_eq!(holy_support.field("martyr_marked"), Some("true"));
    assert_eq!(holy_support.field("martyr_consumed"), Some("false"));
    assert_eq!(holy_support.field("last"), Some("mark-martyr"));

    let formatted = format_validation_snapshot(&snapshot);
    assert!(formatted.contains("grace=2"));
    assert!(formatted.contains("martyr_marked=true"));
    assert!(formatted.contains("last=mark-martyr"));
    assert!(!formatted.contains("holy_support="));
}

#[test]
fn holy_support_invalid_spend_is_visible_as_rejected_snapshot_state() {
    let mut app = app_with_holy_support();
    emit_holy_transition(&mut app, HolySupportTransition::spend_grace(1));

    let snapshot = capture_validation_snapshot(app.world_mut()).expect("snapshot should build");
    let holy_support = holy_support_section(&snapshot);

    assert_eq!(holy_support.field("grace"), Some("0"));
    assert_eq!(holy_support.field("grace_cap"), Some("3"));
    assert_eq!(
        holy_support.field("last"),
        Some("rejected(spend(1);reason=GraceUnderflow)")
    );

    let formatted = format_validation_snapshot(&snapshot);
    assert!(formatted.contains("grace=0"));
    assert!(formatted.contains("last=rejected(spend(1);reason=GraceUnderflow)"));
}
