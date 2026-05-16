use std::sync::Arc;

use crate::combat::{
    api::{
        Beat, BeatEvent, BeatKind, BlueprintState, CompiledTimeline, EventFilter, Intent,
        PassiveListeners, PassiveRunner, SignalPayload, SignalTaxonomy, SkillCtx,
    },
    team::Team,
    types::UnitId,
    unit::Unit,
};
use crate::data::skills_ron::SkillCustomSignal;

use super::CustomSignalDispatchError;

pub const OWNER: &str = "renamon";
const PASSIVE_SIGNAL_NAME: &str = "kitsune_grace";
const PASSIVE_TRIGGER_KEY: &str = "renamon/kitsune_grace/triggered";
const PASSIVE_TIMELINE_ID: &str = "renamon_kitsune_grace_passive";
const PASSIVE_OWNER: UnitId = UnitId(7);

pub fn dispatch(
    signal: &SkillCustomSignal,
    _action: &crate::combat::state::ResolvedAction,
) -> Result<Vec<crate::combat::kernel::CombatKernelTransition>, CustomSignalDispatchError> {
    if signal.owner() != OWNER {
        return Err(CustomSignalDispatchError::UnknownOwner {
            owner: signal.owner().to_owned(),
        });
    }

    match signal.signal() {
        "open_momentum_window" => Ok(vec![crate::combat::kernel::CombatKernelTransition::PrecisionMindGame(
            crate::combat::kernel::PrecisionMindGameTransition::open_window(
                crate::combat::kernel::PrecisionWindowKind::Momentum,
            ),
        )]),
        "commit_precision_press" => Ok(vec![crate::combat::kernel::CombatKernelTransition::PrecisionMindGame(
            crate::combat::kernel::PrecisionMindGameTransition::commit(
                crate::combat::kernel::PrecisionCommitment::Press,
            ),
        )]),
        "reveal_bait" => Ok(vec![crate::combat::kernel::CombatKernelTransition::PrecisionMindGame(
            crate::combat::kernel::PrecisionMindGameTransition::reveal(
                crate::combat::kernel::PrecisionReveal::Baited,
            ),
        )]),
        "resolve_precision_success" => Ok(vec![crate::combat::kernel::CombatKernelTransition::PrecisionMindGame(
            crate::combat::kernel::PrecisionMindGameTransition::resolve(
                crate::combat::kernel::PrecisionOutcome::Success,
            ),
        )]),
        other => Err(CustomSignalDispatchError::UnknownSignal {
            owner: OWNER.to_string(),
            signal: other.to_string(),
        }),
    }
}

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
                hook: Some("renamon/kitsune_grace/passive_proc"),
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
                gate: Some("renamon/kitsune_grace/passive_trigger"),
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
        payload: SignalPayload::UnitTarget(ctx.primary_target),
        cast_id: evt.cast_id,
    });
}

fn register_passive_hooks(app: &mut bevy::prelude::App) {
    let mut regs = app.world_mut().resource_mut::<crate::combat::api::ExtRegistries>();
    regs.predicates
        .register("renamon/kitsune_grace/passive_trigger", passive_trigger);
    regs.hooks.register("renamon/kitsune_grace/passive_proc", passive_proc);
}
