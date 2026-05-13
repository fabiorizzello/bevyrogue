use bevy::{ecs::message::MessageCursor, prelude::*};
use bevyrogue::combat::{
    events::{CombatEvent, CombatEventKind},
    kit::UnitSkills,
    log::ActionLog,
    sp::SpPool,
    state::CombatState,
    team::Team,
    toughness::Toughness,
    turn_order::TurnOrder,
    turn_system::{ActionIntent, resolve_action_system},
    types::{Attribute, DamageTag, EvoStage, SkillId, UnitId},
    ultimate::{UltAccumulationTrigger, UltimateCharge},
    unit::{SlotIndex, Unit},
};
use bevyrogue::data::{
    SkillBookHandle,
    skills_ron::{
        Effect, SelfTargetRule, SkillBook, SkillDef, SkillImplementation, SkillTargeting,
        TargetLife, TargetShape, TargetSide,
    },
};

fn build_app(book: SkillBook) -> App {
    let mut assets = Assets::<SkillBook>::default();
    let handle = assets.add(book);
    let mut app = App::new();
    app.insert_resource(assets)
        .insert_resource(SkillBookHandle(handle))
        .init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .insert_resource(SpPool { current: 5, max: 5 })
        .init_resource::<ActionLog>()
        .init_resource::<Time>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_systems(Update, resolve_action_system);
    app
}

fn blast_skill() -> SkillBook {
    SkillBook(vec![SkillDef {
        id: SkillId("blast_strike".into()),
        name: "Blast Strike".into(),
        damage_tag: DamageTag::Fire,
        sp_cost: 2,
        targeting: SkillTargeting {
            shape: TargetShape::Blast,
            side: TargetSide::Enemy,
            life: TargetLife::Alive,
            self_rule: SelfTargetRule::Forbid,
            ..Default::default()
        },
        implementation: SkillImplementation::Implemented,
        effects: vec![
            Effect::Damage {
                amount: 10,
                target: TargetShape::Blast,
            },
            Effect::ToughnessHit(5),
        ],
        ..Default::default()
    }])
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

fn spawn_attacker(app: &mut App, id: u32, slot: u8) {
    let skill_id = SkillId("blast_strike".into());
    app.world_mut().spawn((
        Unit {
            id: UnitId(id),
            name: format!("Attacker{id}"),
            hp_max: 100,
            hp_current: 100,
            attribute: Attribute::Vaccine,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        Team::Ally,
        SlotIndex(slot),
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
        UnitSkills {
            basic: skill_id.clone(),
            skills: vec![skill_id.clone()],
            ultimate: skill_id,
            follow_up: None,
        },
    ));
}

fn spawn_enemy(app: &mut App, id: u32, slot: u8) {
    app.world_mut().spawn((
        Unit {
            id: UnitId(id),
            name: format!("Enemy{id}"),
            hp_max: 100,
            hp_current: 100,
            attribute: Attribute::Virus,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        Team::Enemy,
        SlotIndex(slot),
        Toughness {
            max: 40,
            current: 40,
            weaknesses: vec![DamageTag::Fire],
            broken: false,
            category: Default::default(),
        },
    ));
}

/// Blast on primary at slot 1 → hits slots 0, 1, 2 (all 3 enemies).
/// SP consumed once (2), not 3×.
#[test]
fn blast_hits_all_three_adjacent_enemies_sp_consumed_once() {
    let mut app = build_app(blast_skill());

    spawn_attacker(&mut app, 1, 0); // ally
    spawn_enemy(&mut app, 10, 0);
    spawn_enemy(&mut app, 11, 1); // primary target
    spawn_enemy(&mut app, 12, 2);

    let sp_before = app.world().resource::<SpPool>().current;
    assert_eq!(sp_before, 5);

    let mut cursor = message_cursor::<CombatEvent>(&mut app);

    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("blast_strike".into()),
        target: UnitId(11), // primary = slot 1
    });
    app.update();

    let events = drain_events(&mut cursor, &app);

    // 3 OnDamageDealt events emitted
    let damage_events: Vec<&CombatEvent> = events
        .iter()
        .filter(|e| matches!(e.kind, CombatEventKind::OnDamageDealt { .. }))
        .collect();
    assert_eq!(
        damage_events.len(),
        3,
        "Blast should deal damage to all 3 enemies; got {}",
        damage_events.len()
    );

    // Events ordered by slot_index ascending: target = 10 (slot 0), 11 (slot 1), 12 (slot 2)
    let target_order: Vec<UnitId> = damage_events.iter().map(|e| e.target).collect();
    assert_eq!(
        target_order,
        vec![UnitId(10), UnitId(11), UnitId(12)],
        "Blast damage order must be slot_index ascending"
    );

    // SP consumed once: 5 - 2 = 3
    let sp_after = app.world().resource::<SpPool>().current;
    assert_eq!(
        sp_after, 3,
        "SP consumed once (cost=2), not 3× (got sp={})",
        sp_after
    );

    // All 3 enemies took damage
    let mut unit_q = app.world_mut().query::<(&Unit, &Team)>();
    let world = app.world();
    for enemy_id in [UnitId(10), UnitId(11), UnitId(12)] {
        let hp = unit_q
            .iter(world)
            .find(|(u, t)| u.id == enemy_id && **t == Team::Enemy)
            .map(|(u, _)| u.hp_current)
            .expect("enemy not found");
        assert!(
            hp < 100,
            "enemy {:?} should have taken damage, hp={}",
            enemy_id,
            hp
        );
    }
}

/// Blast on primary at slot 0 → no slot -1, only slots 0 and 1 hit.
#[test]
fn blast_edge_slot_zero_hits_only_two_enemies() {
    let mut app = build_app(blast_skill());

    spawn_attacker(&mut app, 1, 0);
    spawn_enemy(&mut app, 10, 0); // primary
    spawn_enemy(&mut app, 11, 1);
    spawn_enemy(&mut app, 12, 2);

    let mut cursor = message_cursor::<CombatEvent>(&mut app);

    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("blast_strike".into()),
        target: UnitId(10), // primary = slot 0
    });
    app.update();

    let events = drain_events(&mut cursor, &app);

    let damage_events: Vec<&CombatEvent> = events
        .iter()
        .filter(|e| matches!(e.kind, CombatEventKind::OnDamageDealt { .. }))
        .collect();
    assert_eq!(
        damage_events.len(),
        2,
        "Blast at edge slot 0 should hit 2 enemies (slots 0,1)"
    );

    let target_order: Vec<UnitId> = damage_events.iter().map(|e| e.target).collect();
    assert_eq!(
        target_order,
        vec![UnitId(10), UnitId(11)],
        "Blast edge slot order: slot0 then slot1"
    );
}
