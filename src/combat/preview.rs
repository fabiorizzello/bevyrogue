use std::{collections::VecDeque, sync::Arc};

use bevy::{log, prelude::*};

use crate::combat::{
    api::{
        intent::{CastId, Intent},
        registry::ExtRegistries,
        runner::{BeatRunner, StepOutcome},
        skill_ctx::SkillCtxMode,
        timeline::{Beat, BeatEdge, BeatKind, BeatPayload, CompiledTimeline, Presentation, TimelineLibrary},
    },
    types::{SkillId, UnitId},
};
use crate::data::{
    SkillBookHandle,
    skill_timeline::compile_skill_book_timelines,
    skills_ron::SkillBook,
};

fn intern_timeline_id(value: &str) -> &'static str {
    Box::leak(value.to_owned().into_boxed_str())
}

pub(crate) fn intern_compiled_timeline(
    timeline: &CompiledTimeline<String>,
) -> CompiledTimeline<&'static str> {
    fn intern_payload(payload: &BeatPayload) -> BeatPayload {
        match payload {
            BeatPayload::DealDamage { amount, tag, target } => BeatPayload::DealDamage {
                amount: *amount,
                tag: *tag,
                target: target.clone(),
            },
            BeatPayload::BreakToughness { amount, tag, target } => BeatPayload::BreakToughness {
                amount: *amount,
                tag: *tag,
                target: target.clone(),
            },
            BeatPayload::ApplyStatus { kind, duration, target } => BeatPayload::ApplyStatus {
                kind: kind.clone(),
                duration: *duration,
                target: target.clone(),
            },
            BeatPayload::DelayTurn { amount_pct, target } => BeatPayload::DelayTurn {
                amount_pct: *amount_pct,
                target: target.clone(),
            },
            BeatPayload::AdvanceTurn { amount_pct, target } => BeatPayload::AdvanceTurn {
                amount_pct: *amount_pct,
                target: target.clone(),
            },
            BeatPayload::ApplyBuff { kind, duration, target } => BeatPayload::ApplyBuff {
                kind: kind.clone(),
                duration: *duration,
                target: target.clone(),
            },
            BeatPayload::Revive { pct, target } => BeatPayload::Revive {
                pct: *pct,
                target: target.clone(),
            },
            BeatPayload::GrantFreeSkill { count } => BeatPayload::GrantFreeSkill { count: *count },
            BeatPayload::GrantEnergy { amount } => BeatPayload::GrantEnergy { amount: *amount },
            BeatPayload::SelfAdvance { amount_pct } => BeatPayload::SelfAdvance {
                amount_pct: *amount_pct,
            },
            BeatPayload::BlueprintSignal { owner, name, payload } => BeatPayload::BlueprintSignal {
                owner: owner.clone(),
                name: name.clone(),
                payload: payload.clone(),
            },
        }
    }

    fn intern_presentation(p: &Presentation<String>) -> Presentation<&'static str> {
        Presentation {
            cue_id: intern_timeline_id(&p.cue_id),
            anim: p.anim.as_deref().map(intern_timeline_id),
            vfx: p.vfx.as_deref().map(intern_timeline_id),
            sfx: p.sfx.as_deref().map(intern_timeline_id),
        }
    }

    fn intern_beat(beat: &Beat<String>) -> Beat<&'static str> {
        Beat {
            id: intern_timeline_id(&beat.id),
            kind: match &beat.kind {
                BeatKind::Cast => BeatKind::Cast,
                BeatKind::Phase => BeatKind::Phase,
                BeatKind::Impact => BeatKind::Impact,
                BeatKind::Aftermath => BeatKind::Aftermath,
                BeatKind::Loop { body, exit_when } => BeatKind::Loop {
                    body: body.iter().map(intern_beat).collect(),
                    exit_when: intern_timeline_id(exit_when),
                },
            },
            hook: beat.hook.as_deref().map(intern_timeline_id),
            selector: beat.selector.as_deref().map(intern_timeline_id),
            presentation: beat.presentation.as_ref().map(intern_presentation),
            payload: beat.payload.as_ref().map(intern_payload),
        }
    }

    CompiledTimeline {
        id: intern_timeline_id(&timeline.id),
        entry: intern_timeline_id(&timeline.entry),
        beats: timeline.beats.iter().map(intern_beat).collect(),
        edges: timeline
            .edges
            .iter()
            .map(|edge| BeatEdge {
                from: intern_timeline_id(&edge.from),
                to: intern_timeline_id(&edge.to),
                gate: edge.gate.as_deref().map(intern_timeline_id),
            })
            .collect(),
    }
}

