//! Agumon blueprint: custom-signal dispatch + passive hooks.
//!
//! Twin Core (shared with Gabumon) lives in `blueprints::twin_core`.

use std::collections::HashMap;
use std::sync::Arc;

use bevy::prelude::*;

use crate::combat::{
    api::{
        Beat, BeatEvent, BeatKind, BlueprintState, CompiledTimeline, EventFilter, Intent,
        PassiveListeners, PassiveRunner, SelectorCtx, SignalPayload, SignalTaxonomy, SkillCtx,
    },
    team::Team,
    types::{DamageTag, UnitId},
    unit::Unit,
};

pub mod signals;

/// Per-unit talent ranks. Keys are `"owner::talent_name"` (e.g. `"agumon::bouncing_fire"`).
/// Default rank is 0 (talent disabled).
#[derive(Resource, Default)]
pub struct TalentRanks(pub HashMap<String, u8>);

pub use crate::combat::blueprints::twin_core::{
    TAG_CHILLED, TAG_DEEP_CRACK, TAG_HEATED, TAG_MELTDOWN_CRACK, TAG_PRIMED, TAG_THERMAL_SPARK,
    TwinCoreDesignTag, TwinCoreHook, TwinCoreState, apply_twin_core_transitions_system,
    classify_twin_core_tag, twin_core_added_tag_transition, twin_core_design_tag,
    twin_core_design_tag_name,
};
pub use signals::{OWNER, dispatch};

const PASSIVE_SIGNAL_NAME: &str = "apply_heated";
const PASSIVE_TRIGGER_KEY: &str = "agumon/twin_core/triggered";
const PASSIVE_TIMELINE_ID: &str = "agumon_twin_core_passive";
const PASSIVE_OWNER: UnitId = UnitId(1);

pub struct AgumonPlugin;

/// Register only the Agumon extension-point functions (hooks, predicates, selectors)
/// into an `ExtRegistries` without requiring a full `App`.
/// Useful for timeline validation in tests that build a bare registry.
pub fn register_agumon_ext(regs: &mut crate::combat::api::ExtRegistries) {
    regs.predicates
        .register("agumon/twin_core/passive_trigger", passive_trigger);
    regs.hooks
        .register("agumon/twin_core/passive_proc", passive_proc);
    regs.predicates
        .register("agumon/has_bouncing_fire", has_bouncing_fire);
    regs.predicates.register("agumon/bounce_exit", bounce_exit);
    regs.selectors
        .register("agumon/bounce_pick_next", bounce_pick_next);
    regs.hooks.register("agumon/on_bounce_hop", on_bounce_hop);
}

pub fn register_validation_ext(regs: &mut crate::combat::api::ExtRegistries) {
    crate::combat::blueprints::twin_core::register_validation_ext(regs);
}

pub fn register_passive_runtime(app: &mut App) {
    app.init_resource::<TalentRanks>();
    register_passive_hooks(app);

    app.world_mut()
        .resource_mut::<SignalTaxonomy>()
        .register(OWNER, PASSIVE_SIGNAL_NAME);

    app.world_mut()
        .resource_mut::<PassiveListeners>()
        .runners
        .push(PassiveRunner::new(
            build_passive_timeline(),
            PASSIVE_OWNER,
            vec![EventFilter::blueprint("kernel", "ult_used")],
        ));
}

fn build_passive_timeline() -> Arc<CompiledTimeline> {
    Arc::new(CompiledTimeline {
        id: PASSIVE_TIMELINE_ID,
        entry: "dormant",
        beats: vec![
            Beat {
                id: "dormant",
                kind: BeatKind::Impact,
                hook: None,
                selector: None,
                presentation: None,
                payload: None,
            },
            Beat {
                id: "proc",
                kind: BeatKind::Impact,
                hook: Some("agumon/twin_core/passive_proc"),
                selector: None,
                presentation: None,
                payload: None,
            },
            Beat {
                id: "resolve",
                kind: BeatKind::Impact,
                hook: None,
                selector: None,
                presentation: None,
                payload: None,
            },
        ],
        edges: vec![
            crate::combat::api::timeline::BeatEdge {
                from: "dormant",
                to: "proc",
                gate: Some("agumon/twin_core/passive_trigger"),
            },
            crate::combat::api::timeline::BeatEdge {
                from: "proc",
                to: "resolve",
                gate: None,
            },
        ],
    })
}

