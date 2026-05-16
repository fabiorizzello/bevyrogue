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

fn all_enemies_skill() -> SkillBook {
    SkillBook(vec![SkillDef {
        id: SkillId("wide_blast".into()),
        name: "Wide Blast".into(),
        damage_tag: DamageTag::Fire,
        sp_cost: 2,
        targeting: SkillTargeting {
            shape: TargetShape::AllEnemies,
            side: TargetSide::Enemy,
            life: TargetLife::Alive,
            self_rule: SelfTargetRule::Forbid,
            ..Default::default()
        },
        implementation: SkillImplementation::Implemented,
        legacy_ops: vec![
            Effect::Damage {
                amount: 10,
                target: TargetShape::AllEnemies,
            per_hop: Default::default(),
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

fn spawn_attacker(app: &mut App) {
    let skill_id = SkillId("wide_blast".into());
    app.world_mut().spawn((
        Unit {
            id: UnitId(1),
            name: "Attacker".into(),
            hp_max: 100,
            hp_current: 100,
            attribute: Attribute::Vaccine,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        Team::Ally,
        SlotIndex(0),
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

fn spawn_enemy_alive(app: &mut App, id: u32, slot: u8) {
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

fn spawn_enemy_ko(app: &mut App, id: u32, slot: u8) {
    app.world_mut().spawn((
        Unit {
            id: UnitId(id),
            name: format!("EnemyKO{id}"),
            hp_max: 100,
            hp_current: 0, // KO'd
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

fn run_once(app: &mut App) -> Vec<CombatEvent> {
    let mut cursor = message_cursor::<CombatEvent>(app);
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("wide_blast".into()),
        target: UnitId(10), // primary (doesn't matter for AllEnemies)
    });
    app.update();
    drain_events(&mut cursor, app)
}

/// AllEnemies on 3-enemy formation where slot 1 is KO'd.
/// Exactly 2 OnDamageDealt events, for the 2 alive enemies in slot_index asc order.
/// SP consumed once.
#[test]
fn aoe_skips_ko_and_fires_in_slot_order() {
    let mut app = build_app(all_enemies_skill());

    spawn_attacker(&mut app);
    spawn_enemy_alive(&mut app, 10, 0);  // slot 0 — alive
    spawn_enemy_ko(&mut app, 11, 1);     // slot 1 — KO'd
    spawn_enemy_alive(&mut app, 12, 2);  // slot 2 — alive

    let events = run_once(&mut app);

    let damage_events: Vec<&CombatEvent> = events
        .iter()
        .filter(|e| matches!(e.kind, CombatEventKind::OnDamageDealt { .. }))
        .collect();

    assert_eq!(
        damage_events.len(),
        2,
        "AllEnemies should hit only 2 alive enemies (KO'd skipped); got {}",
        damage_events.len()
    );

    let target_order: Vec<UnitId> = damage_events.iter().map(|e| e.target).collect();
    assert_eq!(
        target_order,
        vec![UnitId(10), UnitId(12)],
        "AllEnemies order must be slot_index ascending, KO'd skipped"
    );

    // SP consumed once: 5 - 2 = 3
    assert_eq!(
        app.world().resource::<SpPool>().current,
        3,
        "SP consumed once, not 2×"
    );

    // KO'd enemy untouched
    let mut q = app.world_mut().query::<(&Unit, &Team)>();
    let world = app.world();
    let ko_hp = q
        .iter(world)
        .find(|(u, t)| u.id == UnitId(11) && **t == Team::Enemy)
        .map(|(u, _)| u.hp_current)
        .expect("KO enemy not found");
    assert_eq!(ko_hp, 0, "KO'd enemy should remain at 0 HP");
}

/// Run the AllEnemies test 10 times to confirm deterministic ordering.
#[test]
fn aoe_order_is_deterministic_across_10_runs() {
    for _run in 0..10 {
        let mut app = build_app(all_enemies_skill());
        spawn_attacker(&mut app);
        spawn_enemy_alive(&mut app, 10, 0);
        spawn_enemy_ko(&mut app, 11, 1);
        spawn_enemy_alive(&mut app, 12, 2);

        let events = run_once(&mut app);
        let target_order: Vec<UnitId> = events
            .iter()
            .filter(|e| matches!(e.kind, CombatEventKind::OnDamageDealt { .. }))
            .map(|e| e.target)
            .collect();

        assert_eq!(
            target_order,
            vec![UnitId(10), UnitId(12)],
            "run {_run}: order must be [10, 12]"
        );
    }
}
