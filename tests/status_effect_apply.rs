/// Integration tests for M008/S04: wiring ApplyStatus through resolve_action / apply_effects.
///
/// Tests in this file exercise:
/// - Casting a skill with ApplyStatus inserts StatusEffect on the defender
/// - OnStatusApplied event is emitted on successful application
/// - Negative path: action on KO'd defender produces no StatusEffect, no OnStatusApplied
/// - Negative path: action that KOs the defender produces no StatusEffect (KO guard)
use bevy::{ecs::message::MessageCursor, prelude::*};
use bevyrogue::combat::{
    StatusEffect, StatusEffectKind,
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
    unit::{Ko, Unit},
};
use bevyrogue::data::{
    SkillBookHandle,
    skills_ron::{
        Effect, LegalityReasonCode, SelfTargetRule, SkillBook, SkillDef, SkillImplementation,
        SkillTargeting, TargetLife, TargetShape, TargetSide,
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

fn ult_charge_default() -> UltimateCharge {
    UltimateCharge {
        current: 0,
        trigger: 100,
        cap: 150,
        trigger_type: UltAccumulationTrigger::OnBasicAttack,
        charge_per_event: 10,
    }
}

fn flame_bite_skill() -> SkillDef {
    SkillDef {
        id: SkillId("flame_bite".into()),
        name: "Flame Bite".into(),
        damage_tag: DamageTag::Fire,
        sp_cost: 4,
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
                amount: 15,
                target: TargetShape::Single,
            },
            Effect::ToughnessHit(8),
            Effect::ApplyStatus {
                kind: StatusEffectKind::Burn { damage_per_turn: 5 },
                duration: 3,
            },
        ],
        animation_sequence: None,
        qte: None,
        custom_signals: vec![],
    }
}

fn attacker_skills() -> UnitSkills {
    UnitSkills {
        basic: SkillId("basic".into()),
        skills: vec![SkillId("flame_bite".into())],
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
        // Provide ample SP so the 4-cost skill always succeeds.
        .insert_resource(SpPool {
            current: 100,
            max: 100,
        })
        .init_resource::<ActionLog>()
        .init_resource::<Time>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_message::<TurnAdvanced>()
        .add_systems(Update, resolve_action_system);
    app
}

fn drain_events(cursor: &mut MessageCursor<CombatEvent>, app: &App) -> Vec<CombatEvent> {
    cursor
        .read(app.world().resource::<Messages<CombatEvent>>())
        .cloned()
        .collect()
}

// ---------------------------------------------------------------------------
// Happy-path tests
// ---------------------------------------------------------------------------

/// Casting Flame Bite on a live defender inserts StatusEffect(Burn, 3) on the defender
/// and emits OnStatusApplied.
#[test]
fn apply_status_inserts_component_and_emits_event() {
    let mut app = setup_app(SkillBook(vec![flame_bite_skill()]));
    let mut event_cursor = app
        .world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor();

    let attacker = app
        .world_mut()
        .spawn((
            make_unit(1, "Attacker", 200),
            Team::Ally,
            attacker_skills(),
            ult_charge_default(),
            Toughness::new(100, vec![]),
        ))
        .id();

    let defender = app
        .world_mut()
        .spawn((
            make_unit(2, "Defender", 500),
            Team::Enemy,
            Toughness::new(100, vec![]),
        ))
        .id();

    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("flame_bite".into()),
        target: UnitId(2),
    });

    app.update();

    // StatusEffect must be present on the defender.
    let status = app.world().get::<StatusEffect>(defender);
    assert!(
        status.is_some(),
        "StatusEffect should be inserted on defender after Flame Bite"
    );
    let s = status.unwrap();
    assert_eq!(s.kind, StatusEffectKind::Burn { damage_per_turn: 5 });
    assert_eq!(s.duration_remaining, 3);

    // OnStatusApplied event must be in the stream.
    let events = drain_events(&mut event_cursor, &app);
    let has_applied = events.iter().any(|e| {
        matches!(&e.kind, CombatEventKind::OnStatusApplied { kind }
            if *kind == StatusEffectKind::Burn { damage_per_turn: 5 })
    });
    assert!(
        has_applied,
        "OnStatusApplied(Burn) must be emitted; events={events:?}"
    );

    // Attacker entity should NOT have StatusEffect.
    assert!(
        app.world().get::<StatusEffect>(attacker).is_none(),
        "StatusEffect must not be placed on the attacker"
    );
}

