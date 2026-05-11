use bevy::{ecs::message::MessageCursor, prelude::*};
use bevyrogue::combat::{
    events::{CombatEvent, CombatEventKind},
    kit::UnitSkills,
    log::ActionLog,
    sp::SpPool,
    speed::SpeedModifier,
    state::CombatState,
    status_effect::{StatusEffect, StatusEffectKind},
    stun::Stunned,
    team::Team,
    toughness::Toughness,
    turn_order::{TurnAdvanced, TurnOrder},
    turn_system::{ActionIntent, advance_turn_system, resolve_action_system},
    types::{Attribute, DamageTag, EvoStage, SkillId, UnitId},
    ultimate::{UltAccumulationTrigger, UltimateCharge},
    unit::Unit,
};
use bevyrogue::data::{
    SkillBookHandle,
    skills_ron::{
        Effect, LegalityReasonCode, SelfTargetRule, SkillBook, SkillDef, SkillImplementation,
        SkillTargeting, TargetLife, TargetShape, TargetSide,
    },
};

fn unit(id: u32, hp_current: i32) -> Unit {
    Unit {
        id: UnitId(id),
        name: format!("Unit{id}"),
        hp_max: hp_current,
        hp_current,
        attribute: Attribute::Vaccine,
        resists: vec![],
        evo_stage: EvoStage::Adult,
    }
}

fn turn_app() -> App {
    let mut app = App::new();
    app.init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .init_resource::<Time>()
        .add_message::<TurnAdvanced>()
        .add_message::<bevyrogue::combat::av::ActionValueUpdated>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_systems(Update, advance_turn_system);
    app
}

fn action_app(skill_book: SkillBook) -> App {
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
        .add_message::<TurnAdvanced>()
        .add_message::<bevyrogue::combat::av::ActionValueUpdated>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_systems(Update, (advance_turn_system, resolve_action_system).chain());
    app
}

fn read_messages<T: Message + Clone>(app: &mut App) -> Vec<T> {
    let mut cursor: MessageCursor<T> = app.world_mut().resource_mut::<Messages<T>>().get_cursor();
    cursor
        .read(app.world().resource::<Messages<T>>())
        .cloned()
        .collect()
}

fn status_events(app: &mut App) -> Vec<CombatEvent> {
    read_messages::<CombatEvent>(app)
}

fn action_intents(app: &mut App) -> Vec<ActionIntent> {
    read_messages::<ActionIntent>(app)
}

fn basic_skill() -> SkillDef {
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
        effects: vec![
            Effect::Damage {
                amount: 10,
                target: TargetShape::Single,
            },
            Effect::ToughnessHit(5),
        ],
        custom_signals: vec![],
        animation_sequence: None,
        qte: None,
    }
}

fn basic_unit_skills() -> UnitSkills {
    UnitSkills {
        basic: SkillId("basic".into()),
        skills: vec![],
        ultimate: SkillId("ult".into()),
        follow_up: None,
    }
}

fn combat_event_matches_tick(
    event: &CombatEvent,
    kind: &StatusEffectKind,
    turns_left: u32,
) -> bool {
    matches!(
        &event.kind,
        CombatEventKind::OnStatusTick {
            kind: tick_kind,
            turns_left: tick_turns_left,
        } if tick_kind == kind && *tick_turns_left == turns_left
    )
}

fn combat_event_matches_expired(event: &CombatEvent, kind: &StatusEffectKind) -> bool {
    matches!(
        &event.kind,
        CombatEventKind::OnStatusExpired { kind: expired_kind } if expired_kind == kind
    )
}

