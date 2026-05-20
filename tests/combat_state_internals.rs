//! Relocated from `src/combat/state.rs` (R003 — no inline `mod tests` in src/).
//! Pure relocate: all touched symbols are already `pub`.

use bevyrogue::combat::state::{CombatPhase, CombatState};
use bevyrogue::combat::team::Team;

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
