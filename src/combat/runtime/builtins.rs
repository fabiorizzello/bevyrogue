use bevy::prelude::World;

use crate::combat::{
    runtime::{
        intent::Intent,
        registry::ExtRegistries,
        timeline::{BeatEvent, BeatPayload, SelectorCtx},
    },
    resolution::{TargetEntry, TargetableSnapshot, resolve_targets},
    team::Team,
    types::UnitId,
    unit::{Ko, SlotIndex, Unit},
};
use crate::data::skills_ron::TargetShape;

/// Register the kernel's built-in timeline extension functions.
///
/// These are the canonical ids that asset-backed compiled timelines can rely on
/// without any blueprint-specific registration.
pub fn register_kernel_builtins(regs: &mut ExtRegistries) {
    regs.hooks.register("core/deal_damage", deal_damage);
    regs.hooks.register("core/break_toughness", break_toughness);
    regs.hooks.register("core/apply_status", apply_status);
    regs.hooks.register("core/delay_turn", delay_turn);
    regs.hooks.register("core/advance_turn", advance_turn);
    regs.hooks.register("core/revive", revive);
    regs.hooks
        .register("core/grant_free_skill", grant_free_skill);
    regs.hooks.register("core/grant_energy", grant_energy);
    regs.hooks.register("core/self_advance", self_advance);
    regs.hooks.register("core/apply_effect", apply_effect);
    regs.selectors
        .register("core/primary", select_primary_target);
    regs.selectors.register("core/caster", select_caster_target);
    regs.predicates.register("core/always", always_true);
    regs.predicates.register("core/never", always_false);
}

fn select_primary_target(ctx: &SelectorCtx<'_>) -> Vec<UnitId> {
    vec![ctx.primary_target]
}

fn select_caster_target(ctx: &SelectorCtx<'_>) -> Vec<UnitId> {
    vec![ctx.caster]
}

fn always_true(_: &BeatEvent, _: &crate::combat::runtime::SkillCtx<'_>) -> bool {
    true
}

fn always_false(_: &BeatEvent, _: &crate::combat::runtime::SkillCtx<'_>) -> bool {
    false
}

fn deal_damage(evt: &BeatEvent, ctx: &mut crate::combat::runtime::SkillCtx<'_>) {
    let Some(BeatPayload::DealDamage {
        amount,
        tag,
        target,
    }) = ctx.beat_payload()
    else {
        panic!(
            "core/deal_damage requires BeatPayload::DealDamage at beat `{}`",
            evt.beat_id
        );
    };

    let caster = ctx.caster;
    let cast_id = ctx.cast_id;
    enqueue_targets(evt, ctx, *target, move |target_id| Intent::DealDamage {
        source: caster,
        target: target_id,
        amount: *amount,
        tag: *tag,
        cast_id,
    });
}

fn break_toughness(evt: &BeatEvent, ctx: &mut crate::combat::runtime::SkillCtx<'_>) {
    let Some(BeatPayload::BreakToughness {
        amount,
        tag,
        target,
    }) = ctx.beat_payload()
    else {
        panic!(
            "core/break_toughness requires BeatPayload::BreakToughness at beat `{}`",
            evt.beat_id
        );
    };

    let caster = ctx.caster;
    let cast_id = ctx.cast_id;
    enqueue_targets(evt, ctx, *target, move |target_id| Intent::BreakToughness {
        source: caster,
        target: target_id,
        amount: *amount,
        tag: *tag,
        cast_id,
    });
}

fn apply_status(evt: &BeatEvent, ctx: &mut crate::combat::runtime::SkillCtx<'_>) {
    let Some(BeatPayload::ApplyStatus {
        kind,
        duration,
        target,
    }) = ctx.beat_payload()
    else {
        panic!(
            "core/apply_status requires BeatPayload::ApplyStatus at beat `{}`",
            evt.beat_id
        );
    };

    let caster = ctx.caster;
    let cast_id = ctx.cast_id;
    enqueue_targets(evt, ctx, *target, move |target_id| Intent::ApplyStatus {
        source: caster,
        target: target_id,
        kind: kind.clone(),
        duration_turns: *duration,
        cast_id,
    });
}

