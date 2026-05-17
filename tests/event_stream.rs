use bevy::{ecs::message::MessageCursor, prelude::*};
use bevyrogue::combat::api::SignalPayload;
use bevyrogue::combat::{
    events::{ActionIntentKind, CombatEvent, CombatEventKind},
    kernel::{CombatBeatId, CombatKernelTransition, register_combat_kernel_runtime},
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
    unit::Unit,
};
use bevyrogue::data::{
    SkillBookHandle,
    skills_ron::{
        Effect, SelfTargetRule, SkillBook, SkillDef, SkillImplementation, SkillTargeting,
        TargetLife, TargetShape, TargetSide,
    },
};

fn unit(id: u32, hp_max: i32, hp_current: i32, attribute: Attribute) -> Unit {
    Unit {
        id: UnitId(id),
        name: format!("Unit{id}"),
        hp_max,
        hp_current,
        attribute,
        resists: vec![],
        evo_stage: EvoStage::Adult,
    }
}

fn cursor(app: &mut App) -> MessageCursor<CombatEvent> {
    app.world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor()
}

fn drain(cursor: &mut MessageCursor<CombatEvent>, app: &App, out: &mut Vec<CombatEvent>) {
    let messages = app.world().resource::<Messages<CombatEvent>>();
    out.extend(cursor.read(messages).cloned());
}

