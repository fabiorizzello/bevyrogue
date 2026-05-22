//! T4 invariants for the out-of-turn ultimate burst (small-feature 260522-1).
//!
//! Drives the real `burst_action_system -> resolve_action_system` chain on a
//! minimal combat app and proves:
//!   (a) a ready, off-turn unit's burst fires (gauge reset, target damaged,
//!       `UltimateUsed` emitted, SP spent);
//!   (b) rejected when the gauge is not ready;
//!   (c) parked in the queue while an enemy is taking its turn, then fired the
//!       moment the enemy turn ends; also fires "between turns" (`WaitingForTurn`);
//!   (d) rejected when the burst unit is KO or stunned;
//!   (e) `TurnOrder.active_unit` and every unit's `ActionValue` are byte-identical
//!       before and after — a burst consumes no turn.

use bevy::{ecs::message::MessageCursor, prelude::*};
use bevyrogue::combat::turn_system::av::ActionValue;
use bevyrogue::combat::{
    events::{CombatEvent, CombatEventKind},
    kit::UnitSkills,
    log::ActionLog,
    sp::SpPool,
    state::{CombatPhase, CombatState},
    stun::Stunned,
    team::Team,
    toughness::Toughness,
    turn_order::{TurnAdvanced, TurnOrder},
    turn_system::{
        ActionIntent, OutOfTurnBurst, PendingBurstQueue, UltBurstRequest, burst_action_system,
        resolve_action_system,
    },
    types::{Attribute, DamageTag, EvoStage, SkillId, UnitId},
    ultimate::{UltAccumulationTrigger, UltimateCharge},
    unit::{Ko, Unit},
};
use bevyrogue::data::{
    SkillBookHandle,
    skills_ron::{
        Effect, SelfTargetRule, SkillBook, SkillDef, SkillImplementation, SkillTargeting,
        TargetLife, TargetShape, TargetSide,
    },
};

const BURST: u32 = 1;
const TARGET: u32 = 2;
/// The unit that actually holds the turn — the burst unit is NOT this one.
const ACTIVE: u32 = 9;
const ULT_SP_COST: i32 = 20;

fn make_unit(id: u32, hp: i32) -> Unit {
    Unit {
        id: UnitId(id),
        name: format!("U{id}"),
        hp_max: hp,
        hp_current: hp,
        attribute: Attribute::Vaccine,
        resists: vec![],
        evo_stage: EvoStage::Adult,
    }
}

fn ult_charge(current: i32) -> UltimateCharge {
    UltimateCharge {
        current,
        trigger: 100,
        cap: 150,
        trigger_type: UltAccumulationTrigger::OnBasicAttack,
        charge_per_event: 25,
    }
}

fn skills_with_ult() -> UnitSkills {
    UnitSkills {
        basic: SkillId("basic".into()),
        skills: vec![],
        ultimate: SkillId("ult".into()),
        follow_up: None,
    }
}

fn ult_skill() -> SkillDef {
    SkillDef {
        id: SkillId("ult".into()),
        name: "Burst Ult".into(),
        damage_tag: DamageTag::Fire,
        sp_cost: ULT_SP_COST,
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
            Effect::ToughnessHit(10),
        ],
        custom_signals: vec![],
        animation_sequence: None,
        qte: None,
        timeline: None,
    }
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
        legacy_ops: vec![Effect::Damage {
            amount: 10,
            target: TargetShape::Single,
            per_hop: Default::default(),
        }],
        custom_signals: vec![],
        animation_sequence: None,
        qte: None,
        timeline: None,
    }
}

/// Minimal combat app running the real burst -> resolve chain. `active_unit` is
/// set to [`ACTIVE`] so the burst unit acts strictly out of turn.
fn burst_app(phase: CombatPhase) -> App {
    let mut app = App::new();
    let mut assets = Assets::<SkillBook>::default();
    let handle = assets.add(SkillBook(vec![ult_skill(), basic_skill()]));
    app.insert_resource(assets)
        .insert_resource(SkillBookHandle(handle))
        .insert_resource(CombatState {
            phase,
            ..Default::default()
        })
        .insert_resource(TurnOrder {
            active_unit: Some(UnitId(ACTIVE)),
        })
        .insert_resource(SpPool {
            current: 100,
            max: 100,
        })
        .init_resource::<ActionLog>()
        .init_resource::<Time>()
        .init_resource::<OutOfTurnBurst>()
        .init_resource::<PendingBurstQueue>()
        .add_message::<ActionIntent>()
        .add_message::<UltBurstRequest>()
        .add_message::<CombatEvent>()
        .add_message::<TurnAdvanced>()
        .add_systems(Update, (burst_action_system, resolve_action_system).chain());
    app
}

