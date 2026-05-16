//! Integration tests for `TargetShape::Bounce` on the compiled-timeline path.
//!
//! These fixtures drive bounce via a compiled timeline hook, not the legacy
//! runtime branch. The hook simulates the hop-by-hop selector path locally so
//! target order, repeat policy, and mid-chain KO behavior remain observable
//! through emitted combat events.

use bevy::{ecs::message::MessageCursor, prelude::*};
use bevyrogue::combat::{
    api::{
        register_kernel_builtins,
        timeline::{Beat, BeatEdge, BeatKind, BeatPayload, TimelineLibrary},
        BeatEvent, ExtRegistries, SkillCtx,
    },
    events::{CombatEvent, CombatEventKind},
    kit::UnitSkills,
    log::ActionLog,
    resolution::{select_bounce_hop, TargetEntry, TargetableSnapshot},
    sp::SpPool,
    state::CombatState,
    team::Team,
    turn_order::TurnOrder,
    turn_system::{resolve_action_system, ActionIntent},
    types::{Attribute, DamageTag, EvoStage, SkillId, UnitId},
    ultimate::{UltAccumulationTrigger, UltimateCharge},
    unit::{Ko, SlotIndex, Unit},
};
use bevyrogue::data::{
    skill_timeline::{compile_skill_book_timelines, SkillTimeline},
    skills_ron::{
        BounceSelector, RepeatPolicy, SelfTargetRule, SkillBook, SkillDef, SkillImplementation,
        SkillTargeting, TargetLife, TargetShape, TargetSide,
    },
    SkillBookHandle,
};

#[derive(Clone, Debug)]
struct LocalTargetState {
    id: UnitId,
    team: Team,
    slot_index: u8,
    alive: bool,
    hp_current: i32,
    hp_max: i32,
}

fn states_to_snapshot(states: &[LocalTargetState]) -> TargetableSnapshot {
    TargetableSnapshot {
        entries: states
            .iter()
            .map(|state| TargetEntry {
                id: state.id,
                team: state.team,
                slot_index: state.slot_index,
                alive: state.alive,
                hp_per_mille: if state.hp_max > 0 {
                    ((state.hp_current.max(0) as u64 * 1000) / state.hp_max as u64) as u32
                } else {
                    0
                },
            })
            .collect(),
    }
}

fn snapshot_state(world: &World) -> Vec<LocalTargetState> {
    let Some(mut q) = world.try_query::<(&Unit, &Team, Option<&Ko>, Option<&SlotIndex>)>() else {
        return vec![];
    };
    q.iter(world)
        .map(|(unit, team, ko, slot)| LocalTargetState {
            id: unit.id,
            team: *team,
            slot_index: slot.map(|s| s.0).unwrap_or(0),
            alive: ko.is_none() && unit.hp_current > 0,
            hp_current: unit.hp_current,
            hp_max: unit.hp_max,
        })
        .collect()
}

fn enqueue_bounce_chain(
    ctx: &mut SkillCtx<'_>,
    selector: BounceSelector,
    repeat: RepeatPolicy,
    hops: u8,
    amounts: &'static [i32],
    tag: DamageTag,
) {
    let Some(primary_team) = snapshot_state(ctx.world)
        .into_iter()
        .find(|entry| entry.id == ctx.primary_target)
        .map(|entry| entry.team) else {
        return;
    };

    let mut states = snapshot_state(ctx.world);
    let mut already_hit = std::collections::HashSet::new();
    let mut last_slot: Option<u8> = None;

    for hop in 0..hops as usize {
        let snapshot = states_to_snapshot(&states);
        let Some(target_id) = select_bounce_hop(
            selector,
            &snapshot,
            &already_hit,
            repeat,
            primary_team,
            last_slot,
        ) else {
            break;
        };

        let amount = amounts
            .get(hop)
            .copied()
            .unwrap_or_else(|| *amounts.last().unwrap_or(&0));

        ctx.enqueue(bevyrogue::combat::api::intent::Intent::DealDamage {
            source: ctx.caster,
            target: target_id,
            amount,
            tag,
            cast_id: ctx.cast_id,
        });

        if repeat == RepeatPolicy::NoRepeat {
            already_hit.insert(target_id);
        }

        let Some(state) = states.iter_mut().find(|state| state.id == target_id) else {
            break;
        };
        last_slot = Some(state.slot_index);

        state.hp_current -= amount;
        if state.hp_current <= 0 {
            state.alive = false;
        }
    }
}

