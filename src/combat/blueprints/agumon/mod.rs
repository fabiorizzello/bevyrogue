//! Agumon blueprint: custom-signal dispatch + identity (Twin Core) wiring.
//!
//! `AgumonPlugin` owns Agumon-specific kernel-runtime registrations (Twin
//! Core resource, applier system, hook) so adding or removing the digimon is
//! a single `add_plugins` line at the call site. Twin Core is shared with
//! Gabumon (paired Fire/Ice identity); Agumon owns the registration as the
//! Fire half.

use std::sync::Arc;

use bevy::prelude::*;

use crate::combat::{
    api::{
        Beat, BeatEvent, BeatKind, BlueprintState, CompiledTimeline, EventFilter, Intent,
        PassiveListeners, PassiveRunner, SignalPayload, SignalTaxonomy, SkillCtx,
    },
    kernel::CombatKernelRegistry,
    team::Team,
    types::UnitId,
    unit::Unit,
};

pub mod identity;
pub mod signals;

pub use identity::{
    TAG_CHILLED, TAG_DEEP_CRACK, TAG_HEATED, TAG_MELTDOWN_CRACK, TAG_PRIMED, TAG_THERMAL_SPARK,
    TwinCoreDesignTag, TwinCoreHook, TwinCoreState,
    apply_twin_core_transitions_system, classify_twin_core_tag,
    twin_core_added_tag_transition, twin_core_design_tag, twin_core_design_tag_name,
};
pub use signals::{OWNER, dispatch};

const PASSIVE_SIGNAL_NAME: &str = "apply_heated";
const PASSIVE_TRIGGER_KEY: &str = "agumon/twin_core/triggered";
const PASSIVE_TIMELINE_ID: &str = "agumon_twin_core_passive";
const PASSIVE_OWNER: UnitId = UnitId(1);

pub struct AgumonPlugin;

impl Plugin for AgumonPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<identity::TwinCoreState>()
            .add_systems(Update, identity::apply_twin_core_transitions_system);

        app.world_mut()
            .resource_mut::<CombatKernelRegistry>()
            .register(identity::TwinCoreHook);
    }
}

pub fn register_passive_runtime(app: &mut App) {
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

    let Some((self_unit, self_team)) = units
        .iter(world)
        .find(|(unit, _)| unit.id == ctx.caster)
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

fn register_passive_hooks(app: &mut App) {
    let mut regs = app.world_mut().resource_mut::<crate::combat::api::ExtRegistries>();
    regs.predicates
        .register("agumon/twin_core/passive_trigger", passive_trigger);
    regs.hooks.register("agumon/twin_core/passive_proc", passive_proc);
}
