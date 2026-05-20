use std::collections::HashSet;

use bevy::prelude::{Resource, World};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::combat::{
    runtime::{intent::CastId, registry::ExtRegistries, signal::SignalPayload},
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
    // Consumed by tests/timeline_chain_bolt_port.rs and tests/passive_event_filters.rs.
    pub hop_index: u32,
    pub beat_targets: Vec<UnitId>,
}

/// Read context passed to selector functions.
pub struct SelectorCtx<'a, S = ()> {
    pub caster: UnitId,
    pub primary_target: UnitId,
    // Consumed by tests/compiled_timeline_builtin_validation.rs (struct literal `state: &()`).
    pub state: &'a S,
    /// Read-only world access for complex selectors that need to query unit state.
    pub world: &'a World,
    /// Units already hit this cast — selectors must skip these to avoid repeats.
    pub cast_hit_set: &'a HashSet<UnitId>,
}

/// Read context passed to cue resolver functions.
pub struct CueCtx<'a, S = ()> {
    // Reserved public API for cue resolver functions; not yet consumed.
    pub caster: UnitId,
    // Reserved public API for cue resolver functions; not yet consumed.
    pub state: &'a S,
}

/// A single dangling-reference error found by `validate_timeline_refs`.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("[{axis}] missing '{missing_id}' at {site}")]
pub struct ValidationError {
    /// Which registry axis the missing id belongs to: `"hook"`, `"selector"`, `"predicate"`.
    pub axis: &'static str,
    /// The unresolved id string.
    pub missing_id: String,
    /// Human-readable location: `beat <id>` or `edge <from>-><to>`.
    pub site: String,
}

/// Aggregated dangling-reference report raised by `CombatPlugin::finish` when
/// one or more registered timelines reference unknown hook/selector/predicate ids.
///
/// `Display` lists every offending site on its own line so the failure can be
/// triaged from log text alone.
#[derive(Debug, Error)]
#[error("dangling timeline references ({} site{}):\n{}", .0.len(), if .0.len() == 1 { "" } else { "s" }, render_validation_errors(.0))]
pub struct DanglingTimelineRefs(pub Vec<ValidationError>);

fn render_validation_errors(errors: &[ValidationError]) -> String {
    errors
        .iter()
        .map(|e| format!("  {e}"))
        .collect::<Vec<_>>()
        .join("\n")
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

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_beat<Id: AsRef<str>>(
    beat: &Beat<Id>,
    regs: &ExtRegistries,
    errors: &mut Vec<ValidationError>,
) {
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
