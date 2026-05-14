use bevy::prelude::*;
use bevyrogue::combat::api::intent::CastId;
use bevyrogue::combat::{
    bootstrap::{EncounterPreset, SelectionRequest, apply_composition, bootstrap_encounter},
    events::{CombatEvent, CombatEventKind},
    follow_up::{
        FollowUpIntent, FollowUpTrace, follow_up_listener_system, resolve_follow_up_action_system,
    },
    log::ActionLog,
    sp::SpPool,
    state::{CombatPhase, CombatState},
    team::Team,
    turn_order::{TurnAdvanced, TurnOrder},
    turn_system::{ActionIntent, check_victory_system, resolve_action_system},
    types::{Attribute, DamageTag, EvoLineId, EvoStage, SkillId, UnitId},
    ultimate::{
        UltAccumulationTrigger, UltGainQueue, flush_ult_gain_system, ult_accumulation_system,
    },
    unit::Unit,
};
use bevyrogue::data::{SkillBookHandle, skills_ron::SkillBook, units_ron::UnitRoster};

// Deterministic tick budget for the smoke run.
const TICK_BUDGET: u32 = 300;

fn canonical_roster() -> UnitRoster {
    ron::from_str(include_str!("../assets/data/units.ron")).expect("parse units.ron")
}

fn canonical_skill_book() -> SkillBook {
    ron::from_str(include_str!("../assets/data/skills.ron")).expect("parse skills.ron")
}

/// Recording resource: every CombatEvent emitted by the bus is appended here.
#[derive(Resource, Default)]
struct EventLog(Vec<CombatEvent>);

fn record_events(mut reader: MessageReader<CombatEvent>, mut log: ResMut<EventLog>) {
    for ev in reader.read() {
        log.0.push(ev.clone());
    }
}

/// Returns true if the unit with the given id is not KO'd.
fn unit_alive(world: &mut World, id: UnitId) -> bool {
    use bevyrogue::combat::unit::Ko;
    let mut query = world.query::<(&Unit, Option<&Ko>)>();
    query
        .iter(world)
        .any(|(u, ko)| u.id == id && u.hp_current > 0 && ko.is_none())
}