#[test]
fn burn_ticks_hp_and_expires() {
    let mut app = turn_app();
    let entity = app
        .world_mut()
        .spawn((
            unit(1, 30),
            Team::Ally,
            StatusEffect {
                kind: StatusEffectKind::Burn { damage_per_turn: 5 },
                duration_remaining: 3,
            },
        ))
        .id();

    let expected_kind = StatusEffectKind::Burn { damage_per_turn: 5 };
    let expected_hps = [25, 20, 15];
    let expected_turns_left = [2_u32, 1, 0];

    for ((hp_expected, turns_left), turn) in
        expected_hps.into_iter().zip(expected_turns_left).zip(0..3)
    {
        app.world_mut().write_message(TurnAdvanced::of(UnitId(1)));
        app.update();

        assert_eq!(
            app.world().get::<Unit>(entity).unwrap().hp_current,
            hp_expected,
            "burn turn {turn} should subtract 5 HP"
        );

        let events = status_events(&mut app);
        assert!(
            events
                .iter()
                .any(|event| combat_event_matches_tick(event, &expected_kind, turns_left)),
            "burn turn {turn} should emit OnStatusTick with turns_left={turns_left}"
        );

        if turns_left == 0 {
            assert!(app.world().get::<StatusEffect>(entity).is_none());
            assert!(
                events
                    .iter()
                    .any(|event| combat_event_matches_expired(event, &expected_kind)),
                "burn expiry should emit OnStatusExpired"
            );
        } else {
            assert!(app.world().get::<StatusEffect>(entity).is_some());
        }
    }
}

#[test]
fn freeze_reduces_effective_speed_and_expires() {
    let mut app = turn_app();
    let entity = app
        .world_mut()
        .spawn((
            unit(1, 40),
            Team::Ally,
            StatusEffect {
                kind: StatusEffectKind::Freeze { speed_reduction: 3 },
                duration_remaining: 2,
            },
        ))
        .id();

    let expected_kind = StatusEffectKind::Freeze { speed_reduction: 3 };

    app.world_mut().write_message(TurnAdvanced::of(UnitId(1)));
    app.update();

    assert_eq!(
        app.world()
            .get::<SpeedModifier>(entity)
            .map(|modifier| modifier.0),
        Some(-3),
        "freeze should attach a -3 speed modifier while active"
    );
    let events = status_events(&mut app);
    assert!(
        events
            .iter()
            .any(|event| combat_event_matches_tick(event, &expected_kind, 1)),
        "freeze should emit OnStatusTick with turns_left=1 after the first tick"
    );

    app.world_mut().write_message(TurnAdvanced::of(UnitId(1)));
    app.update();

    assert!(
        app.world().get::<SpeedModifier>(entity).is_none(),
        "freeze should remove the speed modifier on expiry"
    );
    assert!(app.world().get::<StatusEffect>(entity).is_none());
    let events = status_events(&mut app);
    assert!(
        events
            .iter()
            .any(|event| combat_event_matches_tick(event, &expected_kind, 0)),
        "freeze expiry turn should emit OnStatusTick with turns_left=0"
    );
    assert!(
        events
            .iter()
            .any(|event| combat_event_matches_expired(event, &expected_kind)),
        "freeze expiry should emit OnStatusExpired"
    );
}

#[test]
fn shock_cancels_action_at_100pct() {
    let mut app = turn_app();
    let entity = app
        .world_mut()
        .spawn((
            unit(1, 50),
            Team::Enemy,
            UnitSkills {
                basic: SkillId("basic".into()),
                skills: vec![],
                ultimate: SkillId("ult".into()),
                follow_up: None,
            },
            UltimateCharge::new(100, 150, UltAccumulationTrigger::OnBasicAttack, 25),
            Toughness::new(100, vec![]),
            StatusEffect {
                kind: StatusEffectKind::Shock {
                    cancel_chance_pct: 100,
                },
                duration_remaining: 2,
            },
        ))
        .id();
    app.world_mut()
        .spawn((unit(2, 50), Team::Ally, Toughness::new(100, vec![])));

    let expected_kind = StatusEffectKind::Shock {
        cancel_chance_pct: 100,
    };

    app.world_mut().write_message(TurnAdvanced::of(UnitId(1)));
    app.update();

    assert!(
        action_intents(&mut app).is_empty(),
        "100% shock should not emit an ActionIntent"
    );
    let events = status_events(&mut app);
    assert!(
        events.iter().any(|event| matches!(
            &event.kind,
            CombatEventKind::OnActionFailed { reason } if reason == "Shock"
        )),
        "100% shock should emit OnActionFailed with Shock reason"
    );
    assert!(
        events
            .iter()
            .any(|event| combat_event_matches_tick(event, &expected_kind, 1)),
        "100% shock should still tick down its duration"
    );
    assert!(
        events
            .iter()
            .all(|event| !matches!(event.kind, CombatEventKind::OnDamageDealt { .. })),
        "100% shock should not resolve any damage"
    );
    assert!(app.world().get::<StatusEffect>(entity).is_some());

    app.world_mut().write_message(TurnAdvanced::of(UnitId(1)));
    app.update();

    let events = status_events(&mut app);
    assert!(
        events
            .iter()
            .any(|event| combat_event_matches_tick(event, &expected_kind, 0)),
        "100% shock expiry turn should emit OnStatusTick with turns_left=0"
    );
    assert!(
        events
            .iter()
            .any(|event| combat_event_matches_expired(event, &expected_kind)),
        "100% shock should emit OnStatusExpired when duration reaches zero"
    );
    assert!(app.world().get::<StatusEffect>(entity).is_none());
}

