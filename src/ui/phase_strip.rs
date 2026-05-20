#[cfg(feature = "windowed")]
use bevy::prelude::Resource;

#[cfg(feature = "windowed")]
use crate::combat::kernel::CombatBeatId;

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

#[cfg(all(test, feature = "windowed"))]
mod tests {
    use super::{phase_strip_label, phase_strip_phase, PhaseStripDisplay, PhaseStripPhase};
    use crate::combat::kernel::CombatBeatId;

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
}