fn bounce_case1_hook(_: &BeatEvent, ctx: &mut SkillCtx<'_>) {
    enqueue_bounce_chain(
        ctx,
        BounceSelector::LowestHpPctAlive,
        RepeatPolicy::NoRepeat,
        3,
        &[20, 20, 20],
        DamageTag::Fire,
    );
}

fn bounce_case2_hook(_: &BeatEvent, ctx: &mut SkillCtx<'_>) {
    enqueue_bounce_chain(
        ctx,
        BounceSelector::NextSlotAlive,
        RepeatPolicy::NoRepeat,
        3,
        &[20, 16, 12],
        DamageTag::Fire,
    );
}

fn bounce_case3_hook(_: &BeatEvent, ctx: &mut SkillCtx<'_>) {
    enqueue_bounce_chain(
        ctx,
        BounceSelector::LowestHpPctAlive,
        RepeatPolicy::AllowRepeat,
        3,
        &[30, 15, 5],
        DamageTag::Fire,
    );
}

fn bounce_case4_hook(_: &BeatEvent, ctx: &mut SkillCtx<'_>) {
    enqueue_bounce_chain(
        ctx,
        BounceSelector::LowestHpPctAlive,
        RepeatPolicy::NoRepeat,
        4,
        &[15, 15],
        DamageTag::Fire,
    );
}

// ── App builder ──────────────────────────────────────────────────────────────