#[test]
fn shock_never_cancels_at_0pct() {
    let mut app = action_app(SkillBook(vec![basic_skill()]));
    let attacker = app
        .world_mut()
        .spawn((
            unit(1, 50),
            Team::Enemy,
            basic_unit_skills(),
            UltimateCharge::new(100, 150, UltAccumulationTrigger::OnBasicAttack, 25),
            Toughness::new(100, vec![]),
            StatusEffect {
                kind: StatusEffectKind::Shock {
                    cancel_chance_pct: 0,
                },
                duration_remaining: 2,
            },
        ))
        .id();
    app.world_mut()
        .spawn((unit(2, 50), Team::Ally, Toughness::new(100, vec![])));
    app.world_mut()
        .resource_mut::<TurnOrder>()
        .seed([UnitId(1), UnitId(2)]);

    app.world_mut().write_message(TurnAdvanced::of(UnitId(1)));
    app.update();

    assert!(
        action_intents(&mut app).iter().any(|intent| matches!(
            intent,
            ActionIntent::Basic {
                attacker: UnitId(1),
                target: UnitId(2)
            }
        )),
        "0% shock should still emit an ActionIntent"
    );

    let events = status_events(&mut app);
    assert!(
        events.iter().any(|event| matches!(event.kind, CombatEventKind::OnDamageDealt { amount, .. } if amount > 0)),
        "0% shock should resolve into a damage event"
    );
    assert!(
        events
            .iter()
            .all(|event| !matches!(&event.kind, CombatEventKind::OnActionFailed { .. })),
        "0% shock should not cancel the action"
    );

    // Sanity check: the attacker entity remains present and the status still has one turn left.
    assert!(app.world().get::<Unit>(attacker).is_some());
    assert_eq!(
        app.world()
            .get::<StatusEffect>(attacker)
            .map(|effect| effect.duration_remaining),
        Some(1)
    );
}

#[test]
fn stunned_unit_does_not_tick_status() {
    let mut app = turn_app();
    let entity = app
        .world_mut()
        .spawn((
            unit(1, 42),
            Team::Ally,
            Stunned { turns_left: 1 },
            StatusEffect {
                kind: StatusEffectKind::Burn { damage_per_turn: 9 },
                duration_remaining: 2,
            },
        ))
        .id();

    app.world_mut().write_message(TurnAdvanced::of(UnitId(1)));
    app.update();

    assert_eq!(app.world().get::<Unit>(entity).unwrap().hp_current, 42);
    assert_eq!(
        app.world().get::<StatusEffect>(entity),
        Some(&StatusEffect {
            kind: StatusEffectKind::Burn { damage_per_turn: 9 },
            duration_remaining: 2,
        })
    );
    assert!(app.world().get::<Stunned>(entity).is_none());
    let events = status_events(&mut app);
    assert!(
        events.iter().all(|event| !matches!(
            event.kind,
            CombatEventKind::OnStatusTick { .. } | CombatEventKind::OnStatusExpired { .. }
        )),
        "stunned units should skip status ticking entirely"
    );
}
