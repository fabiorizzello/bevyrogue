use bevy::{ecs::message::MessageCursor, prelude::*};
use bevyrogue::combat::{
    av::ActionValueUpdated,
    energy::Energy,
    events::{CombatEvent, CombatEventKind},
    follow_up::{
        FollowUpIntent, FollowUpTrace, follow_up_listener_system, form_identity_listener_system,
        resolve_follow_up_action_system,
    },
    kit::{FormIdentityKit, UnitSkills},
    log::ActionLog,
    round_flags::RoundFlags,
    sp::{RoundSpTracker, SpPool},
    state::{CombatPhase, CombatState},
    team::Team,
    toughness::Toughness,
    turn_order::TurnOrder,
    turn_system::{ActionIntent, advance_turn_system, resolve_action_system},
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

fn build_app() -> App {
    let mut app = App::new();
    let mut assets = Assets::<SkillBook>::default();
    let book = SkillBook(vec![
        SkillDef {
            id: SkillId("basic".into()),
            name: "Basic".into(),
            damage_tag: DamageTag::Physical,
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
                    amount: 5,
                    target: TargetShape::Single,
                    per_hop: Default::default(),
                },
                Effect::ToughnessHit(1),
            ],

            custom_signals: vec![],
            animation_sequence: None,
            qte: None,
            timeline: None,
        },
        SkillDef {
            id: SkillId("skill".into()),
            name: "Skill".into(),
            damage_tag: DamageTag::Physical,
            sp_cost: 3,
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
                Effect::ToughnessHit(1),
            ],

            custom_signals: vec![],
            animation_sequence: None,
            qte: None,
            timeline: None,
        },
        SkillDef {
            id: SkillId("grant_energy".into()),
            name: "Grant Energy".into(),
            damage_tag: DamageTag::Physical,
            sp_cost: 0,
            targeting: SkillTargeting {
                shape: TargetShape::Single,
                side: TargetSide::Enemy,
                life: TargetLife::Alive,
                self_rule: SelfTargetRule::Forbid,
                ..Default::default()
            },
            implementation: SkillImplementation::Implemented,
            legacy_ops: vec![Effect::GrantEnergy(15)],

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
        .insert_resource(SpPool { current: 3, max: 5 })
        .init_resource::<ActionLog>()
        .init_resource::<Time>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_message::<bevyrogue::combat::turn_order::TurnAdvanced>()
        .add_message::<ActionValueUpdated>()
        .add_systems(Update, (resolve_action_system, advance_turn_system));
    app
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

fn cast(app: &mut App, intent: ActionIntent) {
    app.world_mut().write_message(intent);
    app.update();
}

fn drain_events(cursor: &mut MessageCursor<CombatEvent>, app: &App) -> Vec<CombatEvent> {
    cursor
        .read(app.world().resource::<Messages<CombatEvent>>())
        .cloned()
        .collect()
}

fn energy_gain_amounts(events: &[CombatEvent]) -> Vec<i32> {
    events
        .iter()
        .filter_map(|event| match &event.kind {
            CombatEventKind::EnergyGained { amount, .. } => Some(*amount),
            _ => None,
        })
        .collect()
}

fn load_roster() -> bevyrogue::data::units_ron::UnitRoster {
    bevyrogue::data::aggregate_unit_roster()
}

fn load_skill_book() -> SkillBook {
    bevyrogue::data::aggregate_skill_book()
}

fn pilot(
    roster: &bevyrogue::data::units_ron::UnitRoster,
    name: &str,
) -> bevyrogue::data::units_ron::UnitDef {
    roster
        .0
        .iter()
        .find(|unit| unit.name == name)
        .cloned()
        .unwrap_or_else(|| panic!("missing pilot {name}"))
}

fn spawn_greymon(app: &mut App, def: &bevyrogue::data::units_ron::UnitDef) -> Entity {
    let fi_config = def
        .form_identity
        .clone()
        .expect("Greymon must have form_identity");

    app.world_mut()
        .spawn((
            Unit {
                id: def.id,
                name: def.name.clone(),
                hp_max: def.hp_max,
                hp_current: def.hp_max,
                attribute: def.attribute,
                resists: def.resists.clone(),
                evo_stage: EvoStage::Adult,
            },
            def.team,
            UnitSkills {
                basic: def.basic_skill.clone(),
                skills: def.skill_ids.clone(),
                ultimate: def.ultimate_skill.clone(),
                follow_up: def.follow_up.clone(),
            },
            FormIdentityKit { config: fi_config },
            Toughness {
                max: def.toughness_max,
                current: def.toughness_max,
                weaknesses: def.weaknesses.clone(),
                broken: false,
                category: Default::default(),
            },
            UltimateCharge {
                current: 0,
                trigger: def.ultimate_trigger,
                cap: def.ultimate_cap,
                trigger_type: def.ultimate_accumulation_trigger,
                charge_per_event: def.ultimate_charge_per_event,
            },
            Energy::default(),
            RoundFlags::default(),
        ))
        .id()
}

fn spawn_tough_enemy(app: &mut App, id: UnitId) -> Entity {
    app.world_mut()
        .spawn((
            Unit {
                id,
                name: "Boss".into(),
                hp_max: 1000,
                hp_current: 1000,
                attribute: Attribute::Virus,
                resists: vec![],
                evo_stage: EvoStage::Adult,
            },
            Team::Enemy,
            Toughness {
                max: 200,
                current: 200,
                weaknesses: vec![DamageTag::Fire],
                broken: false,
                category: Default::default(),
            },
            UltimateCharge {
                current: 0,
                trigger: 100,
                cap: 150,
                trigger_type: UltAccumulationTrigger::OnBasicAttack,
                charge_per_event: 25,
            },
            UnitSkills {
                basic: SkillId("dummy".into()),
                skills: vec![SkillId("dummy".into())],
                ultimate: SkillId("dummy_ult".into()),
                follow_up: None,
            },
        ))
        .id()
}

fn unit_energy(app: &mut App, unit_id: UnitId) -> i32 {
    let mut q = app.world_mut().query::<(&Unit, &Energy)>();
    q.iter(app.world())
        .find(|(u, _)| u.id == unit_id)
        .map(|(_, e)| e.current)
        .unwrap_or_else(|| panic!("missing unit {:?}", unit_id))
}

fn setup_form_identity_app(skill_book: SkillBook) -> App {
    let mut app = App::new();
    app.init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .init_resource::<SpPool>()
        .init_resource::<ActionLog>()
        .init_resource::<Time>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_message::<FollowUpIntent>()
        .add_message::<FollowUpTrace>()
        .add_systems(
            Update,
            (
                resolve_action_system,
                follow_up_listener_system,
                form_identity_listener_system,
                resolve_follow_up_action_system,
            )
                .chain(),
        );

    let mut assets = Assets::<SkillBook>::default();
    let handle = assets.add(skill_book);
    app.insert_resource(assets);
    app.insert_resource(SkillBookHandle(handle));
    app.world_mut().resource_mut::<SpPool>().current = 999;
    app
}

/// Proves RoundSpTracker enforces the non-Basic SP cap (+2/round), and that
/// resetting the tracker restores the full budget for the next round.
#[test]
fn sp_non_basic_cap_enforced() {
    let mut tracker = RoundSpTracker::default();
    let mut pool = SpPool { current: 0, max: 5 };

    // Three +1 non-Basic gains: only the first two should be applied (cap = 2/round).
    let g1 = tracker.try_gain_non_basic(1);
    pool.gain(g1);
    let g2 = tracker.try_gain_non_basic(1);
    pool.gain(g2);
    let g3 = tracker.try_gain_non_basic(1);
    pool.gain(g3);

    assert_eq!(g1, 1, "first non-basic gain should succeed");
    assert_eq!(g2, 1, "second non-basic gain should succeed");
    assert_eq!(g3, 0, "third non-basic gain blocked by cap");
    assert_eq!(pool.current, 2, "pool should hold exactly 2 after cap");

    // Reset restores the budget for a new round.
    tracker.reset();
    let g4 = tracker.try_gain_non_basic(1);
    pool.gain(g4);
    let g5 = tracker.try_gain_non_basic(1);
    pool.gain(g5);

    assert_eq!(g4, 1, "first gain after reset should succeed");
    assert_eq!(g5, 1, "second gain after reset should succeed");
    assert_eq!(pool.current, 4, "pool should hold 4 after second batch");
}

#[test]
fn energy_grant_accumulates_without_round_cap() {
    let mut app = build_app();

    let attacker = app
        .world_mut()
        .spawn((
            Unit {
                id: UnitId(10),
                name: "EnergyUser".into(),
                hp_max: 100,
                hp_current: 100,
                attribute: Attribute::Vaccine,
                resists: vec![],
                evo_stage: EvoStage::Adult,
            },
            Team::Ally,
            Toughness::new(1000, vec![]),
            make_ult(),
            UnitSkills {
                basic: SkillId("basic".into()),
                skills: vec![SkillId("grant_energy".into())],
                ultimate: SkillId("basic".into()),
                follow_up: None,
            },
            Energy {
                current: 0,
                max: 50,
            },
        ))
        .id();

    let _target = app
        .world_mut()
        .spawn((
            Unit {
                id: UnitId(11),
                name: "Target".into(),
                hp_max: 100,
                hp_current: 100,
                attribute: Attribute::Virus,
                resists: vec![],
                evo_stage: EvoStage::Adult,
            },
            Team::Enemy,
            Toughness::new(1000, vec![]),
        ))
        .id();

    let mut cursor = app
        .world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor();

    cast(
        &mut app,
        ActionIntent::Skill {
            attacker: UnitId(10),
            skill_id: SkillId("grant_energy".into()),
            target: UnitId(11),
        },
    );

    let first_events = drain_events(&mut cursor, &app);
    assert_eq!(
        energy_gain_amounts(&first_events),
        vec![15],
        "first grant should emit the full GrantEnergy(15) amount"
    );
    assert_eq!(
        app.world().get::<Energy>(attacker).unwrap().current,
        15,
        "first grant should apply the full 15 Energy"
    );

    cast(
        &mut app,
        ActionIntent::Skill {
            attacker: UnitId(10),
            skill_id: SkillId("grant_energy".into()),
            target: UnitId(11),
        },
    );

    let second_events = drain_events(&mut cursor, &app);
    assert_eq!(
        energy_gain_amounts(&second_events),
        vec![15],
        "a second same-round grant is no longer blocked by any per-round cap"
    );
    assert_eq!(
        app.world().get::<Energy>(attacker).unwrap().current,
        30,
        "same-round grants accumulate freely up to Energy.max"
    );
}

#[test]
fn energy_grant_truthfully_clips_at_energy_max() {
    let mut app = build_app();

    let attacker = app
        .world_mut()
        .spawn((
            Unit {
                id: UnitId(20),
                name: "NearMaxEnergyUser".into(),
                hp_max: 100,
                hp_current: 100,
                attribute: Attribute::Vaccine,
                resists: vec![],
                evo_stage: EvoStage::Adult,
            },
            Team::Ally,
            Toughness::new(1000, vec![]),
            make_ult(),
            UnitSkills {
                basic: SkillId("basic".into()),
                skills: vec![SkillId("grant_energy".into())],
                ultimate: SkillId("basic".into()),
                follow_up: None,
            },
            Energy {
                current: 8,
                max: 12,
            },
        ))
        .id();

    let _target = app
        .world_mut()
        .spawn((
            Unit {
                id: UnitId(21),
                name: "Target".into(),
                hp_max: 100,
                hp_current: 100,
                attribute: Attribute::Virus,
                resists: vec![],
                evo_stage: EvoStage::Adult,
            },
            Team::Enemy,
            Toughness::new(1000, vec![]),
        ))
        .id();

    let mut cursor = app
        .world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor();

    cast(
        &mut app,
        ActionIntent::Skill {
            attacker: UnitId(20),
            skill_id: SkillId("grant_energy".into()),
            target: UnitId(21),
        },
    );

    let events = drain_events(&mut cursor, &app);
    let energy_amounts = energy_gain_amounts(&events);
    assert_eq!(
        energy_amounts,
        vec![4],
        "event should report actual Energy gained after max clipping"
    );

    let attacker_energy = app.world().get::<Energy>(attacker).unwrap();
    assert_eq!(
        attacker_energy.current, 12,
        "Energy should clamp to max after grant"
    );
}

#[test]
fn round_flags_reset_on_turn_start() {
    let mut app = build_app();

    let actor = app
        .world_mut()
        .spawn((
            Unit {
                id: UnitId(30),
                name: "TurnActor".into(),
                hp_max: 100,
                hp_current: 100,
                attribute: Attribute::Vaccine,
                resists: vec![],
                evo_stage: EvoStage::Adult,
            },
            Team::Ally,
            RoundFlags {
                break_sealed: true,
                form_identity_used: true,
                hits_received_this_round: 0,
                acted_this_turn: false,
                acted_last_turn: false,
            },
        ))
        .id();

    app.world_mut().resource_mut::<CombatState>().phase = CombatPhase::WaitingAction;
    app.world_mut()
        .write_message(bevyrogue::combat::turn_order::TurnAdvanced::of(UnitId(30)));
    app.update();

    let flags = app.world().get::<RoundFlags>(actor).unwrap();
    assert!(
        !flags.break_sealed,
        "round flags should reset at turn start"
    );
    assert!(
        !flags.form_identity_used,
        "round flags should reset at turn start"
    );
}

#[test]
fn canonical_form_identity_energy_accumulates_without_round_cap() {
    let roster = load_roster();
    let mut app = setup_form_identity_app(load_skill_book());

    let greymon = pilot(&roster, "Greymon");
    let enemy_id = UnitId(99);

    let greymon_entity = spawn_greymon(&mut app, &greymon);
    spawn_tough_enemy(&mut app, enemy_id);

    let fire_skill = greymon.skill_ids[0].clone();

    // Each Form Identity trigger grants the canonical +5 Energy. The per-round cap is gone,
    // so repeatedly re-triggering (by clearing the listener guard) keeps accumulating +5 with
    // no ceiling other than Energy.max.
    for expected in [5, 10, 15, 20] {
        app.world_mut()
            .get_mut::<RoundFlags>(greymon_entity)
            .unwrap()
            .form_identity_used = false;
        app.world_mut().write_message(ActionIntent::Skill {
            attacker: greymon.id,
            skill_id: fire_skill.clone(),
            target: enemy_id,
        });
        app.update();
        assert_eq!(
            unit_energy(&mut app, greymon.id),
            expected,
            "each Form Identity grant adds +5 with no per-round cap"
        );
    }
}
