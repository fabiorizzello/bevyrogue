use bevy::prelude::*;
use bevyrogue::combat::{
    bootstrap::{EncounterComposition, apply_composition},
    team::Team,
    unit::SlotIndex,
};
use bevyrogue::data::units_ron::UnitRoster;
use std::collections::HashSet;

fn load_roster() -> UnitRoster {
    bevyrogue::data::aggregate_unit_roster()
}

fn make_3v3(roster: &UnitRoster) -> EncounterComposition {
    let allies: Vec<_> = roster
        .0
        .iter()
        .filter(|d| d.team == Team::Ally)
        .take(3)
        .cloned()
        .collect();
    assert_eq!(allies.len(), 3);

    let enemies: Vec<_> = roster
        .0
        .iter()
        .filter(|d| d.team == Team::Enemy)
        .take(3)
        .cloned()
        .collect();
    assert_eq!(enemies.len(), 3);

    EncounterComposition { allies, enemies }
}

fn collect_slots(app: &mut App, team_filter: Team) -> Vec<u8> {
    let mut q = app.world_mut().query::<(&Team, &SlotIndex)>();
    let world = app.world();
    q.iter(world)
        .filter_map(|(t, s)| (*t == team_filter).then_some(s.0))
        .collect()
}

/// Each team's slot range after apply_composition is exactly {0, 1, 2}.
#[test]
fn slot_indices_are_0_1_2_per_team() {
    let roster = load_roster();
    let composition = make_3v3(&roster);

    let mut app = App::new();
    apply_composition(&mut app.world_mut().commands(), &composition);
    app.update();

    let ally_slots: HashSet<u8> = collect_slots(&mut app, Team::Ally).into_iter().collect();
    let enemy_slots: HashSet<u8> = collect_slots(&mut app, Team::Enemy).into_iter().collect();

    assert_eq!(
        ally_slots,
        HashSet::from([0, 1, 2]),
        "ally slots must be {{0,1,2}}"
    );
    assert_eq!(
        enemy_slots,
        HashSet::from([0, 1, 2]),
        "enemy slots must be {{0,1,2}}"
    );
}

/// SlotIndex values are unique within each team.
#[test]
fn slot_indices_unique_per_team() {
    let roster = load_roster();
    let composition = make_3v3(&roster);

    let mut app = App::new();
    apply_composition(&mut app.world_mut().commands(), &composition);
    app.update();

    let ally_slots = collect_slots(&mut app, Team::Ally);
    let enemy_slots = collect_slots(&mut app, Team::Enemy);

    let ally_unique: HashSet<_> = ally_slots.iter().copied().collect();
    let enemy_unique: HashSet<_> = enemy_slots.iter().copied().collect();

    assert_eq!(
        ally_slots.len(),
        ally_unique.len(),
        "ally slots must be unique"
    );
    assert_eq!(
        enemy_slots.len(),
        enemy_unique.len(),
        "enemy slots must be unique"
    );
}
