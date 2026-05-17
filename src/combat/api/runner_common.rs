use std::collections::{HashSet, VecDeque};
use std::sync::Arc;

use bevy::prelude::World;

use crate::combat::{
    api::{
        intent::{CastId, Intent},
        registry::ExtRegistries,
        skill_ctx::{SkillCtx, SkillCtxMode},
        timeline::{Beat, BeatEvent, BeatId, BeatKind, CompiledTimeline, SelectorCtx},
    },
    types::UnitId,
};

pub(crate) fn find_beat<'t>(timeline: &'t CompiledTimeline, id: BeatId) -> &'t Beat {
    timeline
        .beats
        .iter()
        .find(|b| b.id == id)
        .unwrap_or_else(|| panic!("beat `{id}` not found in timeline `{}`", timeline.id))
}

pub struct RunnerParams<'a, 'w> {
    pub timeline: &'a Arc<CompiledTimeline>,
    pub caster: UnitId,
    pub primary_target: UnitId,
    pub cast_id: CastId,
    pub cast_hit_set: &'a mut HashSet<UnitId>,
    pub world: &'w World,
    pub regs: &'a ExtRegistries,
    pub mode: SkillCtxMode,
    pub pending: &'a mut VecDeque<Intent>,
}

/// Execute one beat: resolve selector (Impact only), fire hook, fold DealDamage hits.
pub(crate) fn fire_beat(beat: &Beat, hop_index: u32, params: RunnerParams) -> Vec<UnitId> {
    // Selector — only Impact beats resolve targets.
    let beat_targets = if matches!(beat.kind, BeatKind::Impact) {
        if let Some(sel_id) = beat.selector.as_ref() {
            let sel = *params
                .regs
                .selectors
                .get(sel_id.as_ref())
                .unwrap_or_else(|| {
                    panic!(
                        "selector `{sel_id}` not registered \
                     (validate_timeline_refs catches this at App::finish)"
                    )
                });
            let sctx = SelectorCtx {
                caster: params.caster,
                primary_target: params.primary_target,
                state: &(),
                world: params.world,
                cast_hit_set: params.cast_hit_set,
            };
            sel(&sctx)
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // Hook.
    if let Some(hook_id) = beat.hook.as_ref() {
        let f = *params.regs.hooks.get(hook_id.as_ref()).unwrap_or_else(|| {
            panic!(
                "hook `{hook_id}` not registered \
                 (validate_timeline_refs catches this at App::finish)"
            )
        });
        let evt = BeatEvent {
            cast_id: params.cast_id,
            beat_id: beat.id,
            hop_index,
            beat_targets: beat_targets.clone(),
        };
        let prev_len = params.pending.len();
        {
            let mut ctx = SkillCtx::new(
                params.caster,
                params.primary_target,
                params.cast_id,
                params.mode,
                params.regs,
                params.world,
                params.cast_hit_set,
                params.pending,
                beat.payload.as_ref(),
            );
            f(&evt, &mut ctx);
        }
        // F6 fix: fold newly enqueued DealDamage targets into cast_hit_set so
        // subsequent bounce / NoRepeat selectors skip already-hit units.
        for i in prev_len..params.pending.len() {
            if let Some(Intent::DealDamage { target, .. }) = params.pending.get(i) {
                params.cast_hit_set.insert(*target);
            }
        }
    }

    beat_targets
}

/// Evaluate a registered predicate, providing a fresh (read-only) `SkillCtx`.
///
/// Any intents the predicate erroneously enqueues are discarded (dummy queue).
pub(crate) fn eval_predicate(pred_id: &str, evt: &BeatEvent, params: &mut RunnerParams) -> bool {
    let f = *params
        .regs
        .predicates
        .get(pred_id)
        .unwrap_or_else(|| panic!("predicate `{pred_id}` not registered"));
    let mut dummy: VecDeque<Intent> = VecDeque::new();
    let ctx = SkillCtx::new(
        params.caster,
        params.primary_target,
        params.cast_id,
        params.mode,
        params.regs,
        params.world,
        params.cast_hit_set,
        &mut dummy,
        None,
    );
    f(evt, &ctx)
}

/// Pick the next beat by walking outgoing edges from `from` in declaration order.
///
/// F1 fallback-edge rule: edges are tested left-to-right; the first edge whose
/// gate predicate is absent (`None`) or returns `true` is selected. An
/// unconditional edge placed last acts as the implicit fallback / default
/// transition. Returns `None` when the timeline has no more beats.
pub(crate) fn next_beat(
    from: BeatId,
    evt: &BeatEvent,
    params: &mut RunnerParams,
) -> Option<BeatId> {
    for edge in params.timeline.edges.iter().filter(|e| e.from == from) {
        let passes = match edge.gate {
            None => true,
            Some(pred_id) => eval_predicate(pred_id, evt, params),
        };
        if passes {
            return Some(edge.to);
        }
    }
    None
}
