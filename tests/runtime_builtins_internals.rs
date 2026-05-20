use bevy::prelude::World;
use bevyrogue::combat::runtime::builtins::register_kernel_builtins;
use bevyrogue::combat::runtime::intent::Intent;
use bevyrogue::combat::runtime::registry::ExtRegistries;
use bevyrogue::combat::runtime::timeline::{
    Beat, BeatEdge, BeatEvent, BeatKind, BeatPayload, CompiledTimeline,
};
use bevyrogue::combat::runtime::{SignalPayload, intent::CastId, validate_timeline_refs};
use bevyrogue::combat::runtime::{SkillCtx, SkillCtxMode};
use bevyrogue::combat::types::UnitId;
use bevyrogue::data::skills_ron::TargetShape;
use std::{
    collections::{HashSet, VecDeque},
    num::NonZeroU32,
};

fn owned_builtin_timeline(
    hook: &'static str,
    selector: &'static str,
    predicate: &'static str,
) -> CompiledTimeline<&'static str> {
    CompiledTimeline {
        id: "test_skill",
        entry: "cast",
        beats: vec![
            Beat {
                id: "cast",
                kind: BeatKind::Cast,
                hook: None,
                selector: None,
                presentation: None,
                payload: None,
            },
            Beat {
                id: "impact",
                kind: BeatKind::Impact,
                hook: Some(hook),
                selector: Some(selector),
                presentation: None,
                payload: Some(BeatPayload::DealDamage {
                    amount: 1,
                    tag: bevyrogue::combat::types::DamageTag::Fire,
                    target: TargetShape::Single,
                }),
            },
        ],
        edges: vec![BeatEdge {
            from: "cast",
            to: "impact",
            gate: Some(predicate),
        }],
    }
}

#[test]
fn register_kernel_builtins_installs_core_ids() {
    let mut regs = ExtRegistries::default();
    register_kernel_builtins(&mut regs);

    assert!(regs.hooks.get("core/deal_damage").is_some());
    assert!(regs.hooks.get("core/break_toughness").is_some());
    assert!(regs.hooks.get("core/apply_status").is_some());
    assert!(regs.hooks.get("core/delay_turn").is_some());
    assert!(regs.hooks.get("core/advance_turn").is_some());
    assert!(regs.hooks.get("core/revive").is_some());
    assert!(regs.hooks.get("core/grant_free_skill").is_some());
    assert!(regs.hooks.get("core/grant_energy").is_some());
    assert!(regs.hooks.get("core/self_advance").is_some());
    assert!(regs.hooks.get("core/apply_effect").is_some());
    assert!(regs.selectors.get("core/primary").is_some());
    assert!(regs.selectors.get("core/caster").is_some());
    assert!(regs.predicates.get("core/always").is_some());
    assert!(regs.predicates.get("core/never").is_some());
}

#[test]
fn deal_damage_builtin_uses_payload_and_targets() {
    let mut regs = ExtRegistries::default();
    register_kernel_builtins(&mut regs);
    let hook = regs
        .hooks
        .get("core/deal_damage")
        .expect("builtin hook registered");

    let world = World::new();
    let mut cast_hit_set = HashSet::new();
    let mut pending = VecDeque::new();
    let payload = BeatPayload::DealDamage {
        amount: 17,
        tag: bevyrogue::combat::types::DamageTag::Physical,
        target: TargetShape::Single,
    };
    let mut ctx = SkillCtx::new(
        UnitId(1),
        UnitId(2),
        CastId(NonZeroU32::new(7).unwrap()),
        SkillCtxMode::Execute,
        &regs,
        &world,
        &mut cast_hit_set,
        &mut pending,
        Some(&payload),
    );
    let evt = BeatEvent {
        cast_id: ctx.cast_id,
        beat_id: "core/deal_damage",
        hop_index: 0,
        beat_targets: vec![UnitId(9)],
    };

    hook(&evt, &mut ctx);

    match pending.pop_front().expect("hook should enqueue one intent") {
        Intent::DealDamage {
            source,
            target,
            amount,
            ..
        } => {
            assert_eq!(source, UnitId(1));
            assert_eq!(target, UnitId(9));
            assert_eq!(amount, 17);
        }
        other => panic!("unexpected intent: {other:?}"),
    }
}

