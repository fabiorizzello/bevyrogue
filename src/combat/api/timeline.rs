use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

use crate::combat::{
    api::{intent::CastId, registry::ExtRegistries, signal::SignalPayload},
    status_effect::StatusEffectKind,
    types::{DamageTag, UnitId},
};

use crate::data::skills_ron::TargetShape;

/// Resource holding all registered `CompiledTimeline`s for boot-time validation.
///
/// Insert into the `App` before `App::finish()` runs (e.g. in blueprint `build`).
/// `CombatPlugin::finish` iterates this collection and panics on any dangling ref.
/// Wire-up of concrete timelines is S05's responsibility; an empty library is valid.
#[derive(Resource, Default)]
pub struct TimelineLibrary<Id = &'static str> {
    pub timelines: Vec<CompiledTimeline<Id>>,
}

/// Opaque identifier for a beat within a `CompiledTimeline`.
pub type BeatId = &'static str;

/// Minimal beat payload carried alongside a compiled beat.
///
/// Built-in hook functions read this through `SkillCtx` so a generic compiled
/// timeline can carry concrete effect parameters without encoding them into
/// the registry id string.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum BeatPayload {
    DealDamage {
        amount: i32,
        tag: DamageTag,
        target: TargetShape,
    },
    BreakToughness {
        amount: i32,
        tag: DamageTag,
        target: TargetShape,
    },
    ApplyStatus {
        kind: StatusEffectKind,
        duration: u32,
        target: TargetShape,
    },
    DelayTurn {
        amount_pct: u32,
        target: TargetShape,
    },
    AdvanceTurn {
        amount_pct: u32,
        target: TargetShape,
    },
    ApplyBuff {
        kind: StatusEffectKind,
        duration: u32,
        target: TargetShape,
    },
    Revive {
        pct: i32,
        target: TargetShape,
    },
    GrantFreeSkill {
        count: usize,
    },
    GrantEnergy {
        amount: i32,
    },
    SelfAdvance {
        amount_pct: i32,
    },
    BlueprintSignal {
        owner: String,
        name: String,
        payload: SignalPayload,
    },
}

/// Data-only presentation descriptor carried on a beat.
///
/// Consumed by `Clock::Windowed` to drive animation/VFX/SFX.
/// Headless paths ignore it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Presentation<Id = &'static str> {
    pub cue_id: Id,
    pub anim: Option<Id>,
    pub vfx: Option<Id>,
    pub sfx: Option<Id>,
}

/// Structural role of a beat in the timeline FSM.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BeatKind<Id = &'static str> {
    /// Wind-up phase before the skill resolves.
    Cast,
    /// Phase boundary (e.g. stance change).
    Phase,
    /// The beat where damage / effects land.
    Impact,
    /// Post-resolution cleanup or secondary reactions.
    Aftermath,
    /// Single-level loop: execute `body` beats until `exit_when` predicate returns true.
    Loop {
        body: Vec<Beat<Id>>,
        /// Predicate ID resolved in `ExtRegistries::predicates`.
        exit_when: Id,
    },
}

/// One node in the timeline graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Beat<Id = &'static str> {
    pub id: Id,
    pub kind: BeatKind<Id>,
    /// Optional hook ID resolved in `ExtRegistries::hooks` — fires on beat entry.
    pub hook: Option<Id>,
    /// Optional selector ID resolved in `ExtRegistries::selectors` — populates `beat_targets`.
    pub selector: Option<Id>,
    pub presentation: Option<Presentation<Id>>,
    /// Optional typed payload consumed by generic built-in hooks.
    #[serde(default)]
    pub payload: Option<BeatPayload>,
}

/// Directed edge between two beats with an optional predicate gate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeatEdge<Id = &'static str> {
    pub from: Id,
    pub to: Id,
    /// Predicate ID resolved in `ExtRegistries::predicates`. `None` = unconditional.
    pub gate: Option<Id>,
}

/// Compiled, immutable representation of a skill's timeline FSM.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledTimeline<Id = &'static str> {
    pub id: Id,
    pub entry: Id,
    pub beats: Vec<Beat<Id>>,
    pub edges: Vec<BeatEdge<Id>>,
}

/// Event emitted by the `BeatRunner` as each beat is processed.
///
/// `cast_id` propagates the S01 `CastId` invariant through the timeline.
/// `hop_index` is 0 outside a `BeatKind::Loop` body; counts iterations inside.
#[derive(Debug, Clone)]
pub struct BeatEvent {
    pub cast_id: CastId,
    pub beat_id: BeatId,
    pub hop_index: u32,
    pub beat_targets: Vec<UnitId>,
}

/// Read context passed to selector functions.
pub struct SelectorCtx<'a, S = ()> {
    pub caster: UnitId,
    pub primary_target: UnitId,
    pub state: &'a S,
}

/// Read context passed to cue resolver functions.
pub struct CueCtx<'a, S = ()> {
    pub caster: UnitId,
    pub state: &'a S,
}

/// A single dangling-reference error found by `validate_timeline_refs`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    /// Which registry axis the missing id belongs to: `"hook"`, `"selector"`, `"predicate"`.
    pub axis: &'static str,
    /// The unresolved id string.
    pub missing_id: String,
    /// Human-readable location: `"beat <id>"` or `"edge <from>-><to>"`.
    pub site: String,
}

