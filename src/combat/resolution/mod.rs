mod types;
mod skill_extract;
mod apply;

// ── Re-exports ──────────────────────────────────────────────────────────────
// All public items are re-exported so external consumers keep using
// `crate::combat::resolution::Foo` unchanged.

pub use types::{
    ResolutionOutcome, TargetEntry, TargetableSnapshot,
    resolve_targets, select_bounce_hop,
    target_shape_is_executable_now, target_shape_rejection_reason,
};

pub use skill_extract::{
    compute_hop_damage, resolve_action, skill_damage_curve,
};

pub use apply::{
    apply_cleanse_only, apply_damage_only, apply_heal_only, apply_legacy_ops,
};