#[test]
fn s09_event_stream_observes_all_variants() {
    let mut app = App::new();
    register_combat_kernel_runtime(&mut app);

    app.init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .init_resource::<SpPool>()
        .init_resource::<ActionLog>()
        .init_resource::<Time>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_systems(Update, resolve_action_system);

    let mut assets = Assets::<SkillBook>::default();
    let handle = assets.add(SkillBook(vec![
        SkillDef {
            id: SkillId("basic_a1".into()),
            name: "Basic A1".into(),
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
        },
        SkillDef {
            id: SkillId("skill_a1".into()),
            name: "Skill A1".into(),
            damage_tag: DamageTag::Fire,
            sp_cost: 1,
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
                    amount: 20,
                    target: TargetShape::Single,
                    per_hop: Default::default(),
                },
                Effect::ToughnessHit(5),
            ],
            custom_signals: vec![],
            animation_sequence: None,
            qte: None,
            timeline: None,
        },
        SkillDef {
            id: SkillId("enemy_basic".into()),
            name: "Enemy Basic".into(),
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
                Effect::ToughnessHit(0),
            ],
            custom_signals: vec![],
            animation_sequence: None,
            qte: None,
            timeline: None,
        },
    ]));
    app.insert_resource(assets);
    app.insert_resource(SkillBookHandle(handle));
    app.world_mut().resource_mut::<SpPool>().current = 5;

    app.world_mut().spawn((
        unit(1, 100, 100, Attribute::Vaccine),
        Team::Ally,
        Toughness::new(50, vec![]),
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
        UnitSkills {
            basic: SkillId("basic_a1".into()),
            skills: vec![SkillId("skill_a1".into())],
            ultimate: SkillId("skill_a1".into()),
            follow_up: None,
        },
    ));
    app.world_mut().spawn((
        unit(2, 40, 20, Attribute::Data),
        Team::Ally,
        Toughness::new(20, vec![]),
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
        UnitSkills {
            basic: SkillId("basic_a1".into()),
            skills: vec![],
            ultimate: SkillId("basic_a1".into()),
            follow_up: None,
        },
    ));
    app.world_mut().spawn((
        unit(3, 25, 25, Attribute::Virus),
        Team::Enemy,
        Toughness::new(5, vec![DamageTag::Fire]),
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
        UnitSkills {
            basic: SkillId("enemy_basic".into()),
            skills: vec![],
            ultimate: SkillId("enemy_basic".into()),
            follow_up: None,
        },
    ));
    app.world_mut().spawn((
        unit(4, 100, 100, Attribute::Virus),
        Team::Enemy,
        Toughness::new(20, vec![]),
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
        UnitSkills {
            basic: SkillId("enemy_basic".into()),
            skills: vec![],
            ultimate: SkillId("enemy_basic".into()),
            follow_up: None,
        },
    ));

    let mut reader = cursor(&mut app);
    let mut seen = Vec::new();

    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("skill_a1".into()),
        target: UnitId(3),
    });
    app.update();
    drain(&mut reader, &app, &mut seen);

    app.world_mut().write_message(ActionIntent::Basic {
        attacker: UnitId(4),
        target: UnitId(2),
    });
    app.update();
    drain(&mut reader, &app, &mut seen);

    let kinds: Vec<CombatEventKind> = seen.iter().map(|e| e.kind.clone()).collect();
    assert!(
        kinds
            .iter()
            .any(|k| matches!(k, CombatEventKind::OnSkillCast { .. }))
    );
    assert!(
        kinds
            .iter()
            .any(|k| matches!(k, CombatEventKind::OnDamageDealt { .. }))
    );
    assert!(
        kinds
            .iter()
            .any(|k| matches!(k, CombatEventKind::OnBreak { .. }))
    );
    assert!(
        kinds
            .iter()
            .any(|k| matches!(k, CombatEventKind::UnitDied { .. }))
    );
    assert!(
        kinds
            .iter()
            .any(|k| matches!(k, CombatEventKind::OnAllyLowHp))
    );
    assert!(
        kinds
            .iter()
            .any(|k| matches!(k, CombatEventKind::OnEnemyKill))
    );

    assert!(kinds.iter().all(|kind| matches!(
        kind,
        CombatEventKind::OnSkillCast { .. }
            | CombatEventKind::OnDamageDealt { .. }
            | CombatEventKind::OnBreak { .. }
            | CombatEventKind::UnitDied { .. }
            | CombatEventKind::OnAllyLowHp
            | CombatEventKind::OnEnemyKill
            | CombatEventKind::UltGain { .. }
            | CombatEventKind::OnHitTaken { .. }
            | CombatEventKind::OnActionDeclared { .. }
            | CombatEventKind::OnActionPreApp
            | CombatEventKind::OnCombatBeat { .. }
            | CombatEventKind::OnKernelTransition { .. }
            | CombatEventKind::OnActionApplied
            | CombatEventKind::OnActionResolved
            | CombatEventKind::OnStatusApplied { .. }
            | CombatEventKind::OnStatusResisted { .. }
    )));
    let beat_ids: Vec<CombatBeatId> = kinds
        .iter()
        .filter_map(|kind| match kind {
            CombatEventKind::OnCombatBeat { beat } => Some(*beat),
            _ => None,
        })
        .collect();
    assert!(beat_ids.contains(&CombatBeatId::Declared));
    assert!(beat_ids.contains(&CombatBeatId::PreApp));
    assert!(beat_ids.contains(&CombatBeatId::Impact));
    assert!(beat_ids.contains(&CombatBeatId::Damage));
    assert!(beat_ids.contains(&CombatBeatId::Applied));
    assert!(beat_ids.contains(&CombatBeatId::Resolved));

    let kernel_beat_ids: Vec<CombatBeatId> = kinds
        .iter()
        .filter_map(|kind| match kind {
            CombatEventKind::OnKernelTransition {
                transition: CombatKernelTransition::Beat(beat),
            } => Some(*beat),
            _ => None,
        })
        .collect();
    assert_eq!(beat_ids, kernel_beat_ids);

    for owner in ["twin_core", "dorumon", "tentomon"] {
        let event = CombatEvent {
            kind: CombatEventKind::OnKernelTransition {
                transition: CombatKernelTransition::Blueprint {
                    owner: owner.to_string(),
                    name: "signal".to_string(),
                    payload: SignalPayload::Amount(1),
                },
            },
            source: UnitId(1),
            target: UnitId(2),
            follow_up_depth: 0,
            cast_id: bevyrogue::combat::api::intent::CastId::ROOT,
        };
        let serialized = serde_json::to_string(&event).expect("serialize blueprint event");
        assert!(serialized.contains(owner), "{serialized}");
        assert!(serialized.contains("Blueprint"), "{serialized}");
    }
    // Keep the coarse intent enum exercised so serde coverage stays anchored on the shared bus.
    let _ = ActionIntentKind::Basic;
}
