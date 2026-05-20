#[cfg(feature = "windowed")]
use bevy::prelude::*;
#[cfg(feature = "windowed")]
use bevy_egui::{EguiContexts, egui};

#[cfg(feature = "windowed")]
use crate::combat::{
    events::{CombatEvent, CombatEventKind},
    kernel::CombatBeatId,
};

/// Windowed-only UI snapshot derived from `CombatEventKind::OnCombatBeat`.
///
/// This resource is presentation-owned on purpose: it tracks only the last
/// observed beat and derived label helpers. It must stay independent from
/// `CombatState`, unit components, turn order, and pipeline internals so the
/// phase-strip path cannot mutate gameplay state.
#[cfg(feature = "windowed")]
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhaseStripDisplay {
    pub current_beat: Option<CombatBeatId>,
}

#[cfg(feature = "windowed")]
impl PhaseStripDisplay {
    pub fn observe(&mut self, beat: CombatBeatId) {
        self.current_beat = Some(beat);
    }

    pub fn clear(&mut self) {
        self.current_beat = None;
    }

    pub fn active_phase(&self) -> Option<PhaseStripPhase> {
        self.current_beat.map(phase_strip_phase)
    }

    pub fn active_label(&self) -> Option<&'static str> {
        self.current_beat.map(phase_strip_label)
    }
}

/// Canonical section-9 display phases for the small top-center strip.
///
/// `Damage` reuses the `Impact` label because it is the committed hit within the
/// same visual strike window. `ExtraHit` is rendered as `Chain` so multi-hit or
/// reactive follow-through remains explicit without exposing engine internals.
#[cfg(feature = "windowed")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhaseStripPhase {
    Declared,
    PreApp,
    Impact,
    Chain,
    Applied,
    Resolved,
}

#[cfg(feature = "windowed")]
impl PhaseStripPhase {
    pub const ALL: [Self; 6] = [
        Self::Declared,
        Self::PreApp,
        Self::Impact,
        Self::Chain,
        Self::Applied,
        Self::Resolved,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Declared => "Declared",
            Self::PreApp => "Pre-App",
            Self::Impact => "Impact",
            Self::Chain => "Chain",
            Self::Applied => "Applied",
            Self::Resolved => "Resolved",
        }
    }
}

#[cfg(feature = "windowed")]
pub const fn phase_strip_phase(beat: CombatBeatId) -> PhaseStripPhase {
    match beat {
        CombatBeatId::Declared => PhaseStripPhase::Declared,
        CombatBeatId::PreApp => PhaseStripPhase::PreApp,
        CombatBeatId::Impact | CombatBeatId::Damage => PhaseStripPhase::Impact,
        CombatBeatId::ExtraHit => PhaseStripPhase::Chain,
        CombatBeatId::Applied => PhaseStripPhase::Applied,
        CombatBeatId::Resolved => PhaseStripPhase::Resolved,
    }
}

#[cfg(feature = "windowed")]
pub const fn phase_strip_label(beat: CombatBeatId) -> &'static str {
    phase_strip_phase(beat).label()
}

/// Returns the latest combat beat from an event stream, if any.
///
/// This helper is pure over the event payloads: it ignores all non-beat events
/// and does not touch combat resources, components, or world state.
#[cfg(feature = "windowed")]
pub fn latest_observed_combat_beat<'a>(events: impl IntoIterator<Item = &'a CombatEvent>) -> Option<CombatBeatId> {
    let mut latest_beat = None;

    for event in events {
        if let CombatEventKind::OnCombatBeat { beat } = event.kind {
            latest_beat = Some(beat);
        }
    }

    latest_beat
}

/// Read-only system seam for the combat-facing side of the phase-strip ingest path.
///
/// This function exists to make the boundary executable in tests via
/// `assert_is_read_only_system`: the combat bus may be read, but no combat
/// resource or component writer is part of the contract.
#[cfg(feature = "windowed")]
pub fn read_latest_observed_combat_beat(
    mut events: MessageReader<CombatEvent>,
) -> Option<CombatBeatId> {
    latest_observed_combat_beat(events.read())
}

