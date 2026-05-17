use std::sync::Arc;

use crate::combat::{
    api::{
        Beat, BeatEvent, BeatKind, BlueprintState, EventFilter, Intent, PassiveListeners,
        PassiveRunner, SignalPayload, SignalTaxonomy, SkillCtx,
    },
    team::Team,
    types::UnitId,
    unit::Unit,
};

mod signals;
pub use signals::dispatch;

pub const OWNER: &str = "gabumon";
const PASSIVE_SIGNAL_NAME: &str = "apply_chilled";
const PASSIVE_TRIGGER_KEY: &str = "gabumon/twin_core/triggered";
const PASSIVE_TIMELINE_ID: &str = "gabumon_twin_core_passive";
const PASSIVE_OWNER: UnitId = UnitId(2);

pub fn register_passive_runtime(app: &mut bevy::prelude::App) {
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

fn build_passive_timeline() -> Arc<crate::combat::api::CompiledTimeline> {
    Arc::new(crate::combat::api::CompiledTimeline {
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
                hook: Some("gabumon/twin_core/passive_proc"),
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
                gate: Some("gabumon/twin_core/passive_trigger"),
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

fn register_passive_hooks(app: &mut bevy::prelude::App) {
    let mut regs = app
        .world_mut()
        .resource_mut::<crate::combat::api::ExtRegistries>();
    regs.predicates
        .register("gabumon/twin_core/passive_trigger", passive_trigger);
    regs.hooks
        .register("gabumon/twin_core/passive_proc", passive_proc);
}