fn delay_turn(evt: &BeatEvent, ctx: &mut crate::combat::runtime::SkillCtx<'_>) {
    let Some(BeatPayload::DelayTurn { amount_pct, target }) = ctx.beat_payload() else {
        panic!(
            "core/delay_turn requires BeatPayload::DelayTurn at beat `{}`",
            evt.beat_id
        );
    };

    let cast_id = ctx.cast_id;
    enqueue_targets(evt, ctx, *target, move |target_id| Intent::DelayTurn {
        target: target_id,
        amount_pct: (*amount_pct).min(50),
        cast_id,
    });
}

fn advance_turn(evt: &BeatEvent, ctx: &mut crate::combat::runtime::SkillCtx<'_>) {
    let Some(BeatPayload::AdvanceTurn { amount_pct, target }) = ctx.beat_payload() else {
        panic!(
            "core/advance_turn requires BeatPayload::AdvanceTurn at beat `{}`",
            evt.beat_id
        );
    };

    let cast_id = ctx.cast_id;
    enqueue_targets(evt, ctx, *target, move |target_id| Intent::AdvanceTurn {
        target: target_id,
        amount_pct: (*amount_pct).min(50),
        cast_id,
    });
}

fn revive(evt: &BeatEvent, ctx: &mut crate::combat::runtime::SkillCtx<'_>) {
    let Some(BeatPayload::Revive { pct, target }) = ctx.beat_payload() else {
        panic!(
            "core/revive requires BeatPayload::Revive at beat `{}`",
            evt.beat_id
        );
    };

    let caster = ctx.caster;
    let cast_id = ctx.cast_id;
    enqueue_targets(evt, ctx, *target, move |target_id| Intent::Revive {
        source: caster,
        target: target_id,
        pct: *pct,
        cast_id,
    });
}

fn grant_free_skill(evt: &BeatEvent, ctx: &mut crate::combat::runtime::SkillCtx<'_>) {
    let Some(BeatPayload::GrantFreeSkill { count }) = ctx.beat_payload() else {
        panic!(
            "core/grant_free_skill requires BeatPayload::GrantFreeSkill at beat `{}`",
            evt.beat_id
        );
    };

    if *count == 0 {
        return;
    }

    ctx.enqueue(Intent::GrantFreeSkill {
        source: ctx.caster,
        count: *count,
        cast_id: ctx.cast_id,
    });
}

fn grant_energy(evt: &BeatEvent, ctx: &mut crate::combat::runtime::SkillCtx<'_>) {
    let Some(BeatPayload::GrantEnergy { amount }) = ctx.beat_payload() else {
        panic!(
            "core/grant_energy requires BeatPayload::GrantEnergy at beat `{}`",
            evt.beat_id
        );
    };

    if *amount == 0 {
        return;
    }

    ctx.enqueue(Intent::AddEnergy {
        target: ctx.caster,
        amount: *amount,
        cast_id: ctx.cast_id,
    });
}

fn self_advance(evt: &BeatEvent, ctx: &mut crate::combat::runtime::SkillCtx<'_>) {
    let Some(BeatPayload::SelfAdvance { amount_pct }) = ctx.beat_payload() else {
        panic!(
            "core/self_advance requires BeatPayload::SelfAdvance at beat `{}`",
            evt.beat_id
        );
    };

    let capped = (*amount_pct).clamp(0, 50) as u32;
    if capped == 0 {
        return;
    }

    ctx.enqueue(Intent::AdvanceTurn {
        target: ctx.caster,
        amount_pct: capped,
        cast_id: ctx.cast_id,
    });
}

