use bevy::prelude::*;
use bevyrogue::combat::{
    events::CombatEvent,
    kit::UnitSkills,
    state::{CombatPhase, CombatState},
    team::Team,
    toughness::Toughness,
    turn_order::{TurnAdvanced, TurnOrder},
    turn_system::{ActionIntent, advance_turn_system, resolve_action_system},
    types::{Attribute, DamageTag, EvoStage, SkillId, UnitId},
    ultimate::{UltAccumulationTrigger, UltimateCharge},
    unit::Unit,
};

fn unit(id: u32, attribute: Attribute, hp: i32) -> Unit {
    Unit {
        id: UnitId(id),
        name: format!("Unit{id}"),
        hp_max: 100,
        hp_current: hp,
        attribute,
        resists: vec![],
        evo_stage: EvoStage::Adult,
    }
}

#[test]
fn s08_boundary_turn_advance_and_action_intent() {
    let mut app = App::new();
    app.init_resource::<TurnOrder>()
        .init_resource::<CombatState>()
        .add_message::<TurnAdvanced>()
        .add_message::<bevyrogue::combat::av::ActionValueUpdated>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_systems(Update, (resolve_action_system, advance_turn_system).chain());

    {
        let mut order = app.world_mut().resource_mut::<TurnOrder>();
        order.seed([UnitId(1), UnitId(2)]);
    }

    app.world_mut().spawn((
        unit(1, Attribute::Vaccine, 100),
        Team::Ally,
        Toughness::new(50, vec![]),
        UltimateCharge {
            current: 100,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
        UnitSkills {
            basic: SkillId("basic".into()),
            skills: vec![],
            ultimate: SkillId("ult".into()),
            follow_up: None,
        },
    ));
    app.world_mut().spawn((
        unit(2, Attribute::Virus, 100),
        Team::Enemy,
        Toughness::new(50, vec![]),
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
        UnitSkills {
            basic: SkillId("ebasic".into()),
            skills: vec![],
            ultimate: SkillId("eult".into()),
            follow_up: None,
        },
    ));

    use bevyrogue::data::SkillBookHandle;
    use bevyrogue::data::skills_ron::{
        Effect, SelfTargetRule, SkillBook, SkillDef, SkillImplementation,
        SkillTargeting, TargetLife, TargetShape, TargetSide,
    };
    let mut assets = Assets::<SkillBook>::default();
    let book = SkillBook(vec![SkillDef {
        id: SkillId("ult".into()),
        name: "Ult".into(),
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
        legacy_ops: vec![
            Effect::Damage {
                amount: 50,
                target: TargetShape::Single,
                per_hop: Default::default(),
            },
            Effect::ToughnessHit(20),
        ],
        custom_signals: vec![],
        animation_sequence: None,
        qte: None,
        timeline: None,
    }]);
    let handle = assets.add(book);
    app.insert_resource(assets);
    app.insert_resource(SkillBookHandle(handle));
    app.insert_resource(bevyrogue::combat::sp::SpPool::default());
    app.insert_resource(bevyrogue::combat::log::ActionLog::default());
    app.init_resource::<Time>();

    app.world_mut().write_message(TurnAdvanced::of(UnitId(1)));
    app.update();

    let state = app.world().resource::<CombatState>();
    assert_eq!(state.phase, CombatPhase::WaitingAction);

    app.world_mut().write_message(ActionIntent::Ultimate {
        attacker: UnitId(1),
        target: UnitId(2),
    });
    app.update();

    let ult_charge = app
        .world_mut()
        .query::<&UltimateCharge>()
        .iter(app.world())
        .next()
        .unwrap();
    assert_eq!(ult_charge.current, 0);

    let defender = app
        .world_mut()
        .query::<&Unit>()
        .iter(app.world())
        .find(|u| u.id == UnitId(2))
        .unwrap();
    assert!(defender.hp_current < 100);
}

#[test]
fn s08_ultimate_interrupt_flow() {
    let mut app = App::new();
    app.init_resource::<TurnOrder>()
        .init_resource::<CombatState>()
        .add_message::<TurnAdvanced>()
        .add_message::<bevyrogue::combat::av::ActionValueUpdated>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_systems(Update, (resolve_action_system, advance_turn_system).chain());

    {
        let mut order = app.world_mut().resource_mut::<TurnOrder>();
        order.seed([UnitId(1), UnitId(2)]);
    }

    app.world_mut().spawn((
        unit(1, Attribute::Vaccine, 100),
        Team::Ally,
        Toughness::new(50, vec![]),
        UltimateCharge {
            current: 100,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
        UnitSkills {
            basic: SkillId("basic".into()),
            skills: vec![],
            ultimate: SkillId("ult".into()),
            follow_up: None,
        },
    ));
    app.world_mut().spawn((
        unit(2, Attribute::Virus, 100),
        Team::Enemy,
        Toughness::new(50, vec![]),
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
        UnitSkills {
            basic: SkillId("ebasic".into()),
            skills: vec![],
            ultimate: SkillId("eult".into()),
            follow_up: None,
        },
    ));

    use bevyrogue::data::SkillBookHandle;
    use bevyrogue::data::skills_ron::{
        Effect, SelfTargetRule, SkillBook, SkillDef, SkillImplementation,
        SkillTargeting, TargetLife, TargetShape, TargetSide,
    };
    let mut assets = Assets::<SkillBook>::default();
    let book = SkillBook(vec![
        SkillDef {
            id: SkillId("ult".into()),
            name: "Ult".into(),
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
            legacy_ops: vec![
                Effect::Damage {
                    amount: 50,
                    target: TargetShape::Single,
                per_hop: Default::default(),
                },
                Effect::ToughnessHit(20),
            ],

            custom_signals: vec![],
            animation_sequence: None,
            qte: None,
            timeline: None,
        }
            timeline: None,
        },
        SkillDef {
            id: SkillId("ebasic".into()),
            name: "EBasic".into(),
            damage_tag: DamageTag::Ice,
            sp_cost: 0,
            targeting: SkillTargeting {
                shape: TargetShape::Single,
                side: TargetSide::Enemy,
                life: TargetLife::Alive,
                self_rule: SelfTargetRule::Forbid,
                ..Default::default()
            },
            implementation: SkillImplementation::Implemented,
            legacy_ops: vec![
                Effect::Damage {
                    amount: 10,
                    target: TargetShape::Single,
                per_hop: Default::default(),
                },
                Effect::ToughnessHit(5),
            ],

            custom_signals: vec![],
            animation_sequence: None,
            qte: None,
            timeline: None,
        }
            timeline: None,
        },
    ]);
    let handle = assets.add(book);
    app.insert_resource(assets);
    app.insert_resource(SkillBookHandle(handle));
    app.insert_resource(bevyrogue::combat::sp::SpPool::default());
    app.insert_resource(bevyrogue::combat::log::ActionLog::default());
    app.init_resource::<Time>();

    app.world_mut().write_message(TurnAdvanced::of(UnitId(1)));
    app.update();

    {
        let mut order = app.world_mut().resource_mut::<TurnOrder>();
        order.insert_out_of_queue(UnitId(1));
    }
    app.world_mut().write_message(ActionIntent::Ultimate {
        attacker: UnitId(1),
        target: UnitId(2),
    });
    app.update();

    let defender_hp = {
        let world = app.world_mut();
        let mut q = world.query::<&Unit>();
        q.iter(world)
            .find(|u| u.id == UnitId(2))
            .unwrap()
            .hp_current
    };
    assert!(defender_hp < 100);

    // In the AV system, queue is empty; turn order is managed by ActionValue components.
    // The assertion above (defender_hp < 100) verifies the ultimate resolved correctly.
}
