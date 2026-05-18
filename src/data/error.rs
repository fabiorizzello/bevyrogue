//! Typed domain errors for the data layer.
//!
//! Asset bytes loaded at runtime reach the engine through `bevy_common_assets`'
//! `RonAssetPlugin`, so RON syntax errors *on that path* are surfaced by Bevy's
//! asset loader. This layer owns two other failure classes:
//!
//! - *Compile-time aggregate parsing*: the `aggregate_*` helpers parse RON
//!   fragments embedded via `include_str!` for tests and the CLI. A syntax
//!   failure there never reaches Bevy's loader, so it is owned here as
//!   [`DataError::RonParse`], carrying the asset path plus `ron`'s spanned
//!   line/column context.
//! - *Semantic failure of already-parsed data*: a skill book that violates the
//!   validation contract, or a timeline that cannot be compiled into the kernel
//!   representation. Both carry the offending `skill_id` plus the exact site.
//!
//! Every variant is diagnosable from log text alone.

use thiserror::Error;

use super::skill_timeline::SkillTimelineCompileError;
use super::skills_ron::SkillBookValidationError;

/// Failure assembling the runtime skill timeline library from loaded RON assets.
#[derive(Debug, Error)]
pub enum DataError {
    /// A compile-time embedded RON asset fragment failed to parse.
    ///
    /// `ron::error::SpannedError` already carries the line/column of the
    /// failure; the `path` adds *which* embedded asset so the offending file is
    /// identifiable without a debugger.
    #[error("failed to parse RON asset `{path}`: {source}")]
    RonParse {
        /// Repo-relative path of the embedded asset fragment.
        path: &'static str,
        /// The spanned `ron` parse failure (preserves line/column).
        #[source]
        source: ron::error::SpannedError,
    },

    /// A loaded skill book failed the semantic validation contract.
    #[error("skill book validation failed: {0}")]
    Validation(#[from] SkillBookValidationError),

    /// A validated skill book contained a timeline that could not be compiled.
    #[error("skill timeline compilation failed: {0}")]
    TimelineCompile(#[from] SkillTimelineCompileError),
}
