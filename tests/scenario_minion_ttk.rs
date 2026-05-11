use bevy::{ecs::message::MessageCursor, prelude::*};
use bevyrogue::combat::{
    av::ActionValueUpdated,
    bootstrap::{EncounterPreset, SelectionRequest, bootstrap_encounter, spawn_unit_from_def},
    events::CombatEvent,
    follow_up::{
        FollowUpIntent, FollowUpTrace, follow_up_listener_system, form_identity_listener_system,
        resolve_follow_up_action_system,
    },
    log::ActionLog,
    rng::CombatRng,
    sp::SpPool,
    state::{CombatPhase, CombatState},
    team::Team,
    turn_order::{TurnAdvanced, TurnOrder},
    turn_system::{ActionIntent, advance_turn_system, check_victory_system, resolve_action_system},
    types::UnitId,
    unit::{Ko, Unit},
};
use bevyrogue::data::{SkillBookHandle, skills_ron::SkillBook, units_ron::UnitRoster};

fn load_roster() -> UnitRoster {
    ron::from_str(include_str!("../assets/data/units.ron")).expect("parse units.ron")
}

fn load_skill_book() -> SkillBook {
    ron::from_str(include_str!("../assets/data/skills.ron")).expect("parse skills.ron")
}

fn build_app(skill_book: SkillBook) -> App {
    let mut assets = Assets::<SkillBook>::default();
    let handle = assets.add(skill_book);
    let mut app = App::new();
    app.insert_resource(assets)
        .insert_resource(SkillBookHandle(handle))
        .init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .insert_resource(SpPool { current: 5, max: 5 })
        .init_resource::<ActionLog>()
        .init_resource::<Time>()
        .insert_resource(CombatRng::from_seed(42))
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_message::<FollowUpIntent>()
        .add_message::<FollowUpTrace>()
        .add_message::<TurnAdvanced>()
        .add_message::<ActionValueUpdated>()
        .add_systems(
            Update,
            (
                resolve_action_system,
                follow_up_listener_system,
                form_identity_listener_system,
                resolve_follow_up_action_system,
                advance_turn_system,
                check_victory_system,
            )
                .chain(),
        );
    app
}

fn message_cursor<T: Message>(app: &mut App) -> MessageCursor<T> {
    app.world_mut().resource_mut::<Messages<T>>().get_cursor()
}

fn drain_events(cursor: &mut MessageCursor<CombatEvent>, app: &App) -> Vec<CombatEvent> {
    cursor
        .read(app.world().resource::<Messages<CombatEvent>>())
        .cloned()
        .collect()
}

fn live_enemy_ids(app: &mut App) -> Vec<UnitId> {
    let mut q = app.world_mut().query::<(&Unit, &Team, Option<&Ko>)>();
    let mut ids: Vec<UnitId> = q
        .iter(app.world())
        .filter(|(u, t, ko)| **t == Team::Enemy && u.hp_current > 0 && ko.is_none())
        .map(|(u, _, _)| u.id)
        .collect();
    ids.sort_by_key(|id| id.0);
    ids.dedup();
    ids
}

fn is_ally_alive(app: &mut App, id: UnitId) -> bool {
    let mut q = app.world_mut().query::<(&Unit, &Team, Option<&Ko>)>();
    q.iter(app.world())
        .any(|(u, t, ko)| u.id == id && *t == Team::Ally && u.hp_current > 0 && ko.is_none())
}

/// R083 TTK fixture — MinionWave preset (3× Goblimon).
///
/// Party: Greymon, Angemon, Kabuterimon, DORUgamon (Adults).
/// Expected turn band: 2–3 full ally rounds.
/// This test is INTENTIONALLY test-first: it is expected to FAIL against current
/// untuned numbers and defines the rebalance target for T03.
#[test]
fn minion_wave_ttk_target_2_to_3_turns() {
    let roster = load_roster();
    let book = load_skill_book();
    let mut app = build_app(book);

    let request = SelectionRequest {
        rookie_ids: vec![UnitId(12), UnitId(17), UnitId(14), UnitId(16)],
    };
    let composition =
        bootstrap_encounter(&roster, &request, EncounterPreset::MinionWave).expect("bootstrap");

    for def in composition.allies.iter().chain(composition.enemies.iter()) {
        spawn_unit_from_def(&mut app.world_mut().commands(), def);
    }
    app.update();

    let party = [UnitId(12), UnitId(17), UnitId(14), UnitId(16)];
    let mut cursor = message_cursor::<CombatEvent>(&mut app);
    let mut turn_count = 0;

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
            }
            drain_events(&mut cursor, &app);
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
        "MinionWave did not reach Victory within 12 turns"
    );
    // R083 target: expected to FAIL until T03 rebalance ships
    assert!(
        turn_count >= 2 && turn_count <= 3,
        "R083: MinionWave TTK out of target range — expected 2–3 turns, actual {turn_count}"
    );
}
