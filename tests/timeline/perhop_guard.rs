//! Integration test for the PerHop runtime length guard (M019/S04).
//!
//! Constructs a SkillBook with DamageCurve::PerHop(vec![30, 20]) and hops=3
//! WITHOUT calling validate_skill_book, bypassing the load-time validator.
//! The guard in pipeline.rs emits OnActionFailed and clamps the loop to 2 hops.

use bevy::{ecs::message::MessageCursor, prelude::*};
use bevyrogue::combat::{
    events::{CombatEvent, CombatEventKind},
    kit::UnitSkills,
    log::ActionLog,
    sp::SpPool,
    state::CombatState,
    team::Team,
    turn_order::TurnOrder,
    turn_system::{ActionIntent, resolve_action_system},
    types::{Attribute, DamageTag, EvoStage, SkillId, UnitId},
    ultimate::{UltAccumulationTrigger, UltimateCharge},
    unit::{SlotIndex, Unit},
};
use bevyrogue::data::{
    SkillBookHandle,
    skills_ron::{
        BounceSelector, DamageCurve, Effect, RepeatPolicy, SelfTargetRule, SkillBook, SkillDef,
        SkillImplementation, SkillTargeting, TargetLife, TargetShape, TargetSide,
    },
};

fn build_app(book: SkillBook) -> App {
    let mut assets = Assets::<SkillBook>::default();
    let handle = assets.add(book);
    let mut app = App::new();
    app.insert_resource(assets)
        .insert_resource(SkillBookHandle(handle))
        .init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .insert_resource(SpPool { current: 5, max: 5 })
        .init_resource::<ActionLog>()
        .init_resource::<Time>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
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

/// PerHop(vec![30, 20]) with hops=3 — 2 coefficients, 3 hops planned.
/// Expected: OnActionFailed emitted once, exactly 2 OnDamageDealt events (30, 20), no ghost hop.
#[test]
fn perhop_length_guard_emits_diagnostic_and_clamps_loop() {
    let shape = TargetShape::Bounce {
        hops: 3,
        selector: BounceSelector::LowestHpPctAlive,
        repeat: RepeatPolicy::NoRepeat,
    };

    // Construct skill with mismatched PerHop length. Do NOT call validate_skill_book.
    let book = SkillBook(vec![SkillDef {
        id: SkillId("short_chain".into()),
        name: "Short Chain".into(),
        damage_tag: DamageTag::Fire,
        sp_cost: 2,
        targeting: SkillTargeting {
            shape,
            side: TargetSide::Enemy,
            life: TargetLife::Alive,
            self_rule: SelfTargetRule::Forbid,
            ..Default::default()
        },
        implementation: SkillImplementation::Implemented,
        legacy_ops: vec![Effect::Damage {
            amount: 0,
            target: shape,
            per_hop: DamageCurve::PerHop(vec![30, 20]),
        }],
        ..Default::default()
    }]);

    let mut app = build_app(book);

    // Spawn attacker.
    app.world_mut().spawn((
        Unit {
            id: UnitId(1),
            name: "Attacker".into(),
            hp_max: 200,
            hp_current: 200,
            attribute: Attribute::Vaccine,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        Team::Ally,
        SlotIndex(0),
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
        UnitSkills {
            basic: SkillId("short_chain".into()),
            skills: vec![SkillId("short_chain".into())],
            ultimate: SkillId("short_chain".into()),
            follow_up: None,
        },
    ));

    // Spawn three enemies so NoRepeat pool has enough targets for 3 hops.
    for (id, slot, hp) in [(10u32, 0u8, 80i32), (11, 1, 60), (12, 2, 40)] {
        app.world_mut().spawn((
            Unit {
                id: UnitId(id),
                name: format!("Enemy{id}"),
                hp_max: 100,
                hp_current: hp,
                attribute: Attribute::Vaccine,
                resists: vec![],
                evo_stage: EvoStage::Adult,
            },
            Team::Enemy,
            SlotIndex(slot),
        ));
    }

    let mut cursor = message_cursor::<CombatEvent>(&mut app);
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("short_chain".into()),
        target: UnitId(12), // lowest HP
    });
    app.update();

    let events = drain_events(&mut cursor, &app);

    // (a) no panic — test reached here.

    // (b) exactly one OnActionFailed diagnostic naming the length mismatch.
    let failed: Vec<&CombatEvent> = events
        .iter()
        .filter(|e| matches!(e.kind, CombatEventKind::OnActionFailed { .. }))
        .collect();
    assert_eq!(
        failed.len(),
        1,
        "expected exactly 1 OnActionFailed, got {}",
        failed.len()
    );
    match &failed[0].kind {
        CombatEventKind::OnActionFailed { reason } => {
            assert!(
                reason.contains("PerHop length 2 < hops_planned 3"),
                "reason must name the mismatch, got: {reason}"
            );
        }
        _ => unreachable!(),
    }

    // (c) exactly 2 OnDamageDealt events with amounts [30, 20].
    let damage_amounts: Vec<i32> = events
        .iter()
        .filter_map(|e| match e.kind {
            CombatEventKind::OnDamageDealt { amount, .. } => Some(amount),
            _ => None,
        })
        .collect();
    assert_eq!(
        damage_amounts.len(),
        2,
        "expected 2 damage events (one per PerHop coefficient), got {}",
        damage_amounts.len()
    );

    // (d) coefficients applied in order: [30, 20] — no ghost third hop.
    assert_eq!(
        damage_amounts,
        vec![30, 20],
        "damage must match PerHop coefficients exactly"
    );
}
