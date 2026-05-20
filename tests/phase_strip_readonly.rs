#![cfg(feature = "windowed")]

use bevy::{ecs::system::assert_is_read_only_system, prelude::*};
use bevyrogue::combat::{
    events::{ActionIntentKind, CombatEvent, CombatEventKind},
    kernel::CombatBeatId,
    runtime::intent::CastId,
    state::CombatState,
    types::UnitId,
};
use bevyrogue::ui::phase_strip::{
    PhaseStripDisplay, PhaseStripPhase, observe_combat_beats, read_latest_observed_combat_beat,
};

fn beat_event(beat: CombatBeatId) -> CombatEvent {
    CombatEvent {
        kind: CombatEventKind::OnCombatBeat { beat },
        source: UnitId(1),
        target: UnitId(2),
        follow_up_depth: 0,
        cast_id: CastId::ROOT,
    }
}

fn non_beat_event() -> CombatEvent {
    CombatEvent {
        kind: CombatEventKind::OnActionDeclared {
            intent_kind: ActionIntentKind::Basic,
        },
        source: UnitId(1),
        target: UnitId(2),
        follow_up_depth: 0,
        cast_id: CastId::ROOT,
    }
}

fn phase_strip_app() -> App {
    let mut app = App::new();
    app.add_message::<CombatEvent>()
        .init_resource::<CombatState>()
        .init_resource::<PhaseStripDisplay>()
        .add_systems(Update, observe_combat_beats);
    app
}

#[test]
fn combat_event_reader_seam_is_read_only() {
    assert_is_read_only_system(read_latest_observed_combat_beat);
}

#[test]
fn phase_strip_projects_latest_beat_without_mutating_combat_state() {
    let mut app = phase_strip_app();
    let before_state = app.world().resource::<CombatState>().clone();

    app.world_mut()
        .write_message(beat_event(CombatBeatId::Declared));
    app.world_mut()
        .write_message(beat_event(CombatBeatId::PreApp));
    app.update();

    let display = app.world().resource::<PhaseStripDisplay>();
    assert_eq!(display.current_beat, Some(CombatBeatId::PreApp));
    assert_eq!(display.active_phase(), Some(PhaseStripPhase::PreApp));
    assert_eq!(display.active_label(), Some("Pre-App"));
    assert_eq!(*app.world().resource::<CombatState>(), before_state);

    app.world_mut()
        .write_message(beat_event(CombatBeatId::Resolved));
    app.update();

    let display = app.world().resource::<PhaseStripDisplay>();
    assert_eq!(display.current_beat, Some(CombatBeatId::Resolved));
    assert_eq!(display.active_phase(), Some(PhaseStripPhase::Resolved));
    assert_eq!(display.active_label(), Some("Resolved"));
    assert_eq!(*app.world().resource::<CombatState>(), before_state);
}

#[test]
fn phase_strip_ignores_non_beat_events_and_empty_updates() {
    let mut app = phase_strip_app();
    let before_state = app.world().resource::<CombatState>().clone();

    app.update();
    assert_eq!(app.world().resource::<PhaseStripDisplay>().current_beat, None);
    assert_eq!(*app.world().resource::<CombatState>(), before_state);

    {
        let mut display = app.world_mut().resource_mut::<PhaseStripDisplay>();
        display.observe(CombatBeatId::Impact);
    }

    app.world_mut().write_message(non_beat_event());
    app.update();

    let display = app.world().resource::<PhaseStripDisplay>();
    assert_eq!(display.current_beat, Some(CombatBeatId::Impact));
    assert_eq!(display.active_phase(), Some(PhaseStripPhase::Impact));
    assert_eq!(display.active_label(), Some("Impact"));
    assert_eq!(*app.world().resource::<CombatState>(), before_state);
}