#[test]
fn s_m006_roster_smoke_deterministic() {
    // --- 1. Parse asset files inline (no AssetServer) ---
    let roster = canonical_roster();
    let raw_book = canonical_skill_book();

    // Use a party with 2+ follow-up units so the "≥2 distinct ally follow-ups" bar is reachable.
    // Default party.ron (UnitId 1-4) only includes Agumon with a follow-up.
    // UnitId(1) Agumon  = OnEnemyBreak follow-up
    // UnitId(5) Dorumon = OnEnemyKill follow-up
    // UnitId(2) Gabumon = no follow-up (provides basic coverage)
    // UnitId(7) Renamon = OnAllyLowHp follow-up (may fire if HP threshold is crossed)
    let rookie_ids = vec![UnitId(1), UnitId(5), UnitId(2), UnitId(7)];
    let request = SelectionRequest {
        rookie_ids: rookie_ids.clone(),
    };
    let mut composition = bootstrap_encounter(&roster, &request, EncounterPreset::BossEncounter)
        .expect("bootstrap must succeed");

    // Inject two enemies: Enemy A is weak to Fire (Agumon breaks it on first hit).
    // HP is set high enough that the break chain does not kill before we see the KO at depth=0.
    composition.enemies = vec![
        bevyrogue::data::units_ron::UnitDef {
            id: UnitId(101),
            name: "EnemyA".into(),
            role_tags: vec![],
            signature_traits: vec![],
            hp_max: 200,
            attribute: Attribute::Free,
            team: Team::Enemy,
            basic_damage_tag: DamageTag::Fire,
            basic_skill: SkillId("baby_flame".into()),
            skill_ids: vec![],
            ultimate_skill: SkillId("baby_flame".into()),
            follow_up: None,
            enemy_traits: vec![],
            charged_attack: None,
            form_identity: None,
            twin_core: Default::default(),
            holy_support: Default::default(),
            resists: vec![],
            toughness_max: 10, // breaks on first Fire hit from Agumon
            weaknesses: vec![DamageTag::Fire],
            ultimate_trigger: 100,
            ultimate_cap: 100,
            ultimate_accumulation_trigger: UltAccumulationTrigger::OnBasicAttack,
            ultimate_charge_per_event: 25,
            speed: 40,
            evo_stage: EvoStage::Child,
            evo_line: EvoLineId("test".into()),
            evolves_to: vec![],
            tempo_resistant: false,
            toughness_category: Default::default(),
        },
        bevyrogue::data::units_ron::UnitDef {
            id: UnitId(102),
            name: "EnemyB".into(),
            role_tags: vec![],
            signature_traits: vec![],
            hp_max: 200,
            attribute: Attribute::Free,
            team: Team::Enemy,
            basic_damage_tag: DamageTag::Fire,
            basic_skill: SkillId("baby_flame".into()),
            skill_ids: vec![],
            ultimate_skill: SkillId("baby_flame".into()),
            follow_up: None,
            enemy_traits: vec![],
            charged_attack: None,
            form_identity: None,
            twin_core: Default::default(),
            holy_support: Default::default(),
            resists: vec![],
            toughness_max: 30,
            weaknesses: vec![],
            ultimate_trigger: 100,
            ultimate_cap: 100,
            ultimate_accumulation_trigger: UltAccumulationTrigger::OnBasicAttack,
            ultimate_charge_per_event: 25,
            speed: 35,
            evo_stage: EvoStage::Child,
            evo_line: EvoLineId("test".into()),
            evolves_to: vec![],
            tempo_resistant: false,
            toughness_category: Default::default(),
        },
    ];

    // --- 2. Build Bevy app (no plugins, no AssetServer) ---
    let mut app = App::new();

    // Insert skill book as an in-memory asset.
    let mut skill_assets = Assets::<SkillBook>::default();
    let book_handle = skill_assets.add(raw_book);
    app.insert_resource(skill_assets);
    app.insert_resource(SkillBookHandle(book_handle));

    app.init_resource::<CombatState>()
        .init_resource::<ActionLog>()
        .init_resource::<UltGainQueue>()
        .init_resource::<EventLog>()
        .init_resource::<Time>();

    // Use an unlimited SP pool so skills with non-zero SP costs don't silently fail.
    app.insert_resource(SpPool {
        current: 999,
        max: 999,
    });

    // Register message channels required by combat systems.
    app.add_message::<CombatEvent>()
        .add_message::<ActionIntent>()
        .add_message::<FollowUpIntent>()
        .add_message::<FollowUpTrace>()
        .add_message::<TurnAdvanced>();

    // --- 3. Spawn units from composition using Commands ---
    let mut order = TurnOrder::default();
    {
        let world = app.world_mut();
        let mut queue = bevy::ecs::world::CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, world);
            apply_composition(&mut commands, &composition, &mut order);
        }
        queue.apply(world);
    }
    app.insert_resource(order);

    // --- 4. Wire systems ---
    app.add_systems(
        Update,
        (
            resolve_action_system,
            follow_up_listener_system,
            resolve_follow_up_action_system,
            ult_accumulation_system,
            flush_ult_gain_system,
            check_victory_system,
            record_events,
        )
            .chain(),
    );

    // --- 5. Emit bootstrap events ---
    // In the AV system, future_preview is always empty; collect all spawned unit IDs instead.
    let preview: Vec<UnitId> = {
        let mut q = app.world_mut().query::<&Unit>();
        let mut ids: Vec<UnitId> = q.iter(app.world()).map(|u| u.id).collect();
        ids.sort_by_key(|id| id.0);
        ids
    };
    app.world_mut().write_message(CombatEvent {
        kind: CombatEventKind::PartySelected {
            ally_ids: rookie_ids.clone(),
            tamer_id: UnitId(0),
        },
        source: UnitId(0),
        target: UnitId(0),
        follow_up_depth: 0,
        cast_id: CastId::ROOT,
    });
    app.world_mut().write_message(CombatEvent {
        kind: CombatEventKind::TurnOrderSeeded { unit_ids: preview },
        source: UnitId(0),
        target: UnitId(0),
        follow_up_depth: 0,
        cast_id: CastId::ROOT,
    });
    // One tick to flush bootstrap events into the log.
    app.update();

    // --- 6. Drive combat ---
    // Script: cycle through allies with Basic attacks; after enough basics emit Taichi's ultimate.
    // TICK_BUDGET guards against infinite loops.
    let allies = [UnitId(1), UnitId(5), UnitId(2), UnitId(7)];
    let enemy_a = UnitId(101);
    let enemy_b = UnitId(102);
    let taichi = UnitId(0);
    let mut taichi_ult_fired = false;

    for tick in 0..TICK_BUDGET {
        let phase = app.world().resource::<CombatState>().phase;
        if matches!(phase, CombatPhase::Victory | CombatPhase::Defeat) {
            break;
        }

        // Pick target: prefer enemy_a while alive (to trigger break + follow-ups).
        let target_a_alive = unit_alive(app.world_mut(), enemy_a);
        let target = if target_a_alive { enemy_a } else { enemy_b };
        let target_alive = unit_alive(app.world_mut(), target);

        // After 10 ally basics Taichi's ult should be charged (10 events × 10 charge each).
        // Fire the ultimate once.
        if !taichi_ult_fired && tick >= 10 && target_alive {
            app.world_mut().write_message(ActionIntent::Ultimate {
                attacker: taichi,
                target,
            });
            taichi_ult_fired = true;
        } else if target_alive {
            let ally = allies[tick as usize % 4];
            app.world_mut().write_message(ActionIntent::Basic {
                attacker: ally,
                target,
            });
        }

        app.update();
    }

    // --- 7. Assertions ---
    let log = app.world().resource::<EventLog>();
    let events = &log.0;

    // PartySelected with 4 allies and correct tamer.
    let party_selected = events.iter().find(|e| {
        matches!(&e.kind, CombatEventKind::PartySelected { ally_ids, tamer_id }
            if ally_ids.len() == 4 && *tamer_id == UnitId(0))
    });
    assert!(
        party_selected.is_some(),
        "PartySelected event with 4 allies not found"
    );

    // TurnOrderSeeded: all 7 spawned units (5 allies + 2 enemies) in the AV system.
    let seeded = events.iter().find(
        |e| matches!(&e.kind, CombatEventKind::TurnOrderSeeded { unit_ids } if unit_ids.len() >= 5),
    );
    assert!(seeded.is_some(), "TurnOrderSeeded with ≥5 units not found");

    // OnSkillCast from each of the 4 ally rookies (Basic attacks now emit OnSkillCast).
    for &ally_id in &allies {
        let found = events
            .iter()
            .any(|e| e.source == ally_id && matches!(&e.kind, CombatEventKind::OnSkillCast { .. }));
        assert!(found, "no OnSkillCast from ally {:?}", ally_id);
    }

    // At least one Break.
    let has_break = events
        .iter()
        .any(|e| matches!(&e.kind, CombatEventKind::OnBreak { .. }));
    assert!(has_break, "no OnBreak event");

    // Signature follow-ups from ≥2 distinct ally UnitIds (follow_up_depth == 1).
    let follow_up_sources: std::collections::HashSet<UnitId> = events
        .iter()
        .filter(|e| {
            e.follow_up_depth == 1 && matches!(&e.kind, CombatEventKind::OnSkillCast { .. })
        })
        .map(|e| e.source)
        .collect();
    assert!(
        follow_up_sources.len() >= 2,
        "expected ≥2 distinct follow-up sources, got {:?}",
        follow_up_sources
    );

    // UltGain for Taichi (UnitId(0)) via ult_accumulation_system (OnOffensivePartyEvent).
    let taichi_ult_gain = events.iter().any(
        |e| matches!(&e.kind, CombatEventKind::UltGain { unit_id, .. } if *unit_id == UnitId(0)),
    );
    assert!(taichi_ult_gain, "no UltGain for Taichi (UnitId(0))");

    // UltGain for a non-Taichi unit (digimon Basic attack charges their own ult).
    let digimon_ult_gain = events.iter().any(
        |e| matches!(&e.kind, CombatEventKind::UltGain { unit_id, .. } if *unit_id != UnitId(0)),
    );
    assert!(digimon_ult_gain, "no UltGain for a non-Taichi unit");

    // Brave Tri-Strike: brave_tri_strike OnSkillCast followed by 4 ally OnSkillCast events.
    let brave_pos = events.iter().position(|e| {
        matches!(&e.kind, CombatEventKind::OnSkillCast { skill_id }
            if skill_id.0 == "brave_tri_strike")
    });
    assert!(
        brave_pos.is_some(),
        "brave_tri_strike OnSkillCast not found — Taichi ult may not have fired"
    );
    if let Some(pos) = brave_pos {
        let subsequent_skill_casts = events[pos + 1..]
            .iter()
            .filter(|e| matches!(&e.kind, CombatEventKind::OnSkillCast { .. }))
            .count();
        assert!(
            subsequent_skill_casts >= 4,
            "expected ≥4 ally OnSkillCast after brave_tri_strike, got {}",
            subsequent_skill_casts
        );
    }

    // OnRevive present OR documented absence (no ally reached KO during this run).
    let has_revive = events
        .iter()
        .any(|e| matches!(&e.kind, CombatEventKind::OnRevive { .. }));
    if !has_revive {
        println!("revive condition did not arise: no ally was KO'd during this smoke run");
    }

    // Combat must have ended in a terminal phase.
    let final_phase = app.world().resource::<CombatState>().phase;
    assert!(
        matches!(final_phase, CombatPhase::Victory | CombatPhase::Defeat),
        "expected Victory or Defeat, got {:?}",
        final_phase
    );
}
