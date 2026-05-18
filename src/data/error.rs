//! Typed domain errors for the data layer.
//!
//! Asset bytes reach the engine through `bevy_common_assets`' `RonAssetPlugin`,
//! so RON syntax errors are surfaced by Bevy's asset loader, not here. What this
//! layer owns is *semantic* failure of already-parsed skill data: a skill book
//! that violates the validation contract, or a timeline that cannot be compiled
//! into the kernel representation. Both carry the offending `skill_id` plus the
//! exact site, so the failure is diagnosable from log text alone.

use thiserror::Error;

use super::skill_timeline::SkillTimelineCompileError;
use super::skills_ron::SkillBookValidationError;

/// Failure assembling the runtime skill timeline library from loaded RON assets.
#[derive(Debug, Error)]
pub enum DataError {
    /// A loaded skill book failed the semantic validation contract.
    #[error("skill book validation failed: {0}")]
    Validation(#[from] SkillBookValidationError),

    /// A validated skill book contained a timeline that could not be compiled.
    #[error("skill timeline compilation failed: {0}")]
    TimelineCompile(#[from] SkillTimelineCompileError),
}