/// Re-applying Flame Bite overwrites the existing StatusEffect (Bevy insert semantics).
/// Duration resets to the skill's duration value.
#[test]
fn reapply_status_overwrites_existing_component() {
    let mut app = setup_app(SkillBook(vec![flame_bite_skill()]));

    app.world_mut().spawn((
        make_unit(1, "Attacker", 200),
        Team::Ally,
        attacker_skills(),
        ult_charge_default(),
        Toughness::new(100, vec![]),
    ));

    let defender = app
        .world_mut()
        .spawn((
            make_unit(2, "Defender", 500),
            Team::Enemy,
            Toughness::new(100, vec![]),
            // Pre-existing status with different duration (simulates a previous tick).
            StatusEffect {
                kind: StatusEffectKind::Burn { damage_per_turn: 5 },
                duration_remaining: 1,
            },
        ))
        .id();

    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("flame_bite".into()),
        target: UnitId(2),
    });

    app.update();

    let status = app
        .world()
        .get::<StatusEffect>(defender)
        .expect("StatusEffect must exist");
    assert_eq!(
        status.duration_remaining, 3,
        "Re-apply must reset duration to skill value"
    );
}

// ---------------------------------------------------------------------------
// Negative tests
// ---------------------------------------------------------------------------

/// Casting Flame Bite on a KO'd defender (action fails validation) produces no StatusEffect
/// and no OnStatusApplied event.
#[test]
fn action_on_ko_target_produces_no_status() {
    let mut app = setup_app(SkillBook(vec![flame_bite_skill()]));
    let mut event_cursor = app
        .world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor();

    app.world_mut().spawn((
        make_unit(1, "Attacker", 200),
        Team::Ally,
        attacker_skills(),
        ult_charge_default(),
        Toughness::new(100, vec![]),
    ));

    // hp_current=0 so Unit::is_ko() returns true — apply_effects rejects non-revive attacks.
    let defender = app
        .world_mut()
        .spawn((
            Unit {
                id: UnitId(2),
                name: "Defender".into(),
                hp_max: 500,
                hp_current: 0,
                attribute: Attribute::Vaccine,
                resists: vec![],
                evo_stage: EvoStage::Adult,
            },
            Team::Enemy,
            Ko,
            Toughness::new(100, vec![]),
        ))
        .id();

    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("flame_bite".into()),
        target: UnitId(2),
    });

    app.update();

    assert!(
        app.world().get::<StatusEffect>(defender).is_none(),
        "StatusEffect must NOT be inserted when action fails (KO'd target)"
    );

    let events = drain_events(&mut event_cursor, &app);
    let has_applied = events
        .iter()
        .any(|e| matches!(&e.kind, CombatEventKind::OnStatusApplied { .. }));
    assert!(
        !has_applied,
        "OnStatusApplied must NOT be emitted when action fails; events={events:?}"
    );
}

/// When Flame Bite KOs the defender, the KO guard prevents StatusEffect insertion.
/// The defender dies, so there's no point applying a turn-based status.
#[test]
fn action_that_kos_target_produces_no_status() {
    let mut app = setup_app(SkillBook(vec![flame_bite_skill()]));
    let mut event_cursor = app
        .world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor();

    app.world_mut().spawn((
        make_unit(1, "Attacker", 200),
        Team::Ally,
        attacker_skills(),
        ult_charge_default(),
        Toughness::new(100, vec![]),
    ));

    // Defender with 1 HP — any damage KOs it.
    let defender = app
        .world_mut()
        .spawn((
            make_unit(2, "Fragile", 1),
            Team::Enemy,
            Toughness::new(100, vec![]),
        ))
        .id();

    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("flame_bite".into()),
        target: UnitId(2),
    });

    app.update();

    // Defender should be KO'd (Ko component inserted by the system).
    assert!(
        app.world().get::<Ko>(defender).is_some(),
        "Defender with 1 HP must be KO'd after Flame Bite"
    );

    // No StatusEffect on the KO'd defender.
    assert!(
        app.world().get::<StatusEffect>(defender).is_none(),
        "StatusEffect must NOT be inserted on a KO'd defender"
    );

    let events = drain_events(&mut event_cursor, &app);
    let has_applied = events
        .iter()
        .any(|e| matches!(&e.kind, CombatEventKind::OnStatusApplied { .. }));
    assert!(
        !has_applied,
        "OnStatusApplied must NOT be emitted when defender is KO'd; events={events:?}"
    );
}