/// Spawns the burst unit (with `ult_current` charge), an enemy target, and a
/// distinct active ally — each carrying an `ActionValue` for the AV invariant.
fn spawn_units(app: &mut App, ult_current: i32, ko: bool, stunned: bool) {
    let mut burst = app.world_mut().spawn((
        make_unit(BURST, 200),
        Team::Ally,
        skills_with_ult(),
        ult_charge(ult_current),
        Toughness::new(100, vec![]),
        ActionValue(7000),
    ));
    if ko {
        burst.insert(Ko);
    }
    if stunned {
        burst.insert(Stunned { turns_left: 1 });
    }
    app.world_mut().spawn((
        make_unit(TARGET, 1000),
        Team::Enemy,
        Toughness::new(200, vec![]),
        ActionValue(3000),
    ));
    app.world_mut()
        .spawn((make_unit(ACTIVE, 200), Team::Ally, ActionValue(0)));
}

fn send_burst(app: &mut App) {
    app.world_mut().write_message(UltBurstRequest {
        attacker: UnitId(BURST),
        target: UnitId(TARGET),
    });
}

fn ult_charge_of(app: &mut App, id: u32) -> i32 {
    let mut q = app.world_mut().query::<(&Unit, &UltimateCharge)>();
    q.iter(app.world())
        .find(|(u, _)| u.id == UnitId(id))
        .map(|(_, ult)| ult.current)
        .expect("unit must exist")
}

fn hp_of(app: &mut App, id: u32) -> i32 {
    let mut q = app.world_mut().query::<&Unit>();
    q.iter(app.world())
        .find(|u| u.id == UnitId(id))
        .map(|u| u.hp_current)
        .expect("unit must exist")
}

fn action_value_of(app: &mut App, id: u32) -> i32 {
    let mut q = app.world_mut().query::<(&Unit, &ActionValue)>();
    q.iter(app.world())
        .find(|(u, _)| u.id == UnitId(id))
        .map(|(_, av)| av.0)
        .expect("unit must have ActionValue")
}

fn ultimate_used_ids(app: &mut App, cursor: &mut MessageCursor<CombatEvent>) -> Vec<UnitId> {
    let messages = app.world().resource::<Messages<CombatEvent>>();
    cursor
        .read(messages)
        .filter_map(|ev| match ev.kind {
            CombatEventKind::UltimateUsed { unit_id } => Some(unit_id),
            _ => None,
        })
        .collect()
}

#[test]
fn burst_fires_off_turn_when_ready() {
    let mut app = burst_app(CombatPhase::WaitingAction);
    spawn_units(&mut app, 100, false, false);
    let mut cursor = app
        .world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor();

    send_burst(&mut app);
    app.update();

    assert_eq!(
        ult_charge_of(&mut app, BURST),
        0,
        "gauge must reset on burst"
    );
    assert!(
        hp_of(&mut app, TARGET) < 1000,
        "target must take ult damage"
    );
    assert_eq!(
        ultimate_used_ids(&mut app, &mut cursor),
        vec![UnitId(BURST)],
        "exactly one UltimateUsed for the burst unit"
    );
    assert_eq!(
        app.world().resource::<SpPool>().current,
        100 - ULT_SP_COST,
        "SP must be spent on the burst"
    );
}

#[test]
fn burst_preserves_active_unit_and_action_value() {
    let mut app = burst_app(CombatPhase::WaitingAction);
    spawn_units(&mut app, 100, false, false);

    let active_before = app.world().resource::<TurnOrder>().active_unit;
    let burst_av_before = action_value_of(&mut app, BURST);
    let active_av_before = action_value_of(&mut app, ACTIVE);

    send_burst(&mut app);
    app.update();

    assert_eq!(
        app.world().resource::<TurnOrder>().active_unit,
        active_before,
        "active_unit must be untouched by a burst"
    );
    assert_eq!(active_before, Some(UnitId(ACTIVE)));
    assert_eq!(
        action_value_of(&mut app, BURST),
        burst_av_before,
        "burst unit's ActionValue must be untouched"
    );
    assert_eq!(
        action_value_of(&mut app, ACTIVE),
        active_av_before,
        "active unit's ActionValue must be untouched"
    );
    // The burst still resolved.
    assert_eq!(ult_charge_of(&mut app, BURST), 0);
}

#[test]
fn burst_rejected_when_gauge_not_ready() {
    let mut app = burst_app(CombatPhase::WaitingAction);
    spawn_units(&mut app, 0, false, false); // gauge empty
    let mut cursor = app
        .world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor();

    send_burst(&mut app);
    app.update();

    assert_eq!(hp_of(&mut app, TARGET), 1000, "no damage when gauge empty");
    assert!(ultimate_used_ids(&mut app, &mut cursor).is_empty());
    assert_eq!(
        app.world().resource::<SpPool>().current,
        100,
        "no SP spent on a rejected burst"
    );
    assert_eq!(app.world().resource::<OutOfTurnBurst>().0, None);
}