pub(crate) fn resolve_compiled_skill_timeline(
    world: &World,
    skill_id: &SkillId,
    regs: &ExtRegistries,
) -> Option<Arc<CompiledTimeline>> {
    if let Some(timeline) = world
        .get_resource::<TimelineLibrary<String>>()
        .and_then(|library| {
            library
                .timelines
                .iter()
                .find(|timeline| timeline.id == skill_id.0)
                .cloned()
        })
    {
        return Some(Arc::new(intern_compiled_timeline(&timeline)));
    }

    let Some(book) = world
        .get_resource::<Assets<SkillBook>>()
        .and_then(|assets| {
            world
                .get_resource::<SkillBookHandle>()
                .and_then(|handle| assets.get(&handle.0))
        })
    else {
        log::warn!(
            "skill timeline {:?} skipped: compiled timeline not found and SkillBook unavailable",
            skill_id
        );
        return None;
    };

    let compiled = compile_skill_book_timelines(book, regs);
    let Ok(compiled) = compiled else {
        log::warn!(
            "skill timeline {:?} skipped: SkillBook timeline compile failed",
            skill_id
        );
        return None;
    };

    let Some(timeline) = compiled.into_iter().find(|timeline| timeline.id == skill_id.0) else {
        log::warn!(
            "skill timeline {:?} skipped: compiled timeline not found",
            skill_id
        );
        return None;
    };

    Some(Arc::new(intern_compiled_timeline(&timeline)))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PreviewDamageSummary {
    pub total_damage: i32,
    pub deal_damage_intents: usize,
}

/// Collapse a preview intent stream into a damage-only numeric summary.
///
/// This keeps the preview seam shared between UI and AI consumers: callers can
/// assert the pending stream shape separately and use this helper only for the
/// damage estimate they want to surface.
pub fn summarize_preview_damage(pending: &VecDeque<Intent>) -> PreviewDamageSummary {
    let mut summary = PreviewDamageSummary::default();

    for intent in pending {
        if let Intent::DealDamage { amount, .. } = intent {
            summary.total_damage += *amount;
            summary.deal_damage_intents += 1;
        }
    }

    summary
}

/// Build the pending intent stream for a skill cast without applying queued intents.
///
/// The helper runs the shared timeline through `BeatRunner` in `SkillCtxMode::Preview`
/// and returns the deferred `Intent` queue. Callers are responsible for any later
/// world mutation; this seam intentionally does not touch `intent_applier`.
pub fn try_query_skill_preview(
    world: &mut World,
    skill_id: &SkillId,
    cast_id: CastId,
    caster: UnitId,
    primary_target: UnitId,
) -> Option<VecDeque<Intent>> {
    let mut _fallback_regs = None;
    let regs_ptr: *const ExtRegistries = if let Some(regs) = world.get_resource::<ExtRegistries>() {
        regs as *const _
    } else {
        let mut regs = ExtRegistries::default();
        crate::combat::api::builtins::register_kernel_builtins(&mut regs);
        _fallback_regs = Some(regs);
        _fallback_regs
            .as_ref()
            .expect("fallback ext registries initialized") as *const _
    };

    let regs = unsafe { &*regs_ptr };
    let Some(timeline) = resolve_compiled_skill_timeline(world, skill_id, regs) else {
        return None;
    };

    let mut pending = VecDeque::new();
    let mut runner = BeatRunner::new(timeline, cast_id, caster, primary_target);
    let outcome = runner.run_to_completion(world, regs, SkillCtxMode::Preview, &mut pending, 1024);

    if outcome != StepOutcome::Done {
        log::warn!(
            "skill preview {:?} ended with {:?}",
            skill_id,
            outcome
        );
    }

    Some(pending)
}

/// Build the pending intent stream for a skill cast without applying queued intents.
///
/// This convenience wrapper preserves the existing call sites that only care about the
/// preview stream and can treat an unavailable preview as an empty queue.
pub fn query_skill_preview(
    world: &mut World,
    skill_id: &SkillId,
    cast_id: CastId,
    caster: UnitId,
    primary_target: UnitId,
) -> VecDeque<Intent> {
    try_query_skill_preview(world, skill_id, cast_id, caster, primary_target).unwrap_or_default()
}
