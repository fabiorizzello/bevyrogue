//! Consolidated target-shape execution tests for `AllEnemies` and `Blast`.
//!
//! Replaces the previous `target_shape_aoe_all_order.rs` +
//! `target_shape_blast_spillover.rs` (≈480 LOC of near-duplicate setup).
//! Helpers (`build_app`, `spawn_attacker`, `spawn_enemy*`, drain) are
//! defined once here. The Bounce-chain and truthfulness clusters keep their
//! own files because they exercise distinct concerns.

use bevy::{ecs::message::MessageCursor, prelude::*};
use bevyrogue::combat::{
    events::{CombatEvent, CombatEventKind},
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
    unit::{SlotIndex, Unit},
};
use bevyrogue::data::{
    SkillBookHandle,
    skills_ron::{
        Effect, SelfTargetRule, SkillBook, SkillDef, SkillImplementation, SkillTargeting,
        TargetLife, TargetShape, TargetSide,
    },
};

// ── Skill-book + app fixtures ─────────────────────────────────────────────────

fn skill_book(skill_id: &str, shape: TargetShape) -> SkillBook {
    SkillBook(vec![SkillDef {
        id: SkillId(skill_id.into()),
        name: skill_id.into(),
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
        legacy_ops: vec![
            Effect::Damage {
                amount: 10,
                target: shape,
                per_hop: Default::default(),
            },
            Effect::ToughnessHit(5),
        ],
        ..Default::default()
    }])
}

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

// ── Spawn helpers ─────────────────────────────────────────────────────────────

fn spawn_attacker(app: &mut App, skill_id: &str, slot: u8) {
    let skill_id = SkillId(skill_id.into());
    app.world_mut().spawn((
        Unit {
            id: UnitId(1),
            name: "Attacker".into(),
            hp_max: 100,
            hp_current: 100,
            attribute: Attribute::Vaccine,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        Team::Ally,
        SlotIndex(slot),
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
        UnitSkills {
            basic: skill_id.clone(),
            skills: vec![skill_id.clone()],
            ultimate: skill_id,
            follow_up: None,
        },
    ));
}

fn spawn_enemy(app: &mut App, id: u32, slot: u8, hp_current: i32) {
    app.world_mut().spawn((
        Unit {
            id: UnitId(id),
            name: format!("Enemy{id}"),
            hp_max: 100,
            hp_current,
            attribute: Attribute::Virus,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        Team::Enemy,
        SlotIndex(slot),
        Toughness {
            max: 40,
            current: 40,
            weaknesses: vec![DamageTag::Fire],
            broken: false,
            category: Default::default(),
        },
    ));
}

fn damage_targets(events: &[CombatEvent]) -> Vec<UnitId> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            CombatEventKind::OnDamageDealt { .. } => Some(e.target),
            _ => None,
        })
        .collect()
}

fn fire_skill(app: &mut App, skill_id: &str, primary: UnitId) -> Vec<CombatEvent> {
    let mut cursor = message_cursor::<CombatEvent>(app);
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId(skill_id.into()),
        target: primary,
    });
    app.update();
    drain_events(&mut cursor, app)
}

// ── AllEnemies — KO-skipping and slot-ascending order ────────────────────────

/// AllEnemies on a 3-enemy formation with slot 1 KO'd:
/// only 2 OnDamageDealt events for the alive enemies, in slot_index asc order.
/// SP consumed once.
#[test]
fn aoe_skips_ko_and_fires_in_slot_order() {
    let mut app = build_app(skill_book("wide_blast", TargetShape::AllEnemies));
    spawn_attacker(&mut app, "wide_blast", 0);
    spawn_enemy(&mut app, 10, 0, 100); // alive
    spawn_enemy(&mut app, 11, 1, 0); // KO
    spawn_enemy(&mut app, 12, 2, 100); // alive

    let events = fire_skill(&mut app, "wide_blast", UnitId(10));
    let targets = damage_targets(&events);

    assert_eq!(
        targets,
        vec![UnitId(10), UnitId(12)],
        "AllEnemies must skip KO'd and order by slot ascending"
    );
    assert_eq!(
        app.world().resource::<SpPool>().current,
        3,
        "SP consumed once (5 - 2 = 3), not 2×"
    );

    // KO'd enemy untouched.
    let mut q = app.world_mut().query::<&Unit>();
    let ko_hp = q
        .iter(app.world())
        .find(|u| u.id == UnitId(11))
        .map(|u| u.hp_current)
        .expect("KO enemy missing");
    assert_eq!(ko_hp, 0, "KO'd enemy should remain at 0 HP");
}

/// Re-run the AllEnemies test 10 times to confirm deterministic ordering.
#[test]
fn aoe_order_is_deterministic_across_10_runs() {
    for run in 0..10 {
        let mut app = build_app(skill_book("wide_blast", TargetShape::AllEnemies));
        spawn_attacker(&mut app, "wide_blast", 0);
        spawn_enemy(&mut app, 10, 0, 100);
        spawn_enemy(&mut app, 11, 1, 0);
        spawn_enemy(&mut app, 12, 2, 100);

        let events = fire_skill(&mut app, "wide_blast", UnitId(10));
        assert_eq!(
            damage_targets(&events),
            vec![UnitId(10), UnitId(12)],
            "run {run}: AllEnemies order must be [10, 12]"
        );
    }
}

// ── Blast — adjacent spillover and edge clamp ────────────────────────────────

/// Blast on primary at slot 1 hits slots 0, 1, 2 (all three enemies).
/// SP consumed once (cost=2), order is slot ascending.
#[test]
fn blast_hits_all_three_adjacent_enemies_sp_consumed_once() {
    let mut app = build_app(skill_book("blast_strike", TargetShape::Blast));
    spawn_attacker(&mut app, "blast_strike", 0);
    spawn_enemy(&mut app, 10, 0, 100);
    spawn_enemy(&mut app, 11, 1, 100); // primary
    spawn_enemy(&mut app, 12, 2, 100);

    assert_eq!(app.world().resource::<SpPool>().current, 5);

    let events = fire_skill(&mut app, "blast_strike", UnitId(11));
    let targets = damage_targets(&events);

    assert_eq!(
        targets,
        vec![UnitId(10), UnitId(11), UnitId(12)],
        "Blast must hit all 3 in slot order"
    );
    assert_eq!(
        app.world().resource::<SpPool>().current,
        3,
        "SP consumed once (cost=2), not 3×"
    );

    let mut q = app.world_mut().query::<&Unit>();
    for enemy_id in [UnitId(10), UnitId(11), UnitId(12)] {
        let hp = q
            .iter(app.world())
            .find(|u| u.id == enemy_id)
            .map(|u| u.hp_current)
            .expect("enemy missing");
        assert!(hp < 100, "{enemy_id:?} should have taken damage, hp={hp}");
    }
}

/// Blast on primary at slot 0: no slot -1, only slots 0 and 1 hit.
#[test]
fn blast_edge_slot_zero_hits_only_two_enemies() {
    let mut app = build_app(skill_book("blast_strike", TargetShape::Blast));
    spawn_attacker(&mut app, "blast_strike", 0);
    spawn_enemy(&mut app, 10, 0, 100); // primary
    spawn_enemy(&mut app, 11, 1, 100);
    spawn_enemy(&mut app, 12, 2, 100);

    let events = fire_skill(&mut app, "blast_strike", UnitId(10));
    assert_eq!(
        damage_targets(&events),
        vec![UnitId(10), UnitId(11)],
        "Blast at edge slot 0 must hit only slots 0 and 1"
    );
}