#[test]
fn burst_queued_during_enemy_turn_then_fires_when_it_ends() {
    // The enemy (TARGET is Team::Enemy) holds the turn: pressing ult must NOT
    // fire mid-enemy-turn — it parks in the queue.
    let mut app = burst_app(CombatPhase::WaitingAction);
    spawn_units(&mut app, 100, false, false);
    app.world_mut().resource_mut::<TurnOrder>().active_unit = Some(UnitId(TARGET));
    let mut cursor = app
        .world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor();

    send_burst(&mut app);
    app.update();

    // Held: nothing resolved, gauge intact, request retained in the queue.
    assert_eq!(
        hp_of(&mut app, TARGET),
        1000,
        "no damage while the enemy is acting"
    );
    assert!(ultimate_used_ids(&mut app, &mut cursor).is_empty());
    assert_eq!(ult_charge_of(&mut app, BURST), 100, "gauge untouched");
    assert_eq!(app.world().resource::<OutOfTurnBurst>().0, None);
    assert_eq!(
        app.world().resource::<PendingBurstQueue>().0.len(),
        1,
        "the burst must be parked, not dropped"
    );

    // Enemy turn ends: control returns to an ally. The parked burst now fires
    // with no new request sent.
    app.world_mut().resource_mut::<TurnOrder>().active_unit = Some(UnitId(ACTIVE));
    app.update();

    assert_eq!(ult_charge_of(&mut app, BURST), 0, "queued burst must fire");
    assert!(hp_of(&mut app, TARGET) < 1000, "target took the queued ult");
    assert_eq!(
        ultimate_used_ids(&mut app, &mut cursor),
        vec![UnitId(BURST)]
    );
    assert!(
        app.world().resource::<PendingBurstQueue>().0.is_empty(),
        "queue drained after firing"
    );
}

#[test]
fn burst_queued_during_av_gap_fires_when_action_window_opens() {
    // The AV-ticking gap (`WaitingForTurn`, no active unit) is not a launchable
    // window — actions resolve only in `WaitingAction`. A burst pressed here must
    // park, then fire the instant the player's action window opens.
    let mut app = burst_app(CombatPhase::WaitingForTurn);
    spawn_units(&mut app, 100, false, false);
    app.world_mut().resource_mut::<TurnOrder>().active_unit = None;
    let mut cursor = app
        .world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor();

    send_burst(&mut app);
    app.update();

    assert_eq!(
        ult_charge_of(&mut app, BURST),
        100,
        "gauge untouched in AV gap"
    );
    assert!(ultimate_used_ids(&mut app, &mut cursor).is_empty());
    assert_eq!(
        app.world().resource::<PendingBurstQueue>().0.len(),
        1,
        "burst parked during the AV gap"
    );

    // Action window opens with an ally active: the parked burst fires.
    {
        let mut state = app.world_mut().resource_mut::<CombatState>();
        state.phase = CombatPhase::WaitingAction;
    }
    app.world_mut().resource_mut::<TurnOrder>().active_unit = Some(UnitId(ACTIVE));
    app.update();

    assert_eq!(ult_charge_of(&mut app, BURST), 0, "queued burst fires");
    assert!(hp_of(&mut app, TARGET) < 1000, "target took the queued ult");
    assert_eq!(
        ultimate_used_ids(&mut app, &mut cursor),
        vec![UnitId(BURST)]
    );
    assert!(app.world().resource::<PendingBurstQueue>().0.is_empty());
}

#[test]
fn burst_rejected_when_ko() {
    let mut app = burst_app(CombatPhase::WaitingAction);
    spawn_units(&mut app, 100, true, false); // KO
    let mut cursor = app
        .world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor();

    send_burst(&mut app);
    app.update();

    assert_eq!(hp_of(&mut app, TARGET), 1000, "no damage when KO");
    assert!(ultimate_used_ids(&mut app, &mut cursor).is_empty());
    assert_eq!(app.world().resource::<OutOfTurnBurst>().0, None);
}

#[test]
fn burst_rejected_when_stunned() {
    let mut app = burst_app(CombatPhase::WaitingAction);
    spawn_units(&mut app, 100, false, true); // stunned
    let mut cursor = app
        .world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor();

    send_burst(&mut app);
    app.update();

    assert_eq!(hp_of(&mut app, TARGET), 1000, "no damage when stunned");
    assert!(ultimate_used_ids(&mut app, &mut cursor).is_empty());
    assert_eq!(app.world().resource::<OutOfTurnBurst>().0, None);
}