/// Validate all hook/selector/predicate references in `timeline` against `regs`.
///
/// Recurses into `BeatKind::Loop` bodies. Collects all errors before returning.
pub fn validate_timeline_refs<Id: AsRef<str>>(
    timeline: &CompiledTimeline<Id>,
    regs: &ExtRegistries,
) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    for beat in &timeline.beats {
        validate_beat(beat, regs, &mut errors);
    }

    for edge in &timeline.edges {
        if let Some(gate) = edge.gate.as_ref() {
            if regs.predicates.get(gate.as_ref()).is_none() {
                errors.push(ValidationError {
                    axis: "predicate",
                    missing_id: gate.as_ref().to_string(),
                    site: format!("edge {}->{}", edge.from.as_ref(), edge.to.as_ref()),
                });
            }
        }
    }

    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

fn validate_beat<Id: AsRef<str>>(beat: &Beat<Id>, regs: &ExtRegistries, errors: &mut Vec<ValidationError>) {
    let site = format!("beat {}", beat.id.as_ref());

    if let Some(hook_id) = beat.hook.as_ref() {
        if regs.hooks.get(hook_id.as_ref()).is_none() {
            errors.push(ValidationError {
                axis: "hook",
                missing_id: hook_id.as_ref().to_string(),
                site: site.clone(),
            });
        }
    }

    if let Some(selector_id) = beat.selector.as_ref() {
        if regs.selectors.get(selector_id.as_ref()).is_none() {
            errors.push(ValidationError {
                axis: "selector",
                missing_id: selector_id.as_ref().to_string(),
                site: site.clone(),
            });
        }
    }

    if let BeatKind::Loop { body, exit_when } = &beat.kind {
        if regs.predicates.get(exit_when.as_ref()).is_none() {
            errors.push(ValidationError {
                axis: "predicate",
                missing_id: exit_when.as_ref().to_string(),
                site: site.clone(),
            });
        }
        for inner in body {
            validate_beat(inner, regs, errors);
        }
    }
}

// ─── Inline tests ──────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::api::registry::ExtRegistries;

    fn clean_timeline() -> CompiledTimeline {
        CompiledTimeline {
            id: "test_skill",
            entry: "cast",
            beats: vec![Beat {
                id: "cast",
                kind: BeatKind::Cast,
                hook: Some("my_hook"),
                selector: Some("my_selector"),
                presentation: None,
                payload: None,
            }],
            edges: vec![BeatEdge { from: "cast", to: "impact", gate: Some("my_pred") }],
        }
    }

    fn populated_regs() -> ExtRegistries {
        fn noop_hook(_: &BeatEvent, _: &mut crate::combat::api::skill_ctx::SkillCtx<'_>) {}
        fn noop_selector(_: &SelectorCtx<'_>) -> Vec<UnitId> { vec![] }
        fn noop_pred(_: &BeatEvent, _: &crate::combat::api::skill_ctx::SkillCtx<'_>) -> bool { false }
        let mut regs = ExtRegistries::default();
        regs.hooks.register("my_hook", noop_hook);
        regs.selectors.register("my_selector", noop_selector);
        regs.predicates.register("my_pred", noop_pred);
        regs
    }

    #[test]
    fn clean_timeline_validates_ok() {
        let tl = clean_timeline();
        let regs = populated_regs();
        assert!(validate_timeline_refs(&tl, &regs).is_ok());
    }

    #[test]
    fn missing_hook_returns_err_axis_hook() {
        let tl = CompiledTimeline {
            id: "bad_hook",
            entry: "cast",
            beats: vec![Beat {
                id: "cast",
                kind: BeatKind::Cast,
                hook: Some("nonexistent_hook"),
                selector: None,
                presentation: None,
                payload: None,
            }],
            edges: vec![],
        };
        let regs = ExtRegistries::default();
        let err = validate_timeline_refs(&tl, &regs).unwrap_err();
        assert_eq!(err.len(), 1);
        assert_eq!(err[0].axis, "hook");
        assert_eq!(err[0].missing_id, "nonexistent_hook");
        assert_eq!(err[0].site, "beat cast");
    }

    #[test]
    fn missing_edge_gate_returns_err_axis_predicate_with_edge_site() {
        let tl = CompiledTimeline {
            id: "bad_gate",
            entry: "cast",
            beats: vec![Beat { id: "cast", kind: BeatKind::Cast, hook: None, selector: None, presentation: None, payload: None }],
            edges: vec![BeatEdge { from: "cast", to: "impact", gate: Some("missing_gate") }],
        };
        let regs = ExtRegistries::default();
        let err = validate_timeline_refs(&tl, &regs).unwrap_err();
        assert_eq!(err.len(), 1);
        assert_eq!(err[0].axis, "predicate");
        assert_eq!(err[0].missing_id, "missing_gate");
        assert_eq!(err[0].site, "edge cast->impact");
    }

    #[test]
    fn missing_loop_exit_when_predicate_reported_with_loop_beat_site() {
        let tl = CompiledTimeline {
            id: "bad_loop",
            entry: "loop_beat",
            beats: vec![Beat {
                id: "loop_beat",
                kind: BeatKind::Loop {
                    body: vec![],
                    exit_when: "missing_exit_pred",
                },
                hook: None,
                selector: None,
                presentation: None,
                payload: None,
            }],
            edges: vec![],
        };
        let regs = ExtRegistries::default();
        let err = validate_timeline_refs(&tl, &regs).unwrap_err();
        assert_eq!(err.len(), 1);
        assert_eq!(err[0].axis, "predicate");
        assert_eq!(err[0].missing_id, "missing_exit_pred");
        assert_eq!(err[0].site, "beat loop_beat");
    }
}