#[test]
fn builtins_cover_new_active_verbs() {
    let mut regs = ExtRegistries::default();
    register_kernel_builtins(&mut regs);

    let world = World::new();
    let mut cast_hit_set = HashSet::new();
    let mut pending = VecDeque::new();
    let caster = UnitId(1);
    let revive_target = UnitId(2);
    let cast_id = CastId(NonZeroU32::new(11).unwrap());
    let evt = BeatEvent {
        cast_id,
        beat_id: "beat",
        hop_index: 0,
        beat_targets: vec![revive_target],
    };

    let mut ctx = SkillCtx::new(
        caster,
        revive_target,
        cast_id,
        SkillCtxMode::Execute,
        &regs,
        &world,
        &mut cast_hit_set,
        &mut pending,
        Some(&BeatPayload::Revive {
            pct: 25,
            target: TargetShape::Single,
        }),
    );
    regs.hooks.get("core/revive").unwrap()(&evt, &mut ctx);
    match pending.pop_front().unwrap() {
        Intent::Revive {
            source,
            target: revived_target,
            pct,
            ..
        } => {
            assert_eq!(source, caster);
            assert_eq!(revived_target, revive_target);
            assert_eq!(pct, 25);
        }
        other => panic!("unexpected intent: {other:?}"),
    }

    let mut pending = VecDeque::new();
    let mut cast_hit_set = HashSet::new();
    let mut ctx = SkillCtx::new(
        caster,
        revive_target,
        cast_id,
        SkillCtxMode::Execute,
        &regs,
        &world,
        &mut cast_hit_set,
        &mut pending,
        Some(&BeatPayload::GrantEnergy { amount: 6 }),
    );
    regs.hooks.get("core/grant_energy").unwrap()(&evt, &mut ctx);
    match pending.pop_front().unwrap() {
        Intent::AddEnergy { target, amount, .. } => {
            assert_eq!(target, caster);
            assert_eq!(amount, 6);
        }
        other => panic!("unexpected intent: {other:?}"),
    }

    let mut pending = VecDeque::new();
    let mut cast_hit_set = HashSet::new();
    let mut ctx = SkillCtx::new(
        caster,
        revive_target,
        cast_id,
        SkillCtxMode::Execute,
        &regs,
        &world,
        &mut cast_hit_set,
        &mut pending,
        Some(&BeatPayload::SelfAdvance { amount_pct: 20 }),
    );
    regs.hooks.get("core/self_advance").unwrap()(&evt, &mut ctx);
    match pending.pop_front().unwrap() {
        Intent::AdvanceTurn {
            target, amount_pct, ..
        } => {
            assert_eq!(target, caster);
            assert_eq!(amount_pct, 20);
        }
        other => panic!("unexpected intent: {other:?}"),
    }

    let mut pending = VecDeque::new();
    let mut cast_hit_set = HashSet::new();
    let mut ctx = SkillCtx::new(
        caster,
        revive_target,
        cast_id,
        SkillCtxMode::Execute,
        &regs,
        &world,
        &mut cast_hit_set,
        &mut pending,
        Some(&BeatPayload::GrantFreeSkill { count: 3 }),
    );
    regs.hooks.get("core/grant_free_skill").unwrap()(&evt, &mut ctx);
    match pending.pop_front().unwrap() {
        Intent::GrantFreeSkill { source, count, .. } => {
            assert_eq!(source, caster);
            assert_eq!(count, 3);
        }
        other => panic!("unexpected intent: {other:?}"),
    }
}

#[test]
fn apply_effect_builtin_enqueues_blueprint_signal() {
    let mut regs = ExtRegistries::default();
    register_kernel_builtins(&mut regs);
    let hook = regs
        .hooks
        .get("core/apply_effect")
        .expect("builtin hook registered");

    let world = World::new();
    let mut cast_hit_set = HashSet::new();
    let mut pending = VecDeque::new();
    let payload = BeatPayload::BlueprintSignal {
        owner: "tentomon".to_string(),
        name: "build_static_charge".to_string(),
        payload: SignalPayload::Amount(1),
    };
    let mut ctx = SkillCtx::new(
        UnitId(1),
        UnitId(2),
        CastId(NonZeroU32::new(7).unwrap()),
        SkillCtxMode::Execute,
        &regs,
        &world,
        &mut cast_hit_set,
        &mut pending,
        Some(&payload),
    );
    let evt = BeatEvent {
        cast_id: ctx.cast_id,
        beat_id: "core/apply_effect",
        hop_index: 0,
        beat_targets: vec![],
    };

    hook(&evt, &mut ctx);

    match pending.pop_front().expect("hook should enqueue one intent") {
        Intent::BlueprintSignal { owner, name, .. } => {
            assert_eq!(owner, "tentomon");
            assert_eq!(name, "build_static_charge");
        }
        other => panic!("unexpected intent: {other:?}"),
    }
}

#[test]
fn builtin_hook_typos_report_precise_axis_and_site() {
    let regs = {
        let mut regs = ExtRegistries::default();
        register_kernel_builtins(&mut regs);
        regs
    };

    let timeline = owned_builtin_timeline("core/deal_damge", "core/primary", "core/always");
    let err = validate_timeline_refs(&timeline, &regs).unwrap_err();
    assert_eq!(err.len(), 1);
    assert_eq!(err[0].axis, "hook");
    assert_eq!(err[0].missing_id, "core/deal_damge");
    assert_eq!(err[0].site, "beat impact");
}

#[test]
fn builtin_selector_typos_report_precise_axis_and_site() {
    let regs = {
        let mut regs = ExtRegistries::default();
        register_kernel_builtins(&mut regs);
        regs
    };

    let timeline = owned_builtin_timeline("core/deal_damage", "core/priary", "core/always");
    let err = validate_timeline_refs(&timeline, &regs).unwrap_err();
    assert_eq!(err.len(), 1);
    assert_eq!(err[0].axis, "selector");
    assert_eq!(err[0].missing_id, "core/priary");
    assert_eq!(err[0].site, "beat impact");
}

#[test]
fn builtin_predicate_typos_report_precise_axis_and_site() {
    let regs = {
        let mut regs = ExtRegistries::default();
        register_kernel_builtins(&mut regs);
        regs
    };

    let timeline = owned_builtin_timeline("core/deal_damage", "core/primary", "core/alwys");
    let err = validate_timeline_refs(&timeline, &regs).unwrap_err();
    assert_eq!(err.len(), 1);
    assert_eq!(err[0].axis, "predicate");
    assert_eq!(err[0].missing_id, "core/alwys");
    assert_eq!(err[0].site, "edge cast->impact");
}
