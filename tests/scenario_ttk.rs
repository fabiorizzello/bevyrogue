//! R083 TTK fixtures — Minion / MiniBoss / Boss encounters.
//!
//! These tests are INTENTIONALLY test-first: they are expected to FAIL against
//! current untuned numbers and define the rebalance target for T03. The three
//! presets share an identical drive loop (party basics into the first live
//! enemy until Victory) so they live behind one `rstest` case matrix.
//!
//! Replaces three single-purpose files:
//!   * `scenario_minion_ttk.rs`
//!   * `scenario_miniboss_ttk.rs`
//!   * `scenario_boss_ttk.rs`

use bevy::prelude::*;
use bevyrogue::combat::{
    bootstrap::{EncounterPreset, SelectionRequest, bootstrap_encounter, spawn_unit_from_def},
    events::{CombatEvent, CombatEventKind},
    state::{CombatPhase, CombatState},
    turn_system::ActionIntent,
    types::UnitId,
};
use rstest::rstest;

mod common;
use common::{
    build_app, drain_events, is_ally_alive, live_enemy_ids, load_roster, load_skill_book,
    message_cursor,
};

/// Adults: Greymon, Angemon, Kabuterimon, DORUgamon.
const PARTY: [UnitId; 4] = [UnitId(12), UnitId(17), UnitId(14), UnitId(16)];

/// Outcome counters captured during the drive loop.
#[derive(Default, Debug)]
struct TtkOutcome {
    turn_count: usize,
    break_count: usize,
    energy_count: usize,
}

fn drive_to_victory(preset: EncounterPreset, max_turns: usize) -> (App, TtkOutcome) {
    let roster = load_roster();
    let book = load_skill_book();
    let mut app = build_app(book);

    let request = SelectionRequest {
        rookie_ids: PARTY.to_vec(),
    };
    let composition = bootstrap_encounter(&roster, &request, preset).expect("bootstrap");

    for def in composition.allies.iter().chain(composition.enemies.iter()) {
        spawn_unit_from_def(&mut app.world_mut().commands(), def);
    }
    app.update();

    let mut cursor = message_cursor::<CombatEvent>(&mut app);
    let mut outcome = TtkOutcome::default();

    'combat: for _ in 0..max_turns {
        outcome.turn_count += 1;
        for &ally_id in &PARTY {
            if !is_ally_alive(&mut app, ally_id) {
                continue;
            }
            let enemies = live_enemy_ids(&mut app);
            let Some(&target) = enemies.first() else {
                break 'combat;
            };
            app.world_mut().write_message(ActionIntent::Basic {
                attacker: ally_id,
                target,
            });
            for _ in 0..4 {
                app.update();
                for ev in drain_events(&mut cursor, &app) {
                    match ev.kind {
                        CombatEventKind::OnBreak { .. } => outcome.break_count += 1,
                        CombatEventKind::EnergyGained { .. } => outcome.energy_count += 1,
                        _ => {}
                    }
                }
            }
            if app.world().resource::<CombatState>().phase == CombatPhase::Victory {
                break 'combat;
            }
        }
        if app.world().resource::<CombatState>().phase == CombatPhase::Victory {
            break;
        }
    }

    (app, outcome)
}

/// R083 TTK targets per preset.
///
/// * `MinionWave`         — 2–3 turns; no extra event guards.
/// * `MiniBossEncounter`  — 3–5 turns; ≥1 OnBreak (Ogremon bar).
/// * `BossEncounter`      — 4–7 turns; ≥1 OnBreak AND ≥1 EnergyGained.
///
/// All three are expected to FAIL until T03 rebalance ships.
#[rstest]
#[case::minion(EncounterPreset::MinionWave, 2..=3, 0, 0)]
#[case::miniboss(EncounterPreset::MiniBossEncounter, 3..=5, 1, 0)]
#[case::boss(EncounterPreset::BossEncounter, 4..=7, 1, 1)]
fn ttk_band(
    #[case] preset: EncounterPreset,
    #[case] band: std::ops::RangeInclusive<usize>,
    #[case] min_break: usize,
    #[case] min_energy: usize,
) {
    let (app, outcome) = drive_to_victory(preset, 12);

    assert_eq!(
        app.world().resource::<CombatState>().phase,
        CombatPhase::Victory,
        "{preset:?} did not reach Victory within 12 turns ({outcome:?})"
    );
    assert!(
        outcome.break_count >= min_break,
        "R083: {preset:?} expected ≥{min_break} OnBreak, got {}",
        outcome.break_count
    );
    assert!(
        outcome.energy_count >= min_energy,
        "R083: {preset:?} expected ≥{min_energy} EnergyGained, got {}",
        outcome.energy_count
    );
    assert!(
        band.contains(&outcome.turn_count),
        "R083: {preset:?} TTK out of target range {band:?}, actual {}",
        outcome.turn_count
    );
}
