use bevy::prelude::*;
use bevyrogue::combat::{
    bootstrap::{EncounterPreset, SelectionRequest, bootstrap_encounter, spawn_unit_from_def},
    events::{CombatEvent, CombatEventKind},
    state::{CombatPhase, CombatState},
    turn_system::ActionIntent,
    types::UnitId,
};

mod common;
use common::{
    build_app, drain_events, is_ally_alive, live_enemy_ids, load_roster, load_skill_book,
    message_cursor,
};

/// R083 TTK fixture — BossEncounter preset (Devimon solo, Armored, tempo-resistant).
///
/// Party: Greymon, Angemon, Kabuterimon, DORUgamon (Adults).
/// Expected turn band: 4–7 full ally rounds; at least one OnBreak AND one EnergyGained event.
/// This test is INTENTIONALLY test-first: expected to FAIL against current untuned
/// numbers and defines the rebalance target for T03.
#[test]
fn boss_encounter_ttk_target_4_to_7_turns() {
    let roster = load_roster();
    let book = load_skill_book();
    let mut app = build_app(book);

    let request = SelectionRequest {
        rookie_ids: vec![UnitId(12), UnitId(17), UnitId(14), UnitId(16)],
    };
    let composition =
        bootstrap_encounter(&roster, &request, EncounterPreset::BossEncounter).expect("bootstrap");

    for def in composition.allies.iter().chain(composition.enemies.iter()) {
        spawn_unit_from_def(&mut app.world_mut().commands(), def);
    }
    app.update();

    let party = [UnitId(12), UnitId(17), UnitId(14), UnitId(16)];
    let mut cursor = message_cursor::<CombatEvent>(&mut app);
    let mut turn_count = 0;
    let mut break_count = 0usize;
    let mut energy_count = 0usize;

    'combat: for _ in 0..12 {
        turn_count += 1;
        for &ally_id in &party {
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
                        CombatEventKind::OnBreak { .. } => break_count += 1,
                        CombatEventKind::EnergyGained { .. } => energy_count += 1,
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

    assert_eq!(
        app.world().resource::<CombatState>().phase,
        CombatPhase::Victory,
        "BossEncounter did not reach Victory within 12 turns"
    );
    // R083 targets: all three expected to FAIL until T03 rebalance ships
    assert!(
        energy_count >= 1,
        "R083: BossEncounter expected at least 1 EnergyGained (Form Identity), got {energy_count}"
    );
    assert!(
        break_count >= 1,
        "R083: BossEncounter expected at least 1 OnBreak (Devimon bar), got {break_count}"
    );
    assert!(
        turn_count >= 4 && turn_count <= 7,
        "R083: BossEncounter TTK out of target range — expected 4–7 turns, actual {turn_count}"
    );
}
