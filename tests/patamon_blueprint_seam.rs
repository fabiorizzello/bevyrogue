use bevy::{ecs::message::MessageCursor, prelude::*};
use bevyrogue::combat::blueprints::{self, CustomSignalDispatchError};
use bevyrogue::combat::events::{CombatEvent, CombatEventKind};
use bevyrogue::combat::blueprints::patamon::{HolySupportState, HolySupportTransition};
use bevyrogue::combat::kernel::{CombatKernelTransition, register_combat_kernel_runtime};
use bevyrogue::combat::kit::UnitSkills;
use bevyrogue::combat::log::ActionLog;
use bevyrogue::combat::observability::{capture_validation_snapshot, format_validation_snapshot};
use bevyrogue::combat::resolution::resolve_action;
use bevyrogue::combat::sp::SpPool;
use bevyrogue::combat::state::{CombatState, ResolvedAction, UltEffect};
use bevyrogue::combat::team::Team;
use bevyrogue::combat::toughness::Toughness;
use bevyrogue::combat::turn_order::TurnOrder;
use bevyrogue::combat::turn_system::{ActionIntent, resolve_action_system};
use bevyrogue::combat::types::{Attribute, DamageTag, EvoStage, SkillId, UnitId};
use bevyrogue::combat::ultimate::{UltAccumulationTrigger, UltimateCharge};
use bevyrogue::combat::unit::Unit;
use bevyrogue::data::SkillBookHandle;
use bevyrogue::data::skills_ron::{
    CustomSignalPayload, Effect, SelfTargetRule, SkillBook, SkillCustomSignal, SkillDef,
    SkillImplementation, SkillTargeting, TargetLife, TargetShape, TargetSide,
};

fn canonical_skill_book() -> SkillBook {
    ron::from_str(include_str!("../assets/data/skills.ron")).expect("parse skills.ron")
}

fn find_skill<'a>(book: &'a SkillBook, id: &str) -> &'a SkillDef {
    book.0
        .iter()
        .find(|skill| skill.id == SkillId(id.into()))
        .unwrap_or_else(|| panic!("{id} in canonical skill book"))
}

fn no_signal_skill_ron() -> &'static str {
    r#"(
        id: SkillId("plain_patamon_test_skill"),
        name: "Plain Patamon Test Skill",
        damage_tag: Light,
        sp_cost: 0,
        targeting: SkillTargeting(
            shape: Single,
            side: Enemy,
            life: Alive,
            self_rule: Forbid,
            target_hp_rule: Any,
        ),
        implementation: Implemented,
        effects: [Damage(amount: 1, target: Single)],
    )"#
}

fn signal(owner: &str, signal: &str, amount: i32) -> SkillCustomSignal {
    SkillCustomSignal::blueprint(owner, signal, CustomSignalPayload::Amount { amount })
}

fn unit(id: u32, hp_current: i32, attribute: Attribute) -> Unit {
    Unit {
        id: UnitId(id),
        name: format!("Unit{id}"),
        hp_max: 100,
        hp_current,
        attribute,
        resists: vec![],
        evo_stage: EvoStage::Child,
    }
}

fn ultimate_charge(current: i32) -> UltimateCharge {
    UltimateCharge {
        current,
        trigger: 80,
        cap: 150,
        trigger_type: UltAccumulationTrigger::OnBasicAttack,
        charge_per_event: 20,
    }
}

fn cursor(app: &mut App) -> MessageCursor<CombatEvent> {
    app.world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor()
}

fn drain(cursor: &mut MessageCursor<CombatEvent>, app: &App) -> Vec<CombatEvent> {
    let messages = app.world().resource::<Messages<CombatEvent>>();
    cursor.read(messages).cloned().collect()
}

fn app_with_canonical_skills() -> App {
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
    let handle = assets.add(canonical_skill_book());
    app.insert_resource(assets);
    app.insert_resource(SkillBookHandle(handle));
    app.world_mut().resource_mut::<SpPool>().current = 5;

    app.world_mut().spawn((
        unit(9, 88, Attribute::Vaccine),
        Team::Ally,
        Toughness::new(44, vec![DamageTag::Dark]),
        ultimate_charge(80),
        UnitSkills {
            basic: SkillId("holy_breeze".into()),
            skills: vec![
                SkillId("holy_breeze".into()),
                SkillId("patamon_revive".into()),
            ],
            ultimate: SkillId("patamon_ult".into()),
            follow_up: None,
        },
    ));
    app.world_mut().spawn((
        unit(101, 120, Attribute::Virus),
        Team::Enemy,
        Toughness::new(35, vec![DamageTag::Light]),
        ultimate_charge(0),
        UnitSkills {
            basic: SkillId("enemy_skill_fire".into()),
            skills: vec![SkillId("enemy_skill_fire".into())],
            ultimate: SkillId("enemy_ult_fire".into()),
            follow_up: None,
        },
    ));

    app
}

#[test]
fn custom_signal_missing_field_defaults_to_empty() {
    let skill: SkillDef =
        ron::from_str(no_signal_skill_ron()).expect("missing custom signals default");

    assert!(skill.custom_signals.is_empty());
}

