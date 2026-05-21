use bevy::prelude::*;
use bevyrogue::combat::runtime::timeline::{Beat, BeatEdge, BeatKind, BeatPayload};
use bevyrogue::combat::{
    events::CombatEvent,
    kit::UnitSkills,
    log::ActionLog,
    rng::CombatRng,
    runtime::{
        ExtRegistries, SignalBus, SignalTaxonomy, register_kernel_builtins,
        timeline::TimelineLibrary,
    },
    sp::SpPool,
    state::{CombatPhase, CombatState},
    team::Team,
    toughness::Toughness,
    turn_order::{TurnAdvanced, TurnOrder},
    turn_system::{ActionIntent, advance_turn_system, resolve_action_system},
    types::{Attribute, DamageTag, EvoStage, SkillId, UnitId},
    ultimate::{UltAccumulationTrigger, UltimateCharge},
    unit::Unit,
};
use bevyrogue::data::skill_timeline::SkillTimeline;
use bevyrogue::data::{
    SkillBookHandle,
    skill_timeline::compile_skill_book_timelines,
    skills_ron::{
        Effect, SelfTargetRule, SkillBook, SkillDef, SkillImplementation, SkillTargeting,
        TargetLife, TargetShape, TargetSide,
    },
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

fn damage_timeline(amount: i32, tag: DamageTag, toughness: i32) -> Option<SkillTimeline> {
    Some(SkillTimeline {
        entry: "cast".into(),
        beats: vec![
            Beat {
                id: "cast".into(),
                kind: BeatKind::Cast,
                hook: None,
                selector: None,
                presentation: None,
                payload: None,
            },
            Beat {
                id: "impact_damage".into(),
                kind: BeatKind::Impact,
                hook: Some("core/deal_damage".into()),
                selector: Some("core/primary".into()),
                presentation: None,
                payload: Some(BeatPayload::DealDamage {
                    amount,
                    tag,
                    target: TargetShape::Single,
                }),
            },
            Beat {
                id: "impact_break".into(),
                kind: BeatKind::Impact,
                hook: Some("core/apply_effect".into()),
                selector: Some("core/primary".into()),
                presentation: None,
                payload: Some(BeatPayload::BreakToughness {
                    amount: toughness,
                    tag,
                    target: TargetShape::Single,
                }),
            },
        ],
        edges: vec![
            BeatEdge {
                from: "cast".into(),
                to: "impact_damage".into(),
                gate: Some("core/always".into()),
            },
            BeatEdge {
                from: "impact_damage".into(),
                to: "impact_break".into(),
                gate: Some("core/always".into()),
            },
        ],
    })
}

fn build_app(book: &SkillBook) -> App {
    let mut app = App::new();
    let mut assets = Assets::<SkillBook>::default();
    let handle = assets.add(book.clone());

    app.insert_resource(assets)
        .insert_resource(SkillBookHandle(handle))
        .init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .insert_resource(SpPool::default())
        .init_resource::<ActionLog>()
        .init_resource::<Time>()
        .insert_resource(CombatRng::from_seed(42))
        .insert_resource(TimelineLibrary::<String>::default())
        .init_resource::<SignalBus>()
        .init_resource::<ExtRegistries>()
        .init_resource::<SignalTaxonomy>()
        .add_message::<TurnAdvanced>()
        .add_message::<bevyrogue::combat::av::ActionValueUpdated>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_systems(Update, (resolve_action_system, advance_turn_system).chain());

    {
        let mut regs = app.world_mut().resource_mut::<ExtRegistries>();
        register_kernel_builtins(&mut regs);
        let compiled =
            compile_skill_book_timelines(book, &regs).expect("test timeline book must compile");
        app.world_mut()
            .resource_mut::<TimelineLibrary<String>>()
            .timelines = compiled;
    }

    app
}

#[test]
fn turn_advance_then_ultimate_consumes_meter_and_damages_target() {
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
        timeline: damage_timeline(50, DamageTag::Fire, 20),
    }]);

    let mut app = build_app(&book);

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
            basic: SkillId("ult".into()),
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
            basic: SkillId("ult".into()),
            skills: vec![],
            ultimate: SkillId("ult".into()),
            follow_up: None,
        },
    ));

    app.world_mut().write_message(TurnAdvanced::of(UnitId(1)));
    app.update();

    let state = app.world().resource::<CombatState>();
    assert_eq!(state.phase, CombatPhase::WaitingAction);

    app.world_mut().write_message(ActionIntent::Ultimate {
        attacker: UnitId(1),
        target: UnitId(2),
    });
    app.update();

    let attacker_ult = {
        let world = app.world_mut();
        let mut q = world.query::<(&Unit, &UltimateCharge)>();
        q.iter(world)
            .find(|(u, _)| u.id == UnitId(1))
            .map(|(_, uc)| uc.current)
            .unwrap()
    };
    assert_eq!(attacker_ult, 0);

    let defender = app
        .world_mut()
        .query::<&Unit>()
        .iter(app.world())
        .find(|u| u.id == UnitId(2))
        .unwrap();
    assert!(defender.hp_current < 100);
}