/// Scans the combat event bus and updates only the UI-owned phase display.
///
/// The last `OnCombatBeat` message in the reader window wins. All other combat
/// event kinds are ignored so the strip remains a pure projection of the combat
/// observability stream rather than a second gameplay state machine.
#[cfg(feature = "windowed")]
pub fn observe_combat_beats(
    mut events: MessageReader<CombatEvent>,
    mut display: ResMut<PhaseStripDisplay>,
) {
    if let Some(beat) = latest_observed_combat_beat(events.read()) {
        display.observe(beat);
    }
}

/// Draws a compact top-center phase strip from the presentation-owned display.
///
/// Renders nothing until at least one combat beat has been observed. This keeps
/// the UI path side-effect-free: it reads the derived display resource and egui
/// context only, never combat state or world entities.
#[cfg(feature = "windowed")]
pub fn render_phase_strip(
    mut contexts: EguiContexts,
    display: Res<PhaseStripDisplay>,
) -> Result {
    let Some(active_phase) = display.active_phase() else {
        return Ok(());
    };

    let ctx = contexts.ctx_mut()?;
    egui::Area::new(egui::Id::new("combat_phase_strip"))
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 12.0))
        .order(egui::Order::Foreground)
        .interactable(false)
        .show(ctx, |ui| {
            egui::Frame::window(ui.style())
                .inner_margin(egui::Margin::symmetric(10, 8))
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        for phase in PhaseStripPhase::ALL {
                            let is_active = phase == active_phase;
                            let fill = if is_active {
                                egui::Color32::from_rgb(76, 114, 176)
                            } else {
                                egui::Color32::from_gray(44)
                            };
                            let stroke = if is_active {
                                egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(190, 220, 255))
                            } else {
                                egui::Stroke::new(1.0_f32, egui::Color32::from_gray(72))
                            };
                            let text = egui::RichText::new(phase.label())
                                .strong()
                                .color(if is_active {
                                    egui::Color32::WHITE
                                } else {
                                    egui::Color32::from_gray(170)
                                });

                            egui::Frame::default()
                                .fill(fill)
                                .stroke(stroke)
                                .corner_radius(egui::CornerRadius::same(6))
                                .inner_margin(egui::Margin::symmetric(8, 4))
                                .show(ui, |ui| {
                                    ui.label(text);
                                });
                        }
                    });
                });
        });

    Ok(())
}

#[cfg(all(test, feature = "windowed"))]
mod tests {
    use bevy::prelude::*;

    use super::{
        PhaseStripDisplay, PhaseStripPhase, observe_combat_beats, phase_strip_label,
        phase_strip_phase,
    };
    use crate::combat::{
        events::{ActionIntentKind, CombatEvent, CombatEventKind},
        kernel::CombatBeatId,
        runtime::intent::CastId,
        state::CombatState,
        types::UnitId,
    };

    fn build_event(beat: CombatBeatId) -> CombatEvent {
        CombatEvent {
            kind: CombatEventKind::OnCombatBeat { beat },
            source: UnitId(1),
            target: UnitId(2),
            follow_up_depth: 0,
            cast_id: CastId::ROOT,
        }
    }

    fn build_app() -> App {
        let mut app = App::new();
        app.add_message::<CombatEvent>()
            .init_resource::<PhaseStripDisplay>()
            .insert_resource(CombatState::default())
            .add_systems(Update, observe_combat_beats);
        app
    }

    #[test]
    fn default_display_has_no_active_phase() {
        let display = PhaseStripDisplay::default();

        assert_eq!(display.current_beat, None);
        assert_eq!(display.active_phase(), None);
        assert_eq!(display.active_label(), None);
    }

    #[test]
    fn every_known_beat_has_a_non_empty_label() {
        for beat in CombatBeatId::ALL {
            let label = phase_strip_label(beat);
            assert!(
                !label.trim().is_empty(),
                "expected non-empty phase-strip label for {:?}",
                beat
            );
        }
    }

