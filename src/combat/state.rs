use bevy::prelude::Resource;

use crate::data::skills_ron::{DamageCurve, SkillCustomSignal, TargetShape};

use super::status_effect::StatusEffectKind;
use super::team::Team;
use super::types::{DamageTag, SkillId, UnitId};

// Used by S06/T02.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatPhase {
    /// AV system is ticking; no unit has been selected yet.
    WaitingForTurn,
    /// A unit has been selected to act; waiting for an ActionIntent.
    WaitingAction,
    Resolving,
    Victory,
    Defeat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UltEffect {
    /// Basic attack: amount comes from attacker's charge_per_event field at application time.
    GainFromBasic,
    None,
    Reset,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedAction {
    pub source: UnitId,
    pub target: UnitId,
    pub skill_id: SkillId,
    pub damage_tag: DamageTag,
    pub base_damage: i32,
    pub toughness_damage: i32,
    pub revive_pct: i32,
    pub sp_cost: i32,
    pub ult_effect: UltEffect,
    /// Number of ally free-basic casts to grant after this action resolves (from GrantFreeSkill effect).
    pub grant_free_skill_count: usize,
    /// First ApplyStatus effect found in the skill definition; first match wins.
    pub status_to_apply: Option<(StatusEffectKind, u32)>,
    pub advance_pct: u32,
    pub delay_pct: u32,
    /// Energy to grant the attacker from a GrantEnergy effect (0 = none).
    pub energy_grant: i32,
    /// AV self-advance percent from SelfAdvance effect (targets attacker, not defender).
    pub self_advance_pct: i32,
    pub target_shape: TargetShape,
    pub custom_signals: Vec<SkillCustomSignal>,
    /// Per-hop damage curve (relevant for `TargetShape::Bounce`; `Constant` for all other shapes).
    pub damage_curve: DamageCurve,
}

#[derive(Debug, Clone)]
pub struct InFlightAction {
    pub action: ResolvedAction,
    pub interrupted: bool,
    pub follow_up_depth: u8,
}

// Used by S06/T02.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatState {
    pub phase: CombatPhase,
    pub winner: Option<Team>,
}

impl Resource for CombatState {}

impl Default for CombatState {
    fn default() -> Self {
        Self {
            phase: CombatPhase::WaitingAction,
            winner: None,
        }
    }
}

impl CombatState {
    // Used by S06/T02.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    // kept for: M020 reactive bus (UnitDied taxonomy) + M023 phase-strip
    // observability; exercised by unit tests in this module.
    #[allow(dead_code)]
    pub fn update_terminal_state(&mut self, ally_alive: bool, enemy_alive: bool) {
        if self.winner.is_some() {
            return;
        }
        if !enemy_alive {
            self.phase = CombatPhase::Victory;
            self.winner = Some(Team::Ally);
        } else if !ally_alive {
            self.phase = CombatPhase::Defeat;
            self.winner = Some(Team::Enemy);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_starts_waiting_without_winner() {
        let state = CombatState::default();
        assert_eq!(state.phase, CombatPhase::WaitingAction);
        assert_eq!(state.winner, None);
    }

    #[test]
    fn victory_on_all_enemies_ko() {
        let mut state = CombatState::default();

        state.update_terminal_state(true, false);

        assert_eq!(state.phase, CombatPhase::Victory);
        assert_eq!(state.winner, Some(Team::Ally));
    }

    #[test]
    fn defeat_on_all_allies_ko() {
        let mut state = CombatState::default();

        state.update_terminal_state(false, true);

        assert_eq!(state.phase, CombatPhase::Defeat);
        assert_eq!(state.winner, Some(Team::Enemy));
    }

    #[test]
    fn reset_clears_winner() {
        let mut state = CombatState {
            phase: CombatPhase::Victory,
            winner: Some(Team::Ally),
        };

        state.reset();

        assert_eq!(state.phase, CombatPhase::WaitingAction);
        assert_eq!(state.winner, None);
    }
}