fn passive_trigger(evt: &BeatEvent, ctx: &SkillCtx<'_>) -> bool {
    let world = ctx.world;

    let Some(mut units) = world.try_query::<(&Unit, &Team)>() else {
        return false;
    };

    let Some((_, target_team)) = units
        .iter(world)
        .find(|(unit, _)| unit.id == ctx.primary_target)
    else {
        return false;
    };

    let Some((self_unit, self_team)) = units.iter(world).find(|(unit, _)| unit.id == ctx.caster)
    else {
        return false;
    };

    let guard_key = (ctx.caster, PASSIVE_TRIGGER_KEY.to_string());
    let guard_written = world
        .resource::<BlueprintState>()
        .map
        .get(&guard_key)
        .copied()
        .unwrap_or_default()
        != 0;

    ctx.primary_target != ctx.caster
        && self_unit.hp_current > 0
        && self_team == target_team
        && evt.beat_id == "dormant"
        && !guard_written
}

fn passive_proc(evt: &BeatEvent, ctx: &mut SkillCtx<'_>) {
    ctx.enqueue(Intent::SetBlueprintState {
        actor: ctx.caster,
        key: PASSIVE_TRIGGER_KEY.to_string(),
        value: 1,
        cast_id: evt.cast_id,
    });
    ctx.enqueue(Intent::BlueprintSignal {
        source: ctx.caster,
        owner: OWNER,
        name: PASSIVE_SIGNAL_NAME,
        payload: SignalPayload::Amount(3),
        cast_id: evt.cast_id,
    });
}

// ─── Bouncing Fire talent: BeatKind::Loop branch on baby_flame ────────────────

/// Gate: true when Agumon has at least 1 rank in the Bouncing Fire talent.
fn has_bouncing_fire(_evt: &BeatEvent, ctx: &SkillCtx<'_>) -> bool {
    ctx.world
        .get_resource::<TalentRanks>()
        .map(|r| r.0.get("agumon::bouncing_fire").copied().unwrap_or(0) >= 1)
        .unwrap_or(false)
}

/// Exit predicate: returns true when no alive enemy outside `cast_hit_set` remains.
fn bounce_exit(_evt: &BeatEvent, ctx: &SkillCtx<'_>) -> bool {
    let world = ctx.world;
    let Some(mut units) = world.try_query::<(&Unit, &Team)>() else {
        return true;
    };

    let caster_team = units
        .iter(world)
        .find_map(|(unit, team)| (unit.id == ctx.caster).then_some(*team));

    let Some(caster_team) = caster_team else {
        return true;
    };

    !units.iter(world).any(|(unit, team)| {
        *team != caster_team && unit.hp_current > 0 && !ctx.cast_hit_set.contains(&unit.id)
    })
}

/// Selector: picks the first alive enemy not already in `cast_hit_set`.
/// Returns empty when no valid bounce target remains (terminates the loop).
fn bounce_pick_next(ctx: &SelectorCtx<'_>) -> Vec<UnitId> {
    let world = ctx.world;
    let Some(mut units) = world.try_query::<(&Unit, &Team)>() else {
        return vec![];
    };

    let caster_team = units
        .iter(world)
        .find_map(|(unit, team)| (unit.id == ctx.caster).then_some(*team));

    let Some(caster_team) = caster_team else {
        return vec![];
    };

    // find_map gives item by value, so team: &Team and *team: Team (no double-ref like find).
    units
        .iter(world)
        .find_map(|(unit, team)| {
            if *team != caster_team && unit.hp_current > 0 && !ctx.cast_hit_set.contains(&unit.id) {
                Some(vec![unit.id])
            } else {
                None
            }
        })
        .unwrap_or_default()
}

/// Hook: deals half of baby_flame's base damage (9) to each selected bounce target.
fn on_bounce_hop(evt: &BeatEvent, ctx: &mut SkillCtx<'_>) {
    for &target in &evt.beat_targets {
        ctx.enqueue(Intent::DealDamage {
            source: ctx.caster,
            target,
            amount: 9,
            tag: DamageTag::Fire,
            cast_id: evt.cast_id,
        });
    }
}

fn register_passive_hooks(app: &mut App) {
    let mut regs = app
        .world_mut()
        .resource_mut::<crate::combat::api::ExtRegistries>();
    register_agumon_ext(&mut regs);
}