    #[test]
    fn core_cycle_beats_keep_canonical_labels() {
        assert_eq!(phase_strip_label(CombatBeatId::Declared), "Declared");
        assert_eq!(phase_strip_label(CombatBeatId::PreApp), "Pre-App");
        assert_eq!(phase_strip_label(CombatBeatId::Impact), "Impact");
        assert_eq!(phase_strip_label(CombatBeatId::Applied), "Applied");
        assert_eq!(phase_strip_label(CombatBeatId::Resolved), "Resolved");
    }

    #[test]
    fn grouped_display_mapping_is_explicit_for_damage_and_extra_hit() {
        assert_eq!(
            phase_strip_phase(CombatBeatId::Damage),
            PhaseStripPhase::Impact
        );
        assert_eq!(phase_strip_label(CombatBeatId::Damage), "Impact");
        assert_eq!(
            phase_strip_phase(CombatBeatId::ExtraHit),
            PhaseStripPhase::Chain
        );
        assert_eq!(phase_strip_label(CombatBeatId::ExtraHit), "Chain");
    }

    #[test]
    fn display_tracks_observed_beats_without_needing_combat_state() {
        let mut display = PhaseStripDisplay::default();
        display.observe(CombatBeatId::PreApp);
        assert_eq!(display.active_phase(), Some(PhaseStripPhase::PreApp));
        assert_eq!(display.active_label(), Some("Pre-App"));

        display.observe(CombatBeatId::Resolved);
        assert_eq!(display.active_phase(), Some(PhaseStripPhase::Resolved));
        assert_eq!(display.active_label(), Some("Resolved"));

        display.clear();
        assert_eq!(display.active_phase(), None);
        assert_eq!(display.active_label(), None);
    }

    #[test]
    fn ingest_system_leaves_display_inactive_without_events() {
        let mut app = build_app();
        let before_state = app.world().resource::<CombatState>().clone();

        app.update();

        assert_eq!(
            app.world().resource::<PhaseStripDisplay>().current_beat,
            None,
            "no combat events should leave the phase strip empty"
        );
        assert_eq!(
            *app.world().resource::<CombatState>(),
            before_state,
            "phase strip ingestion must not mutate combat state"
        );
    }

    #[test]
    fn ingest_system_ignores_non_beat_events() {
        let mut app = build_app();
        {
            let mut display = app.world_mut().resource_mut::<PhaseStripDisplay>();
            display.observe(CombatBeatId::Impact);
        }
        let before_state = app.world().resource::<CombatState>().clone();

        app.world_mut().write_message(CombatEvent {
            kind: CombatEventKind::OnActionDeclared {
                intent_kind: ActionIntentKind::Basic,
            },
            source: UnitId(1),
            target: UnitId(2),
            follow_up_depth: 0,
            cast_id: CastId::ROOT,
        });

        app.update();

        assert_eq!(
            app.world().resource::<PhaseStripDisplay>().current_beat,
            Some(CombatBeatId::Impact),
            "non-beat events must not disturb the active phase display"
        );
        assert_eq!(*app.world().resource::<CombatState>(), before_state);
    }

    #[test]
    fn ingest_system_uses_the_latest_beat_when_multiple_arrive() {
        let mut app = build_app();
        let before_state = app.world().resource::<CombatState>().clone();

        app.world_mut().write_message(build_event(CombatBeatId::Declared));
        app.world_mut().write_message(build_event(CombatBeatId::PreApp));
        app.world_mut().write_message(build_event(CombatBeatId::Resolved));

        app.update();

        let display = app.world().resource::<PhaseStripDisplay>();
        assert_eq!(display.current_beat, Some(CombatBeatId::Resolved));
        assert_eq!(display.active_phase(), Some(PhaseStripPhase::Resolved));
        assert_eq!(display.active_label(), Some("Resolved"));
        assert_eq!(*app.world().resource::<CombatState>(), before_state);
    }
}
