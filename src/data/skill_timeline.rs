use std::{collections::HashSet, fmt};

use serde::{Deserialize, Serialize};

use crate::combat::api::{
    Beat, BeatEdge, BeatKind, CompiledTimeline, ExtRegistries, ValidationError,
    validate_timeline_refs,
};
use crate::combat::types::SkillId;

use super::skills_ron::{SkillBook, SkillDef};

/// Optional SkillBook-side timeline schema.
///
/// The schema stays close to the compiled kernel shape, but keeps the asset
/// field optional so legacy skills can continue to rely on the effect pipeline
/// until S06 migrates them incrementally.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillTimeline {
    pub entry: String,
    pub beats: Vec<Beat<String>>,
    pub edges: Vec<BeatEdge<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillTimelineCompileError {
    pub skill_id: SkillId,
    pub site: String,
    pub detail: String,
}

impl SkillTimelineCompileError {
    fn new(skill_id: &SkillId, site: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            skill_id: skill_id.clone(),
            site: site.into(),
            detail: detail.into(),
        }
    }
}

impl fmt::Display for SkillTimelineCompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "skill_id={} site={} detail={}",
            self.skill_id.0, self.site, self.detail
        )
    }
}

impl std::error::Error for SkillTimelineCompileError {}

/// Compile every timeline-backed skill in `book` into the kernel representation.
///
/// Skills without a `timeline` field are ignored so catalog migration can happen
/// incrementally. Any compiler or registry-reference error is returned with the
/// owning skill id plus the exact beat/edge site.
pub fn compile_skill_book_timelines(
    book: &SkillBook,
    regs: &ExtRegistries,
) -> Result<Vec<CompiledTimeline<String>>, SkillTimelineCompileError> {
    let mut compiled = Vec::new();

    for skill in &book.0 {
        if let Some(timeline) = skill.timeline.as_ref() {
            compiled.push(compile_skill_timeline(skill, timeline, regs)?);
        }
    }

    Ok(compiled)
}

fn compile_skill_timeline(
    skill: &SkillDef,
    timeline: &SkillTimeline,
    regs: &ExtRegistries,
) -> Result<CompiledTimeline<String>, SkillTimelineCompileError> {
    validate_timeline_structure(skill, timeline)?;

    let compiled = CompiledTimeline {
        id: skill.id.0.clone(),
        entry: timeline.entry.clone(),
        beats: timeline.beats.clone(),
        edges: timeline.edges.clone(),
    };

    if let Err(errors) = validate_timeline_refs(&compiled, regs) {
        return Err(compilation_error_from_refs(skill, &errors[0]));
    }

    Ok(compiled)
}

fn validate_timeline_structure(
    skill: &SkillDef,
    timeline: &SkillTimeline,
) -> Result<(), SkillTimelineCompileError> {
    let mut ids = HashSet::new();
    let mut entry_found = false;

    for beat in &timeline.beats {
        if beat.id == timeline.entry {
            entry_found = true;
        }

        if !ids.insert(beat.id.as_str()) {
            return Err(SkillTimelineCompileError::new(
                &skill.id,
                format!("beat {}", beat.id),
                format!("duplicate beat id `{}` in compiled timeline", beat.id),
            ));
        }

        validate_beat_tree(skill, beat)?;
    }

    if !entry_found {
        return Err(SkillTimelineCompileError::new(
            &skill.id,
            "timeline entry",
            format!(
                "entry `{}` does not match any top-level beat id",
                timeline.entry
            ),
        ));
    }

    for edge in &timeline.edges {
        if !ids.contains(edge.from.as_str()) {
            return Err(SkillTimelineCompileError::new(
                &skill.id,
                format!("edge {}->{}", edge.from, edge.to),
                format!(
                    "edge source `{}` is not declared as a top-level beat",
                    edge.from
                ),
            ));
        }
        if !ids.contains(edge.to.as_str()) {
            return Err(SkillTimelineCompileError::new(
                &skill.id,
                format!("edge {}->{}", edge.from, edge.to),
                format!(
                    "edge target `{}` is not declared as a top-level beat",
                    edge.to
                ),
            ));
        }
    }

    Ok(())
}

fn validate_beat_tree(
    skill: &SkillDef,
    beat: &Beat<String>,
) -> Result<(), SkillTimelineCompileError> {
    if let BeatKind::Loop { body, .. } = &beat.kind {
        let mut seen = HashSet::new();
        for nested in body {
            if !seen.insert(nested.id.as_str()) {
                return Err(SkillTimelineCompileError::new(
                    &skill.id,
                    format!("beat {}", nested.id),
                    format!("duplicate nested beat id `{}` in loop body", nested.id),
                ));
            }
            validate_beat_tree(skill, nested)?;
        }
    }

    Ok(())
}

fn compilation_error_from_refs(
    skill: &SkillDef,
    error: &ValidationError,
) -> SkillTimelineCompileError {
    SkillTimelineCompileError::new(
        &skill.id,
        error.site.clone(),
        format!(
            "[{}] missing `{}` at {}",
            error.axis, error.missing_id, error.site
        ),
    )
}
