//! Patamon blueprint: custom-signal dispatch + identity (Holy Support) wiring.
//!
//! `PatamonPlugin` owns Patamon-specific kernel-runtime registrations
//! (Holy Support resource, applier system, hook) so adding or removing
//! the digimon is a single `add_plugins` line at the call site.

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

pub(crate) const SIGNAL_BUILD_HOLY_SUPPORT_GRACE: &str = "build_holy_support_grace";
pub(crate) const SIGNAL_SPEND_HOLY_SUPPORT_GRACE: &str = "spend_holy_support_grace";
pub(crate) const SIGNAL_MARK_MARTYR_LIGHT: &str = "mark_martyr_light";
pub(crate) const SIGNAL_CONSUME_MARTYR_LIGHT: &str = "consume_martyr_light";
pub(crate) const SIGNAL_CYCLE_RESET: &str = "cycle_reset";

pub use identity::{
    GRACE_CAP, HolySupportDesignTag, HolySupportHook, HolySupportRejectReason, HolySupportSnapshot,
    HolySupportState, HolySupportStep, HolySupportTransition, TAG_GRACE, TAG_MARTYR_LIGHT,
    apply_holy_support_transitions_system, classify_holy_support_tag,
    holy_support_added_tag_transition, holy_support_design_tag, holy_support_design_tag_name,
};
pub use signals::{OWNER, dispatch};

const PASSIVE_SIGNAL_NAME: &str = SIGNAL_BUILD_HOLY_SUPPORT_GRACE;
const PASSIVE_TRIGGER_KEY: &str = "patamon/holy_support/triggered";
const PASSIVE_TIMELINE_ID: &str = "patamon_holy_support_passive";
const PASSIVE_OWNER: UnitId = UnitId(9);

pub struct PatamonPlugin;

impl Plugin for PatamonPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<identity::HolySupportState>()
            .add_systems(Update, identity::apply_holy_support_transitions_system);

        app.world_mut()
            .resource_mut::<CombatKernelRegistry>()
            .register(identity::HolySupportHook);
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
                hook: Some("patamon/holy_support/passive_proc"),
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
                gate: Some("patamon/holy_support/passive_trigger"),
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
        payload: SignalPayload::Amount(1),
        cast_id: evt.cast_id,
    });
}

fn register_passive_hooks(app: &mut App) {
    let mut regs = app
        .world_mut()
        .resource_mut::<crate::combat::api::ExtRegistries>();
    regs.predicates
        .register("patamon/holy_support/passive_trigger", passive_trigger);
    regs.hooks
        .register("patamon/holy_support/passive_proc", passive_proc);
}
