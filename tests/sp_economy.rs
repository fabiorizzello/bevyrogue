/// Integration test for M008/S05: deterministic SP economy regression harness.
///
/// This test exercises the real `resolve_action_system` path with a scripted
/// 20-turn headless combat loop and records `SpPool.current` after every turn.
/// It verifies both gate conditions separately:
/// - no 3-turn window can remain at cap
/// - a second revive attempt fails until enough basic attacks rebuild SP
use bevy::prelude::*;
use bevyrogue::combat::{
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
    unit::{Ko, Unit},
};
use bevyrogue::data::{
    SkillBookHandle,
    skills_ron::{
        Effect, SelfTargetRule, SkillBook, SkillDef, SkillImplementation,
        SkillTargeting, TargetLife, TargetShape, TargetSide,
    },
};

fn make_unit(id: u32, name: &str, hp_max: i32, hp_current: i32, attribute: Attribute) -> Unit {
    Unit {
        id: UnitId(id),
        name: name.into(),
        hp_max,
        hp_current,
        attribute,
        resists: vec![],
        evo_stage: EvoStage::Adult,
    }
}

fn make_ult() -> UltimateCharge {
    UltimateCharge {
        current: 0,
        trigger: 100,
        cap: 150,
        trigger_type: UltAccumulationTrigger::OnBasicAttack,
        charge_per_event: 25,
    }
}

