use bevy::prelude::*;

use bevyrogue::combat::api::{SignalPayload, intent::CastId};
use bevyrogue::combat::blueprints::twin_core::TwinCoreState;
use bevyrogue::combat::blueprints::patamon::{
    GRACE_CAP, HolySupportDesignTag, HolySupportHook, HolySupportRejectReason, HolySupportState,
    HolySupportStep, HolySupportTransition, classify_holy_support_tag, holy_support_design_tag,
    register_validation_ext,
};
use bevyrogue::combat::events::{CombatEvent, CombatEventKind};
use bevyrogue::combat::api::ExtRegistries;
use bevyrogue::combat::blueprints::patamon::identity::HolySupportSignal;
use bevyrogue::combat::kernel::{
    CombatKernelRegistry, CombatKernelTransition, CombatTagChangeKind, CombatTagState,
    CombatTagTransition, TacticalCyclePhase, TacticalCycleStep, TacticalCycleTransition,
};
use bevyrogue::combat::log::ActionLog;
use bevyrogue::combat::observability::{capture_validation_snapshot, format_validation_snapshot};
use bevyrogue::combat::sp::SpPool;
use bevyrogue::combat::state::CombatState;
use bevyrogue::combat::types::UnitId;

fn app_with_holy_support() -> App {
    let mut app = App::new();
    app.add_message::<CombatEvent>();
    bevyrogue::combat::kernel::register_combat_kernel_runtime(&mut app);
    bevyrogue::combat::blueprints::add_runtime_plugins(&mut app);
    app
}

fn queue_kernel_transition(app: &mut App, transition: CombatKernelTransition) {
    let outputs = {
        let registry = app.world().resource::<CombatKernelRegistry>();
        registry.dispatch(transition)
    };

    for transition in outputs {
        app.world_mut().write_message(CombatEvent {
            kind: CombatEventKind::OnKernelTransition { transition },
            source: UnitId(1),
            target: UnitId(2),
            follow_up_depth: 0,
            cast_id: CastId::ROOT,
        });
    }
}

fn emit_kernel_transition(app: &mut App, transition: CombatKernelTransition) {
    queue_kernel_transition(app, transition);
    app.update();
}

fn emit_holy_blueprint_transition(app: &mut App, transition: HolySupportTransition) {
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
        target: UnitId(2),
        follow_up_depth: 0,
        cast_id: CastId::ROOT,
    });
    app.update();
}

#[test]
fn hook_translates_holy_tags_and_ignores_unknown_tags() {
    let mut registry = CombatKernelRegistry::new();
    registry.register(HolySupportHook);

    assert_eq!(
        classify_holy_support_tag(&holy_support_design_tag(HolySupportDesignTag::Grace)),
        Some(HolySupportDesignTag::Grace)
    );
    assert_eq!(
        classify_holy_support_tag(&holy_support_design_tag(HolySupportDesignTag::MartyrLight)),
        Some(HolySupportDesignTag::MartyrLight)
    );

    let grace = CombatKernelTransition::Tag(CombatTagTransition {
        before: CombatTagState::new(holy_support_design_tag(HolySupportDesignTag::Grace), 3),
        after: CombatTagState::new(holy_support_design_tag(HolySupportDesignTag::Grace), 3),
        kind: CombatTagChangeKind::Added,
    });
    let outputs = registry.dispatch(grace);
    assert_eq!(outputs.len(), 2);
    assert_eq!(
        outputs[1],
        CombatKernelTransition::Blueprint {
            owner: "patamon".to_owned(),
            name: "build_holy_support_grace".to_owned(),
            payload: SignalPayload::Amount(1),
        }
    );

    let martyr = CombatKernelTransition::Tag(CombatTagTransition {
        before: CombatTagState::new(
            holy_support_design_tag(HolySupportDesignTag::MartyrLight),
            2,
        ),
        after: CombatTagState::new(
            holy_support_design_tag(HolySupportDesignTag::MartyrLight),
            2,
        ),
        kind: CombatTagChangeKind::Added,
    });
    let martyr_outputs = registry.dispatch(martyr);
    assert_eq!(
        martyr_outputs[1],
        CombatKernelTransition::Blueprint {
            owner: "patamon".to_owned(),
            name: "mark_martyr_light".to_owned(),
            payload: SignalPayload::Empty,
        }
    );

    let unrelated = CombatKernelTransition::Tag(CombatTagTransition {
        before: CombatTagState::new("Other", 1),
        after: CombatTagState::new("Other", 1),
        kind: CombatTagChangeKind::Added,
    });
    assert_eq!(registry.dispatch(unrelated).len(), 1);
}

#[test]
fn grace_builds_saturate_at_cap_after_bevy_flush() {
    let mut app = app_with_holy_support();

    app.world_mut().write_message(CombatEvent {
        kind: CombatEventKind::OnKernelTransition {
            transition: CombatKernelTransition::Blueprint {
                owner: "patamon".to_owned(),
                name: "build_holy_support_grace".to_owned(),
                payload: SignalPayload::Amount(i64::from(GRACE_CAP.saturating_add(4))),
            },
        },
        source: UnitId(1),
        target: UnitId(2),
        follow_up_depth: 0,
        cast_id: CastId::ROOT,
    });

    {
        let state = app.world().resource::<HolySupportState>();
        assert_eq!(
            state.grace, 0,
            "state should not change before app.update()"
        );
    }

    app.update();

    {
        let state = app.world().resource::<HolySupportState>();
        assert_eq!(state.grace, GRACE_CAP);
        assert_eq!(
            state.last_signal,
            Some(HolySupportTransition::build_grace(
                GRACE_CAP.saturating_add(4)
            ))
        );
    }
}

