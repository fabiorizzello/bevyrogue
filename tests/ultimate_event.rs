/// Integration tests for M020/S01: UltimateUsed event emitted on ultimate cast.
///
/// Verifies:
/// - Exactly one `UltimateUsed { unit_id }` event with the correct attacker ID is emitted
///   when an Ultimate intent fires with a full meter.
/// - No `UltimateUsed` event is emitted for a Basic intent.
/// - No `UltimateUsed` event is emitted for a Skill (non-Reset) intent.
use bevy::{ecs::message::MessageCursor, prelude::*};
use bevyrogue::combat::{
    events::{CombatEvent, CombatEventKind},
    kit::UnitSkills,
    log::ActionLog,
    sp::SpPool,
    state::CombatState,
    team::Team,
    toughness::Toughness,
    turn_order::{TurnAdvanced, TurnOrder},
    turn_system::{ActionIntent, resolve_action_system},
    types::{Attribute, DamageTag, EvoStage, SkillId, UnitId},
    ultimate::{UltAccumulationTrigger, UltimateCharge},
    unit::Unit,
};
use bevyrogue::data::{
    SkillBookHandle,
    skills_ron::{
        Effect, SelfTargetRule, SkillBook, SkillDef, SkillImplementation, SkillTargeting,
        TargetLife, TargetShape, TargetSide,
    },
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_unit(id: u32, name: &str, hp: i32) -> Unit {
    Unit {
        id: UnitId(id),
        name: name.into(),
        hp_max: hp,
        hp_current: hp,
        attribute: Attribute::Vaccine,
        resists: vec![],
        evo_stage: EvoStage::Adult,
    }
}

fn ult_charge_ready() -> UltimateCharge {
    UltimateCharge {
        current: 100,
        trigger: 100,
        cap: 150,
        trigger_type: UltAccumulationTrigger::OnBasicAttack,
        charge_per_event: 25,
    }
}

fn ult_charge_empty() -> UltimateCharge {
    UltimateCharge {
        current: 0,
        trigger: 100,
        cap: 150,
        trigger_type: UltAccumulationTrigger::OnBasicAttack,
        charge_per_event: 25,
    }
}

fn ult_skill_def() -> SkillDef {
    SkillDef {
        id: SkillId("ult".into()),
        name: "Ultimate".into(),
        damage_tag: DamageTag::Fire,
        sp_cost: 0,
        targeting: SkillTargeting {
            shape: TargetShape::Single,
            side: TargetSide::Enemy,
            life: TargetLife::Alive,
            self_rule: SelfTargetRule::Forbid,
            ..Default::default()
        },
        implementation: SkillImplementation::Implemented,
        effects: vec![Effect::Damage {
            amount: 50,
            target: TargetShape::Single,
            per_hop: Default::default(),
        }],
        custom_signals: vec![],
        animation_sequence: None,
        qte: None,
    }
}

fn basic_skill_def() -> SkillDef {
    SkillDef {
        id: SkillId("basic".into()),
        name: "Basic".into(),
        damage_tag: DamageTag::Fire,
        sp_cost: 0,
        targeting: SkillTargeting {
            shape: TargetShape::Single,
            side: TargetSide::Enemy,
            life: TargetLife::Alive,
            self_rule: SelfTargetRule::Forbid,
            ..Default::default()
        },
        implementation: SkillImplementation::Implemented,
        effects: vec![Effect::Damage {
            amount: 10,
            target: TargetShape::Single,
            per_hop: Default::default(),
        }],
        custom_signals: vec![],
        animation_sequence: None,
        qte: None,
    }
}

fn skill_def() -> SkillDef {
    SkillDef {
        id: SkillId("skill1".into()),
        name: "Skill".into(),
        damage_tag: DamageTag::Fire,
        sp_cost: 0,
        targeting: SkillTargeting {
            shape: TargetShape::Single,
            side: TargetSide::Enemy,
            life: TargetLife::Alive,
            self_rule: SelfTargetRule::Forbid,
            ..Default::default()
        },
        implementation: SkillImplementation::Implemented,
        effects: vec![Effect::Damage {
            amount: 20,
            target: TargetShape::Single,
            per_hop: Default::default(),
        }],
        custom_signals: vec![],
        animation_sequence: None,
        qte: None,
    }
}

fn unit_skills() -> UnitSkills {
    UnitSkills {
        basic: SkillId("basic".into()),
        skills: vec![SkillId("skill1".into())],
        ultimate: SkillId("ult".into()),
        follow_up: None,
    }
}

fn setup_app(skill_book: SkillBook) -> App {
    let mut app = App::new();
    let mut assets = Assets::<SkillBook>::default();
    let handle = assets.add(skill_book);
    app.insert_resource(assets)
        .insert_resource(SkillBookHandle(handle))
        .init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .init_resource::<SpPool>()
        .init_resource::<ActionLog>()
        .init_resource::<Time>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_message::<TurnAdvanced>()
        .add_systems(Update, resolve_action_system);
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

fn collect_ultimate_used(events: &[CombatEvent]) -> Vec<UnitId> {
    events
        .iter()
        .filter_map(|ev| {
            if let CombatEventKind::UltimateUsed { unit_id } = ev.kind {
                Some(unit_id)
            } else {
                None
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Firing an ultimate with a full meter emits exactly one `UltimateUsed` event
/// with `unit_id == attacker`.
#[test]
fn ultimate_used_emitted_once_on_ult_cast() {
    let mut app = setup_app(SkillBook(vec![
        ult_skill_def(),
        basic_skill_def(),
        skill_def(),
    ]));

    let attacker_id = UnitId(1);
    let defender_id = UnitId(2);

    app.world_mut().spawn((
        make_unit(attacker_id.0, "Attacker", 200),
        Team::Ally,
        unit_skills(),
        ult_charge_ready(),
        Toughness::new(100, vec![]),
    ));
    app.world_mut().spawn((
        make_unit(defender_id.0, "Defender", 500),
        Team::Enemy,
        Toughness::new(200, vec![]),
    ));

    app.world_mut().resource_mut::<TurnOrder>().seed([attacker_id, defender_id]);

    let mut cursor = message_cursor::<CombatEvent>(&mut app);
    app.world_mut().write_message(ActionIntent::Ultimate {
        attacker: attacker_id,
        target: defender_id,
    });
    app.update();

    let events = drain_events(&mut cursor, &app);
    let used = collect_ultimate_used(&events);
    assert_eq!(
        used.len(),
        1,
        "expected exactly one UltimateUsed event, got {:?}",
        used
    );
    assert_eq!(
        used[0], attacker_id,
        "UltimateUsed unit_id should match attacker"
    );
}

/// Basic attack emits no `UltimateUsed` event.
#[test]
fn no_ultimate_used_on_basic_attack() {
    let mut app = setup_app(SkillBook(vec![
        ult_skill_def(),
        basic_skill_def(),
        skill_def(),
    ]));

    let attacker_id = UnitId(1);
    let defender_id = UnitId(2);

    app.world_mut().spawn((
        make_unit(attacker_id.0, "Attacker", 200),
        Team::Ally,
        unit_skills(),
        ult_charge_empty(),
        Toughness::new(100, vec![]),
    ));
    app.world_mut().spawn((
        make_unit(defender_id.0, "Defender", 500),
        Team::Enemy,
        Toughness::new(200, vec![]),
    ));

    app.world_mut().resource_mut::<TurnOrder>().seed([attacker_id, defender_id]);

    let mut cursor = message_cursor::<CombatEvent>(&mut app);
    app.world_mut().write_message(ActionIntent::Basic {
        attacker: attacker_id,
        target: defender_id,
    });
    app.update();

    let events = drain_events(&mut cursor, &app);
    let used = collect_ultimate_used(&events);
    assert!(
        used.is_empty(),
        "expected no UltimateUsed event for Basic, got {:?}",
        used
    );
}

/// Skill intent (non-Reset, no ult spend) emits no `UltimateUsed` event.
#[test]
fn no_ultimate_used_on_skill_cast() {
    let mut app = setup_app(SkillBook(vec![
        ult_skill_def(),
        basic_skill_def(),
        skill_def(),
    ]));

    let attacker_id = UnitId(1);
    let defender_id = UnitId(2);

    app.world_mut().spawn((
        make_unit(attacker_id.0, "Attacker", 200),
        Team::Ally,
        unit_skills(),
        ult_charge_empty(),
        Toughness::new(100, vec![]),
    ));
    app.world_mut().spawn((
        make_unit(defender_id.0, "Defender", 500),
        Team::Enemy,
        Toughness::new(200, vec![]),
    ));

    app.world_mut().resource_mut::<TurnOrder>().seed([attacker_id, defender_id]);

    let mut cursor = message_cursor::<CombatEvent>(&mut app);
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: attacker_id,
        target: defender_id,
        skill_id: SkillId("skill1".into()),
    });
    app.update();

    let events = drain_events(&mut cursor, &app);
    let used = collect_ultimate_used(&events);
    assert!(
        used.is_empty(),
        "expected no UltimateUsed event for Skill, got {:?}",
        used
    );
}
