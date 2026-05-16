use bevy::prelude::*;
use bevyrogue::combat::{
    kit::UnitSkills,
    log::{ActionLog, LogEntry},
    sp::SpPool,
    state::CombatState,
    team::Team,
    toughness::Toughness,
    turn_system::{ActionIntent, resolve_action_system},
    types::{Attribute, DamageTag, EvoStage, SkillId, UnitId},
    ultimate::{UltAccumulationTrigger, UltimateCharge},
    unit::{Ko, Unit},
};
use bevyrogue::data::{
    SkillBookHandle,
    skills_ron::{
        Effect, SelfTargetRule, SkillBook, SkillDef, SkillImplementation,
        SkillTargeting, TargetLife, TargetShape, TargetSide,
    },
};

fn sync_events(app: &App, collected: &mut Vec<LogEntry>) {
    let log = app.world().resource::<ActionLog>();
    for ev in log.events.iter() {
        let key = format!("{ev:?}");
        if !collected.iter().any(|c| format!("{c:?}") == key) {
            collected.push(ev.clone());
        }
    }
}

#[test]
fn s12_revive_semantics() {
    let mut app = App::new();
    app.init_resource::<CombatState>()
        .init_resource::<bevyrogue::combat::turn_order::TurnOrder>()
        .add_message::<ActionIntent>()
        .add_message::<bevyrogue::combat::events::CombatEvent>()
        .add_systems(Update, resolve_action_system);

    let mut assets = Assets::<SkillBook>::default();
    let book = SkillBook(vec![
        SkillDef {
            id: SkillId("revive_skill".into()),
            name: "Revive Skill".into(),
            damage_tag: DamageTag::Light,
            sp_cost: 0,
            targeting: SkillTargeting {
                shape: TargetShape::Single,
                side: TargetSide::Ally,
                life: TargetLife::Ko,
                self_rule: SelfTargetRule::Forbid,
                ..Default::default()
            },
            implementation: SkillImplementation::Implemented,
            legacy_ops: vec![Effect::Revive(25)],

            custom_signals: vec![],
            animation_sequence: None,
            qte: None,
            timeline: None,
        },
        SkillDef {
            id: SkillId("attack_skill".into()),
            name: "Attack Skill".into(),
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
            legacy_ops: vec![Effect::Damage {
                amount: 1000,
                target: TargetShape::Single,
                per_hop: Default::default(),
            }],

            custom_signals: vec![],
            animation_sequence: None,
            qte: None,
            timeline: None,
        },
    ]);
    let handle = assets.add(book);
    app.insert_resource(assets);
    app.insert_resource(SkillBookHandle(handle));
    app.insert_resource(ActionLog::default());
    app.insert_resource(SpPool::default());
    app.init_resource::<Time>();

    let unit = |id: u32, name: &str, hp: i32, team: Team| {
        (
            Unit {
                id: UnitId(id),
                name: name.into(),
                hp_max: hp,
                hp_current: hp,
                attribute: Attribute::Vaccine,
                resists: vec![],
                evo_stage: EvoStage::Adult,
            },
            team,
            Toughness::new(50, vec![]),
            UltimateCharge {
                current: 0,
                trigger: 100,
                cap: 150,
                trigger_type: UltAccumulationTrigger::OnBasicAttack,
                charge_per_event: 25,
            },
            UnitSkills {
                basic: SkillId("attack_skill".into()),
                skills: vec![SkillId("revive_skill".into())],
                ultimate: SkillId("attack_skill".into()),
                follow_up: None,
            },
        )
    };

    let _a1_entity = app
        .world_mut()
        .spawn(unit(1, "Ally1", 100, Team::Ally))
        .id();
    let a2_entity = app
        .world_mut()
        .spawn(unit(2, "Ally2", 100, Team::Ally))
        .id();
    let _e1_entity = app
        .world_mut()
        .spawn(unit(3, "Enemy1", 100, Team::Enemy))
        .id();

    let mut collected = Vec::new();

    // --- 1. Successful revive of a KO ally ---

    // Enemy attacks Ally2 to KO
    app.world_mut().write_message(ActionIntent::Basic {
        attacker: UnitId(3),
        target: UnitId(2),
    });
    app.update();
    sync_events(&app, &mut collected);

    assert!(
        app.world().get::<Ko>(a2_entity).is_some(),
        "Ally2 should be KO"
    );
    assert!(
        collected
            .iter()
            .any(|e| matches!(e, LogEntry::Ko { target } if *target == UnitId(2)))
    );

    // Ally1 revives Ally2
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("revive_skill".into()),
        target: UnitId(2),
    });
    app.update();
    sync_events(&app, &mut collected);

    assert!(
        app.world().get::<Ko>(a2_entity).is_none(),
        "Ally2 should not be KO after revive"
    );
    assert!(
        collected
            .iter()
            .any(|e| matches!(e, LogEntry::Revive { target, .. } if *target == UnitId(2)))
    );

    let ally2_hp = app.world().get::<Unit>(a2_entity).unwrap().hp_current;
    assert_eq!(ally2_hp, 25, "Ally2 should have 25 HP (25% of 100)");

    // --- 2. Explicit failure on non-KO ally ---

    // Ally1 tries to revive Ally2 again (Ally2 is now alive)
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("revive_skill".into()),
        target: UnitId(2),
    });
    app.update();
    sync_events(&app, &mut collected);

    assert!(
        collected
            .iter()
            .any(|e| matches!(e, LogEntry::ActionFailed { reason } if reason == "TargetNotKo"))
    );

    // --- 3. Explicit failure of attack on KO target ---

    // Enemy attacks Ally2 to KO again
    app.world_mut().write_message(ActionIntent::Basic {
        attacker: UnitId(3),
        target: UnitId(2),
    });
    app.update();
    assert!(app.world().get::<Ko>(a2_entity).is_some());

    // Enemy tries to attack KO'd Ally2 again
    app.world_mut().write_message(ActionIntent::Basic {
        attacker: UnitId(3),
        target: UnitId(2),
    });
    app.update();
    sync_events(&app, &mut collected);

    assert!(
        collected
            .iter()
            .any(|e| matches!(e, LogEntry::ActionFailed { reason } if reason == "TargetKo"))
    );
}