fn build_app() -> App {
    let mut app = App::new();
    let mut assets = Assets::<SkillBook>::default();
    let book = SkillBook(vec![
        SkillDef {
            id: SkillId("ally_basic".into()),
            name: "Ally Basic".into(),
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
                amount: 8,
                target: TargetShape::Single,
            per_hop: Default::default(),
            }],

            custom_signals: vec![],
            animation_sequence: None,
            qte: None,
            timeline: None,
        },
        SkillDef {
            id: SkillId("ally_skill_3".into()),
            name: "Ally Skill 3".into(),
            damage_tag: DamageTag::Ice,
            sp_cost: 3,
            targeting: SkillTargeting {
                shape: TargetShape::Single,
                side: TargetSide::Enemy,
                life: TargetLife::Alive,
                self_rule: SelfTargetRule::Forbid,
                ..Default::default()
            },
            implementation: SkillImplementation::Implemented,
            legacy_ops: vec![Effect::Damage {
                amount: 16,
                target: TargetShape::Single,
            per_hop: Default::default(),
            }],

            custom_signals: vec![],
            animation_sequence: None,
            qte: None,
            timeline: None,
        },
        SkillDef {
            id: SkillId("ally_skill_4".into()),
            name: "Ally Skill 4".into(),
            damage_tag: DamageTag::Light,
            sp_cost: 4,
            targeting: SkillTargeting {
                shape: TargetShape::Single,
                side: TargetSide::Enemy,
                life: TargetLife::Alive,
                self_rule: SelfTargetRule::Forbid,
                ..Default::default()
            },
            implementation: SkillImplementation::Implemented,
            legacy_ops: vec![Effect::Damage {
                amount: 18,
                target: TargetShape::Single,
            per_hop: Default::default(),
            }],

            custom_signals: vec![],
            animation_sequence: None,
            qte: None,
            timeline: None,
        },
        SkillDef {
            id: SkillId("holy_revive".into()),
            name: "Holy Revive".into(),
            damage_tag: DamageTag::Light,
            sp_cost: 5,
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
            id: SkillId("enemy_smash".into()),
            name: "Enemy Smash".into(),
            damage_tag: DamageTag::Dark,
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
                amount: 9999,
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

    app.insert_resource(assets)
        .insert_resource(SkillBookHandle(handle))
        .init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .insert_resource(SpPool { current: 5, max: 5 })
        .init_resource::<ActionLog>()
        .init_resource::<Time>()
        .add_message::<ActionIntent>()
        .add_message::<bevyrogue::combat::events::CombatEvent>()
        .add_systems(Update, resolve_action_system);
    app
}

fn cast(app: &mut App, intent: ActionIntent, history: &mut Vec<i32>) {
    app.world_mut().write_message(intent);
    app.update();
    history.push(app.world().resource::<SpPool>().current);
}

#[test]
fn s_m008_s05_sp_economy_20_turn_regression() {
    let mut app = build_app();
    let ally = app
        .world_mut()
        .spawn((
            make_unit(1, "Calibrator", 120, 120, Attribute::Vaccine),
            Team::Ally,
            Toughness::new(1000, vec![]),
            make_ult(),
            UnitSkills {
                basic: SkillId("ally_basic".into()),
                skills: vec![
                    SkillId("ally_skill_3".into()),
                    SkillId("ally_skill_4".into()),
                    SkillId("holy_revive".into()),
                ],
                ultimate: SkillId("ally_basic".into()),
                follow_up: None,
            },
        ))
        .id();

    let victim = app
        .world_mut()
        .spawn((
            make_unit(2, "Victim", 100, 0, Attribute::Vaccine),
            Team::Ally,
            Ko,
            Toughness::new(1000, vec![]),
            make_ult(),
        ))
        .id();

    let enemy = app
        .world_mut()
        .spawn((
            make_unit(3, "Training Dummy", 10_000, 10_000, Attribute::Virus),
            Team::Enemy,
            Toughness::new(1000, vec![]),
            make_ult(),
            UnitSkills {
                basic: SkillId("enemy_smash".into()),
                skills: vec![SkillId("enemy_smash".into())],
                ultimate: SkillId("enemy_smash".into()),
                follow_up: None,
            },
        ))
        .id();

    let mut sp_history = Vec::with_capacity(20);

    // Cap-pressure phase: two consecutive capped turns are allowed, but the pool must not
    // remain at cap for a 3-turn window. With max=5, ally_skill_4 (cost 4) drains from cap.
    cast(
        &mut app,
        ActionIntent::Basic {
            attacker: UnitId(1),
            target: UnitId(3),
        },
        &mut sp_history,
    );
    cast(
        &mut app,
        ActionIntent::Basic {
            attacker: UnitId(1),
            target: UnitId(3),
        },
        &mut sp_history,
    );
    cast(
        &mut app,
        ActionIntent::Skill {
            attacker: UnitId(1),
            skill_id: SkillId("ally_skill_4".into()),
            target: UnitId(3),
        },
        &mut sp_history,
    );
    // Rebuild from 1 SP back to cap with 5 basics.
    for _ in 0..5 {
        cast(
            &mut app,
            ActionIntent::Basic {
                attacker: UnitId(1),
                target: UnitId(3),
            },
            &mut sp_history,
        );
    }

    // First revive succeeds at exactly 5 SP, then the enemy KOs the target again so the
    // immediate second revive attempt can fail on insufficient SP rather than target state.
    cast(
        &mut app,
        ActionIntent::Skill {
            attacker: UnitId(1),
            skill_id: SkillId("holy_revive".into()),
            target: UnitId(2),
        },
        &mut sp_history,
    );
    {
        let _turn_order = app.world().resource::<TurnOrder>();
        let victim_unit = app
            .world()
            .get::<Unit>(victim)
            .expect("victim should exist");
        assert!(
            app.world().get::<Ko>(victim).is_none(),
            "victim should be revived after the first successful revive; history={sp_history:?}"
        );
        assert_eq!(
            victim_unit.hp_current, 25,
            "victim hp should be restored to 25 on revive; history={sp_history:?}"
        );
        // In AV system, future_preview is always empty; revived unit re-enters via Ko removal.
        assert!(
            app.world().get::<Ko>(victim).is_none(),
            "revived victim should not have Ko — it will re-enter AV turn order automatically"
        );
        assert_eq!(
            app.world().resource::<SpPool>().current,
            0,
            "first revive should spend the full 5 SP; history={sp_history:?}"
        );
    }
    cast(
        &mut app,
        ActionIntent::Skill {
            attacker: UnitId(3),
            skill_id: SkillId("enemy_smash".into()),
            target: UnitId(2),
        },
        &mut sp_history,
    );
    cast(
        &mut app,
        ActionIntent::Skill {
            attacker: UnitId(1),
            skill_id: SkillId("holy_revive".into()),
            target: UnitId(2),
        },
        &mut sp_history,
    );
    {
        let victim_unit = app
            .world()
            .get::<Unit>(victim)
            .expect("victim should exist");
        assert!(
            app.world().get::<Ko>(victim).is_some(),
            "victim should remain KO after the unaffordable revive; history={sp_history:?}"
        );
        assert!(
            victim_unit.hp_current <= 0,
            "KO target should not regain HP on the failed revive; hp_current={}, history={sp_history:?}",
            victim_unit.hp_current
        );
        assert_eq!(
            app.world().resource::<SpPool>().current,
            0,
            "failed revive should not spend SP; history={sp_history:?}"
        );
    }

    // Rebuild from 0 SP with real basics before the second successful revive.
    for _ in 0..5 {
        cast(
            &mut app,
            ActionIntent::Basic {
                attacker: UnitId(1),
                target: UnitId(3),
            },
            &mut sp_history,
        );
    }

    cast(
        &mut app,
        ActionIntent::Skill {
            attacker: UnitId(1),
            skill_id: SkillId("holy_revive".into()),
            target: UnitId(2),
        },
        &mut sp_history,
    );
    {
        let _turn_order = app.world().resource::<TurnOrder>();
        let victim_unit = app
            .world()
            .get::<Unit>(victim)
            .expect("victim should exist");
        assert!(
            app.world().get::<Ko>(victim).is_none(),
            "victim should be revived after the second successful revive; history={sp_history:?}"
        );
        assert_eq!(
            victim_unit.hp_current, 25,
            "victim hp should be restored to 25 on the second revive; history={sp_history:?}"
        );
        // In AV system, future_preview is always empty; revived unit re-enters via Ko removal.
        assert!(
            app.world().get::<Ko>(victim).is_none(),
            "revived victim should not have Ko — it will re-enter AV turn order automatically"
        );
        assert_eq!(
            app.world().resource::<SpPool>().current,
            0,
            "second revive should spend the full 5 SP; history={sp_history:?}"
        );
    }
    cast(
        &mut app,
        ActionIntent::Skill {
            attacker: UnitId(3),
            skill_id: SkillId("enemy_smash".into()),
            target: UnitId(2),
        },
        &mut sp_history,
    );
    cast(
        &mut app,
        ActionIntent::Basic {
            attacker: UnitId(1),
            target: UnitId(3),
        },
        &mut sp_history,
    );
    cast(
        &mut app,
        ActionIntent::Basic {
            attacker: UnitId(1),
            target: UnitId(3),
        },
        &mut sp_history,
    );

    assert_eq!(
        sp_history.len(),
        20,
        "expected a full 20-turn history, got {sp_history:?}"
    );

    let max_sp = app.world().resource::<SpPool>().max;
    assert!(
        sp_history
            .windows(3)
            .all(|window| !window.iter().all(|&sp| sp == max_sp)),
        "SP pool stayed at cap for a forbidden 3-turn window; history={sp_history:?}"
    );
    assert!(
        sp_history
            .windows(3)
            .any(|window| window[0] == max_sp && window[1] == max_sp && window[2] < max_sp),
        "expected two capped turns to be allowed before a drain; history={sp_history:?}"
    );

    assert_eq!(
        app.world().resource::<SpPool>().current,
        2,
        "expected the final two basics to rebuild SP without affecting revive gating; history={sp_history:?}"
    );

    let _ = ally;
    let _ = enemy;
}