fn build_app(book: SkillBook, sp_start: i32) -> App {
    let mut app = App::new();
    let mut assets = Assets::<SkillBook>::default();
    let handle = assets.add(book.clone());

    let mut regs = ExtRegistries::default();
    register_kernel_builtins(&mut regs);
    regs.hooks.register("test/bounce_case1", bounce_case1_hook);
    regs.hooks.register("test/bounce_case2", bounce_case2_hook);
    regs.hooks.register("test/bounce_case3", bounce_case3_hook);
    regs.hooks.register("test/bounce_case4", bounce_case4_hook);

    let compiled = compile_skill_book_timelines(&book, &regs)
        .expect("bounce test book must compile into timelines");

    app.insert_resource(assets)
        .insert_resource(SkillBookHandle(handle))
        .insert_resource(TimelineLibrary::<String>::default())
        .insert_resource(regs)
        .init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .insert_resource(SpPool {
            current: sp_start,
            max: sp_start.max(5),
        })
        .init_resource::<ActionLog>()
        .init_resource::<Time>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_systems(Update, resolve_action_system);

    app.world_mut()
        .resource_mut::<TimelineLibrary<String>>()
        .timelines = compiled;

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

// ── Spawn helpers ────────────────────────────────────────────────────────────

fn spawn_attacker(app: &mut App, id: u32, slot: u8, skill_id: &str) {
    let skill_id = SkillId(skill_id.into());
    app.world_mut().spawn((
        Unit {
            id: UnitId(id),
            name: format!("Attacker{id}"),
            hp_max: 200,
            hp_current: 200,
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

fn spawn_enemy(app: &mut App, id: u32, slot: u8, hp_max: i32) {
    app.world_mut().spawn((
        Unit {
            id: UnitId(id),
            name: format!("Enemy{id}"),
            hp_max,
            hp_current: hp_max,
            attribute: Attribute::Vaccine,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        Team::Enemy,
        SlotIndex(slot),
    ));
}

fn spawn_enemy_with_hp(app: &mut App, id: u32, slot: u8, hp_max: i32, hp_current: i32) {
    app.world_mut().spawn((
        Unit {
            id: UnitId(id),
            name: format!("Enemy{id}"),
            hp_max,
            hp_current,
            attribute: Attribute::Vaccine,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        Team::Enemy,
        SlotIndex(slot),
    ));
}

// ── Skill book fixtures ─────────────────────────────────────────────────────

fn bounce_skill(
    skill_id: &str,
    sp_cost: i32,
    hops: u8,
    selector: BounceSelector,
    repeat: RepeatPolicy,
    hook_id: &'static str,
) -> SkillBook {
    let shape = TargetShape::Bounce {
        hops,
        selector,
        repeat,
    };
    SkillBook(vec![SkillDef {
        id: SkillId(skill_id.into()),
        name: skill_id.into(),
        damage_tag: DamageTag::Fire,
        sp_cost,
        targeting: SkillTargeting {
            shape,
            side: TargetSide::Enemy,
            life: TargetLife::Alive,
            self_rule: SelfTargetRule::Forbid,
            ..Default::default()
        },
        implementation: SkillImplementation::Implemented,
        legacy_ops: vec![],
        timeline: Some(SkillTimeline {
            entry: "cast".into(),
            beats: vec![
                Beat {
                    id: "cast".into(),
                    kind: BeatKind::Cast,
                    hook: None,
                    selector: None,
                    presentation: None,
                    payload: None,
                },
                Beat {
                    id: "impact".into(),
                    kind: BeatKind::Impact,
                    hook: Some(hook_id.into()),
                    selector: None,
                    presentation: None,
                    payload: Some(BeatPayload::DealDamage {
                        amount: 1,
                        tag: DamageTag::Fire,
                        target: shape,
                    }),
                },
            ],
            edges: vec![BeatEdge {
                from: "cast".into(),
                to: "impact".into(),
                gate: Some("core/always".into()),
            }],
        }),
        ..Default::default()
    }])
}

/// Collect the amounts from all OnDamageDealt events in order.
fn damage_amounts(events: &[CombatEvent]) -> Vec<i32> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            CombatEventKind::OnDamageDealt { amount, .. } => Some(amount),
            _ => None,
        })
        .collect()
}

/// Collect the targets from all OnDamageDealt events in order.
fn damage_targets(events: &[CombatEvent]) -> Vec<UnitId> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            CombatEventKind::OnDamageDealt { .. } => Some(e.target),
            _ => None,
        })
        .collect()
}

// ── Case 1: LowestHpPct + NoRepeat + Constant, 3 hops, no KO ─────────────────

#[test]
fn bounce_lowest_hp_no_repeat_constant_full_chain() {
    let book = bounce_skill(
        "chain_bolt",
        2,
        3,
        BounceSelector::LowestHpPctAlive,
        RepeatPolicy::NoRepeat,
        "test/bounce_case1",
    );
    let mut app = build_app(book, 5);

    spawn_attacker(&mut app, 1, 0, "chain_bolt");
    spawn_enemy_with_hp(&mut app, 10, 0, 100, 80); // 800‰
    spawn_enemy_with_hp(&mut app, 11, 1, 100, 50); // 500‰
    spawn_enemy_with_hp(&mut app, 12, 2, 100, 30); // 300‰

    let sp_before = app.world().resource::<SpPool>().current;

    let mut cursor = message_cursor::<CombatEvent>(&mut app);
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("chain_bolt".into()),
        target: UnitId(12),
    });
    app.update();

    let events = drain_events(&mut cursor, &app);
    let amounts = damage_amounts(&events);
    let targets = damage_targets(&events);

    assert_eq!(amounts.len(), 3, "expected 3 hops, got {}", amounts.len());
    assert_eq!(amounts, vec![20, 20, 20]);
    assert_eq!(targets, vec![UnitId(12), UnitId(11), UnitId(10)]);

    let sp_after = app.world().resource::<SpPool>().current;
    assert_eq!(sp_after, sp_before - 2);
}

// ── Case 2: NextSlotAlive + NoRepeat + Falloff, KO mid-chain ─────────────────