#[test]
fn spending_grace_underflow_is_rejected_without_underflowing() {
    let mut app = app_with_holy_support();

    emit_holy_blueprint_transition(&mut app, HolySupportTransition::spend_grace(1));

    let state = app.world().resource::<HolySupportState>();
    assert_eq!(state.grace, 0);
    assert_eq!(
        state.last_signal,
        Some(HolySupportTransition::rejected(
            HolySupportStep::SpendGrace { amount: 1 },
            HolySupportRejectReason::GraceUnderflow,
        ))
    );
}

#[test]
fn martyr_light_is_only_marked_and_consumed_once_per_cycle() {
    let mut app = app_with_holy_support();

    emit_holy_blueprint_transition(&mut app, HolySupportTransition::mark_martyr_light());
    {
        let state = app.world().resource::<HolySupportState>();
        assert!(state.martyr_light_marked_this_cycle);
        assert!(!state.martyr_light_consumed_this_cycle);
        assert_eq!(
            state.last_signal,
            Some(HolySupportTransition::mark_martyr_light())
        );
    }

    emit_holy_blueprint_transition(&mut app, HolySupportTransition::mark_martyr_light());
    {
        let state = app.world().resource::<HolySupportState>();
        assert!(state.martyr_light_marked_this_cycle);
        assert_eq!(
            state.last_signal,
            Some(HolySupportTransition::rejected(
                HolySupportStep::MarkMartyrLight,
                HolySupportRejectReason::MartyrAlreadyMarked,
            ))
        );
    }

    emit_holy_blueprint_transition(&mut app, HolySupportTransition::consume_martyr_light());
    {
        let state = app.world().resource::<HolySupportState>();
        assert!(state.martyr_light_marked_this_cycle);
        assert!(state.martyr_light_consumed_this_cycle);
        assert_eq!(
            state.last_signal,
            Some(HolySupportTransition::consume_martyr_light())
        );
    }

    emit_holy_blueprint_transition(&mut app, HolySupportTransition::consume_martyr_light());
    {
        let state = app.world().resource::<HolySupportState>();
        assert!(state.martyr_light_consumed_this_cycle);
        assert_eq!(
            state.last_signal,
            Some(HolySupportTransition::rejected(
                HolySupportStep::ConsumeMartyrLight,
                HolySupportRejectReason::MartyrAlreadyConsumed,
            ))
        );
    }
}

#[test]
fn wrapped_tactical_cycle_resets_holy_support_guards() {
    let mut app = app_with_holy_support();

    emit_holy_blueprint_transition(&mut app, HolySupportTransition::mark_martyr_light());
    emit_holy_blueprint_transition(&mut app, HolySupportTransition::consume_martyr_light());

    let wrapped_cycle = CombatKernelTransition::TacticalCycle(TacticalCycleTransition {
        before: TacticalCycleStep {
            phase: TacticalCyclePhase::Applied,
            step_in_phase: 1,
            cycle_index: 2,
        },
        after: TacticalCycleStep {
            phase: TacticalCyclePhase::Declared,
            step_in_phase: 0,
            cycle_index: 3,
        },
        wrapped_phase: true,
        wrapped_cycle: true,
    });
    emit_kernel_transition(&mut app, wrapped_cycle);

    let state = app.world().resource::<HolySupportState>();
    assert!(!state.martyr_light_marked_this_cycle);
    assert!(!state.martyr_light_consumed_this_cycle);
    assert_eq!(
        state.last_signal,
        Some(HolySupportTransition::cycle_reset())
    );
}

#[test]
fn validation_snapshot_includes_holy_support_fields() {
    let mut world = World::new();
    world.insert_resource(CombatState::default());
    world.insert_resource(SpPool::default());
    world.insert_resource(ActionLog::default());
    world.insert_resource(TwinCoreState::default());
    world.insert_resource(HolySupportState {
        grace: 2,
        martyr_light_marked_this_cycle: true,
        martyr_light_consumed_this_cycle: false,
        last_signal: Some(HolySupportTransition::build_grace(2)),
    });
    world.insert_resource(ExtRegistries::default());
    {
        let mut regs = world.resource_mut::<ExtRegistries>();
        register_validation_ext(&mut regs);
    }

    let snapshot = capture_validation_snapshot(&mut world).expect("snapshot should build");
    let holy_support = snapshot
        .section("support")
        .expect("support section should be present");

    assert_eq!(holy_support.field("grace"), Some("2"));
    assert_eq!(holy_support.field("grace_cap"), Some("3"));
    assert_eq!(holy_support.field("martyr_marked"), Some("true"));
    assert_eq!(holy_support.field("martyr_consumed"), Some("false"));
    assert_eq!(holy_support.field("last"), Some("build(2)"));

    let formatted = format_validation_snapshot(&snapshot);
    assert!(formatted.contains("support=grace=2/3"));
    assert!(formatted.contains("martyr_marked=true"));
    assert!(formatted.contains("last=build(2)"));
    assert!(!formatted.contains("holy_support="));
}