#[test]
fn custom_signal_rejects_unknown_patamon_variant() {
    let malformed = no_signal_skill_ron().replace(
        "effects: [Damage(amount: 1, target: Single)],",
        "effects: [Damage(amount: 1, target: Single)],\n        custom_signals: [(owner: \"patamon\", signal: \"unknown_signal\")],",
    );

    let skill: SkillDef = ron::from_str(&malformed).expect("generic custom signal parses");
    let err = blueprints::dispatch_custom_signal(
        &skill.custom_signals[0],
        &ResolvedAction {
            source: UnitId(1),
            target: UnitId(2),
            skill_id: skill.id.clone(),
            damage_tag: DamageTag::Light,
            base_damage: 1,
            toughness_damage: 0,
            revive_pct: 0,
            heal_pct: 0,
            sp_cost: 0,
            ult_effect: UltEffect::None,
            grant_free_skill_count: 0,
            status_to_apply: None,
            advance_pct: 0,
        delay_pct: 0,
            energy_grant: 0,
            self_advance_pct: 0,
            target_shape: TargetShape::Single,
            custom_signals: skill.custom_signals.clone(),
            damage_curve: Default::default(),
            cleanse_count: None,
        },
    )
    .expect_err("unknown custom signal rejected");

    assert_eq!(
        err,
        CustomSignalDispatchError::UnknownSignal {
            owner: "patamon".into(),
            signal: "unknown_signal".into(),
        }
    );
}

#[test]
fn custom_signal_parses_from_tracked_patamon_skill() {
    let book = canonical_skill_book();
    let patamon_ult = find_skill(&book, "patamon_ult");

    assert_eq!(
        patamon_ult.custom_signals,
        vec![signal("patamon", "build_holy_support_grace", 1)]
    );

    let holy_breeze = find_skill(&book, "holy_breeze");
    assert!(
        holy_breeze.custom_signals.is_empty(),
        "only the seeded Patamon skill should declare a custom signal in this task"
    );
}

#[test]
fn custom_signal_resolved_action_carries_metadata_without_interpreting_it() {
    let skill = SkillDef {
        id: SkillId("patamon_signal_test".into()),
        name: "Patamon Signal Test".into(),
        damage_tag: DamageTag::Light,
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
            amount: 7,
            target: TargetShape::Single,
            per_hop: Default::default(),
        }],
        custom_signals: vec![signal("patamon", "build_holy_support_grace", 1)],
        ..Default::default()
    };
    let book = SkillBook(vec![skill.clone()]);
    let kit = UnitSkills {
        basic: SkillId("unused_basic".into()),
        skills: vec![skill.id.clone()],
        ultimate: SkillId("unused_ult".into()),
        follow_up: None,
    };
    let intent = ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: skill.id.clone(),
        target: UnitId(2),
    };

    let resolved = resolve_action(&intent, &kit, Some(&book)).expect("skill resolves");

    assert_eq!(resolved.custom_signals, skill.custom_signals);
    assert_eq!(
        resolved.base_damage, 7,
        "normal effect metadata remains intact"
    );
}

#[test]
fn no_signal_action_produces_no_patamon_blueprint_transitions() {
    let book = canonical_skill_book();
    let skill = find_skill(&book, "holy_breeze");
    let kit = UnitSkills {
        basic: skill.id.clone(),
        skills: vec![skill.id.clone()],
        ultimate: SkillId("patamon_ult".into()),
        follow_up: None,
    };
    let intent = ActionIntent::Skill {
        attacker: UnitId(9),
        skill_id: skill.id.clone(),
        target: UnitId(101),
    };
    let resolved = resolve_action(&intent, &kit, Some(&book)).expect("plain skill resolves");

    assert!(bevyrogue::combat::blueprints::transitions_for_action(&resolved).is_empty());
}

#[test]
fn patamon_signal_maps_to_expected_holy_support_transition() {
    let book = canonical_skill_book();
    let skill = find_skill(&book, "patamon_ult");
    let kit = UnitSkills {
        basic: SkillId("holy_breeze".into()),
        skills: vec![],
        ultimate: skill.id.clone(),
        follow_up: None,
    };
    let intent = ActionIntent::Ultimate {
        attacker: UnitId(9),
        target: UnitId(101),
    };
    let resolved = resolve_action(&intent, &kit, Some(&book)).expect("patamon ult resolves");

    let transitions = bevyrogue::combat::blueprints::transitions_for_action(&resolved);

    assert_eq!(
        transitions,
        vec![CombatKernelTransition::HolySupport(
            HolySupportTransition::build_grace(1)
        )]
    );
}

#[test]
fn patamon_ultimate_dispatches_blueprint_transition_into_holy_support_state_and_snapshot() {
    let mut app = app_with_canonical_skills();
    let mut reader = cursor(&mut app);

    app.world_mut().write_message(ActionIntent::Ultimate {
        attacker: UnitId(9),
        target: UnitId(101),
    });
    app.update();
    app.update();
    let events = drain(&mut reader, &app);

    let holy_transitions: Vec<_> = events
        .iter()
        .filter_map(|event| match &event.kind {
            CombatEventKind::OnKernelTransition {
                transition: CombatKernelTransition::HolySupport(transition),
            } => Some(*transition),
            _ => None,
        })
        .collect();
    assert_eq!(
        holy_transitions,
        vec![HolySupportTransition::build_grace(1)]
    );

    let state = app.world().resource::<HolySupportState>();
    assert_eq!(state.grace, 1);
    assert_eq!(
        state.last_signal,
        Some(HolySupportTransition::build_grace(1))
    );

    let snapshot = capture_validation_snapshot(app.world_mut()).expect("snapshot");
    let formatted = format_validation_snapshot(&snapshot);
    assert!(formatted.contains("holy_support=grace=1/3"));
    assert!(formatted.contains("last=build(1)"));
}