fn apply_effect(evt: &BeatEvent, ctx: &mut crate::combat::runtime::SkillCtx<'_>) {
    match ctx.beat_payload() {
        Some(BeatPayload::DealDamage { .. }) => deal_damage(evt, ctx),
        Some(BeatPayload::BreakToughness { .. }) => break_toughness(evt, ctx),
        Some(BeatPayload::ApplyStatus { .. }) => apply_status(evt, ctx),
        Some(BeatPayload::DelayTurn { .. }) => delay_turn(evt, ctx),
        Some(BeatPayload::AdvanceTurn { .. }) => advance_turn(evt, ctx),
        Some(BeatPayload::Revive { .. }) => revive(evt, ctx),
        Some(BeatPayload::GrantFreeSkill { .. }) => grant_free_skill(evt, ctx),
        Some(BeatPayload::GrantEnergy { .. }) => grant_energy(evt, ctx),
        Some(BeatPayload::SelfAdvance { .. }) => self_advance(evt, ctx),
        Some(BeatPayload::ApplyBuff {
            kind,
            duration,
            target,
        }) => {
            let cast_id = ctx.cast_id;
            enqueue_targets(evt, ctx, *target, move |target_id| Intent::ApplyBuff {
                target: target_id,
                kind: kind.clone(),
                duration_turns: *duration,
                cast_id,
            });
        }
        Some(BeatPayload::BlueprintSignal {
            owner,
            name,
            payload,
        }) => {
            let owner: &'static str = Box::leak(owner.clone().into_boxed_str());
            let name: &'static str = Box::leak(name.clone().into_boxed_str());
            ctx.enqueue(Intent::BlueprintSignal {
                source: ctx.caster,
                owner,
                name,
                payload: payload.clone(),
                cast_id: ctx.cast_id,
            });
        }
        None => {
            panic!(
                "core/apply_effect requires a BeatPayload at beat `{}`",
                evt.beat_id
            );
        }
    }
}

fn enqueue_targets<F>(
    evt: &BeatEvent,
    ctx: &mut crate::combat::runtime::SkillCtx<'_>,
    target: TargetShape,
    mut build: F,
) where
    F: FnMut(UnitId) -> Intent,
{
    let targets = if !evt.beat_targets.is_empty() {
        evt.beat_targets.clone()
    } else {
        let snapshot = snapshot_targets(ctx.world);
        let resolved = resolve_targets(&target, ctx.primary_target, &snapshot);
        if resolved.is_empty() {
            vec![ctx.primary_target]
        } else {
            resolved
        }
    };

    for target_id in targets {
        ctx.enqueue(build(target_id));
    }
}

fn snapshot_targets(world: &World) -> TargetableSnapshot {
    let mut q = match world.try_query::<(&Unit, &Team, Option<&Ko>, Option<&SlotIndex>)>() {
        Some(q) => q,
        None => return TargetableSnapshot { entries: vec![] },
    };
    let entries = q
        .iter(world)
        .map(|(unit, team, ko, slot)| TargetEntry {
            id: unit.id,
            team: *team,
            slot_index: slot.map(|s| s.0).unwrap_or(0),
            alive: ko.is_none() && unit.hp_current > 0,
            hp_per_mille: if unit.hp_max > 0 {
                ((unit.hp_current.max(0) as u64 * 1000) / unit.hp_max as u64) as u32
            } else {
                0
            },
        })
        .collect();
    TargetableSnapshot { entries }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::runtime::timeline::{Beat, BeatEdge, BeatKind, CompiledTimeline};
    use crate::combat::runtime::{SignalPayload, intent::CastId, timeline::BeatPayload, validate_timeline_refs};
    use bevy::prelude::World;
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
                        tag: crate::combat::types::DamageTag::Fire,
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
            tag: crate::combat::types::DamageTag::Physical,
            target: TargetShape::Single,
        };
        let mut ctx = crate::combat::runtime::SkillCtx::new(
            UnitId(1),
            UnitId(2),
            CastId(NonZeroU32::new(7).unwrap()),
            crate::combat::runtime::SkillCtxMode::Execute,
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

        let mut ctx = crate::combat::runtime::SkillCtx::new(
            caster,
            revive_target,
            cast_id,
            crate::combat::runtime::SkillCtxMode::Execute,
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
        let mut ctx = crate::combat::runtime::SkillCtx::new(
            caster,
            revive_target,
            cast_id,
            crate::combat::runtime::SkillCtxMode::Execute,
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
        let mut ctx = crate::combat::runtime::SkillCtx::new(
            caster,
            revive_target,
            cast_id,
            crate::combat::runtime::SkillCtxMode::Execute,
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
        let mut ctx = crate::combat::runtime::SkillCtx::new(
            caster,
            revive_target,
            cast_id,
            crate::combat::runtime::SkillCtxMode::Execute,
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
        let mut ctx = crate::combat::runtime::SkillCtx::new(
            UnitId(1),
            UnitId(2),
            CastId(NonZeroU32::new(7).unwrap()),
            crate::combat::runtime::SkillCtxMode::Execute,
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
}