#[test]
fn bounce_next_slot_no_repeat_falloff_ko_mid_chain() {
    let book = bounce_skill(
        "bolt_chain",
        2,
        3,
        BounceSelector::NextSlotAlive,
        RepeatPolicy::NoRepeat,
        "test/bounce_case2",
    );
    let mut app = build_app(book, 5);

    spawn_attacker(&mut app, 1, 0, "bolt_chain");
    spawn_enemy(&mut app, 10, 0, 100);
    spawn_enemy_with_hp(&mut app, 11, 1, 100, 1);
    spawn_enemy(&mut app, 12, 2, 100);

    let mut cursor = message_cursor::<CombatEvent>(&mut app);
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("bolt_chain".into()),
        target: UnitId(10),
    });
    app.update();

    let events = drain_events(&mut cursor, &app);
    let amounts = damage_amounts(&events);
    let targets = damage_targets(&events);

    assert_eq!(amounts.len(), 3, "expected 3 hops, got {}", amounts.len());
    assert_eq!(amounts, vec![20, 16, 12]);
    assert_eq!(targets, vec![UnitId(10), UnitId(11), UnitId(12)]);

    let Some(mut unit_q) = app.world().try_query::<(&Unit, &Team)>() else {
        panic!("E11 not found");
    };
    let world = app.world();
    let e11_hp = unit_q
        .iter(world)
        .find(|(u, t)| u.id == UnitId(11) && **t == Team::Enemy)
        .map(|(u, _)| u.hp_current)
        .expect("E11 not found");
    assert!(e11_hp <= 0, "E11 should be KO'd, hp={}", e11_hp);
}

// ── Case 3: LowestHpPct + AllowRepeat + PerHop ────────────────────────────────

#[test]
fn bounce_lowest_hp_allow_repeat_per_hop_curve() {
    let book = bounce_skill(
        "scatter_bolt",
        2,
        3,
        BounceSelector::LowestHpPctAlive,
        RepeatPolicy::AllowRepeat,
        "test/bounce_case3",
    );
    let mut app = build_app(book, 5);

    spawn_attacker(&mut app, 1, 0, "scatter_bolt");
    spawn_enemy_with_hp(&mut app, 10, 0, 100, 90);
    spawn_enemy_with_hp(&mut app, 11, 1, 100, 70);
    spawn_enemy_with_hp(&mut app, 12, 2, 100, 20);

    let mut cursor = message_cursor::<CombatEvent>(&mut app);
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("scatter_bolt".into()),
        target: UnitId(12),
    });
    app.update();

    let events = drain_events(&mut cursor, &app);
    let amounts = damage_amounts(&events);

    assert_eq!(amounts.len(), 3, "expected 3 hops, got {}", amounts.len());
    assert_eq!(amounts, vec![30, 15, 5]);

    let targets = damage_targets(&events);
    assert_eq!(targets[0], UnitId(12));
}

// ── Case 4: Pool exhaustion truncates silently ────────────────────────────────

#[test]
fn bounce_pool_exhaustion_truncates_silently() {
    let book = bounce_skill(
        "arc_bolt",
        2,
        4,
        BounceSelector::LowestHpPctAlive,
        RepeatPolicy::NoRepeat,
        "test/bounce_case4",
    );
    let mut app = build_app(book, 5);

    spawn_attacker(&mut app, 1, 0, "arc_bolt");
    spawn_enemy(&mut app, 10, 0, 100);
    spawn_enemy(&mut app, 11, 1, 100);

    let mut cursor = message_cursor::<CombatEvent>(&mut app);
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("arc_bolt".into()),
        target: UnitId(10),
    });
    app.update();

    let events = drain_events(&mut cursor, &app);
    let amounts = damage_amounts(&events);

    assert_eq!(amounts.len(), 2, "expected chain truncation after 2 hits");
    assert_eq!(amounts, vec![15, 15]);

    let failed = events
        .iter()
        .filter(|e| matches!(e.kind, CombatEventKind::OnActionFailed { .. }))
        .count();
    assert_eq!(failed, 0, "pool exhaustion must not emit OnActionFailed");
}
  assert_eq!(failed, 0, "pool exhaustion must not emit OnActionFailed");
}
