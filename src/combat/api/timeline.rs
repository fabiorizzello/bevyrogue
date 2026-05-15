use crate::combat::{api::intent::CastId, api::registry::ExtRegistries, types::UnitId};

/// Opaque identifier for a beat within a `CompiledTimeline`.
pub type BeatId = &'static str;

/// Data-only presentation descriptor carried on a beat.
///
/// Consumed by `Clock::Windowed` to drive animation/VFX/SFX.
/// Headless paths ignore it.
#[derive(Debug, Clone)]
pub struct Presentation {
    pub cue_id: &'static str,
    pub anim: Option<&'static str>,
    pub vfx: Option<&'static str>,
    pub sfx: Option<&'static str>,
}

/// Structural role of a beat in the timeline FSM.
#[derive(Debug, Clone)]
pub enum BeatKind {
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
        body: Vec<Beat>,
        /// Predicate ID resolved in `ExtRegistries::predicates`.
        exit_when: &'static str,
    },
}

/// One node in the timeline graph.
#[derive(Debug, Clone)]
pub struct Beat {
    pub id: BeatId,
    pub kind: BeatKind,
    /// Optional hook ID resolved in `ExtRegistries::hooks` — fires on beat entry.
    pub hook: Option<&'static str>,
    /// Optional selector ID resolved in `ExtRegistries::selectors` — populates `beat_targets`.
    pub selector: Option<&'static str>,
    pub presentation: Option<Presentation>,
}

/// Directed edge between two beats with an optional predicate gate.
#[derive(Debug, Clone)]
pub struct BeatEdge {
    pub from: BeatId,
    pub to: BeatId,
    /// Predicate ID resolved in `ExtRegistries::predicates`. `None` = unconditional.
    pub gate: Option<&'static str>,
}

/// Compiled, immutable representation of a skill's timeline FSM.
#[derive(Debug, Clone)]
pub struct CompiledTimeline {
    pub id: &'static str,
    pub entry: BeatId,
    pub beats: Vec<Beat>,
    pub edges: Vec<BeatEdge>,
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
pub fn validate_timeline_refs(
    timeline: &CompiledTimeline,
    regs: &ExtRegistries,
) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    for beat in &timeline.beats {
        validate_beat(beat, regs, &mut errors);
    }

    for edge in &timeline.edges {
        if let Some(gate) = edge.gate {
            if regs.predicates.get(gate).is_none() {
                errors.push(ValidationError {
                    axis: "predicate",
                    missing_id: gate.to_string(),
                    site: format!("edge {}->{}", edge.from, edge.to),
                });
            }
        }
    }

    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

fn validate_beat(beat: &Beat, regs: &ExtRegistries, errors: &mut Vec<ValidationError>) {
    let site = format!("beat {}", beat.id);

    if let Some(hook_id) = beat.hook {
        if regs.hooks.get(hook_id).is_none() {
            errors.push(ValidationError {
                axis: "hook",
                missing_id: hook_id.to_string(),
                site: site.clone(),
            });
        }
    }

    if let Some(selector_id) = beat.selector {
        if regs.selectors.get(selector_id).is_none() {
            errors.push(ValidationError {
                axis: "selector",
                missing_id: selector_id.to_string(),
                site: site.clone(),
            });
        }
    }

    if let BeatKind::Loop { body, exit_when } = &beat.kind {
        if regs.predicates.get(exit_when).is_none() {
            errors.push(ValidationError {
                axis: "predicate",
                missing_id: exit_when.to_string(),
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
            beats: vec![
                Beat {
                    id: "cast",
                    kind: BeatKind::Cast,
                    hook: Some("my_hook"),
                    selector: Some("my_selector"),
                    presentation: None,
                },
            ],
            edges: vec![
                BeatEdge { from: "cast", to: "impact", gate: Some("my_pred") },
            ],
        }
    }

    fn populated_regs() -> ExtRegistries {
        let mut regs = ExtRegistries::default();
        regs.hooks.register("my_hook", || {});
        regs.selectors.register("my_selector", || {});
        regs.predicates.register("my_pred", || {});
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
            beats: vec![
                Beat {
                    id: "cast",
                    kind: BeatKind::Cast,
                    hook: Some("nonexistent_hook"),
                    selector: None,
                    presentation: None,
                },
            ],
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
            beats: vec![
                Beat { id: "cast", kind: BeatKind::Cast, hook: None, selector: None, presentation: None },
            ],
            edges: vec![
                BeatEdge { from: "cast", to: "impact", gate: Some("missing_gate") },
            ],
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
            beats: vec![
                Beat {
                    id: "loop_beat",
                    kind: BeatKind::Loop {
                        body: vec![],
                        exit_when: "missing_exit_pred",
                    },
                    hook: None,
                    selector: None,
                    presentation: None,
                },
            ],
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
