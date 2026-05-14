use std::collections::VecDeque;

use crate::combat::{
    api::intent::{CastId, Intent},
    types::UnitId,
};

/// Governs how the skill pipeline processes enqueued `Intent` values.
///
/// `Execute` is the normal in-game path. `DryRun` and `Preview` are reserved
/// for AI simulation and UI affordance display respectively; full support arrives
/// in S05+. For S01 all paths behave identically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SkillCtxMode {
    /// Normal in-game execution: intents are committed to state.
    #[default]
    Execute,
    /// Simulation path: intents are resolved but state mutations are rolled back.
    DryRun,
    /// UI affordance path: intent effects are previewed without committing.
    Preview,
}

/// Short-lived borrow context passed to skill hook functions.
///
/// Provides read-only identity fields (`caster`, `primary_target`, `cast_id`,
/// `mode`) and a single write-deferred channel (`enqueue`) that buffers `Intent`
/// values for the `intent_applier` system to execute after the hook returns.
///
/// # Lifetime
/// `'a` is tied to the `VecDeque<Intent>` that the hook framework allocates
/// per-cast and passes to each hook in sequence. No allocation happens inside
/// the hook; only the deferred queue grows.
pub struct SkillCtx<'a> {
    pub caster: UnitId,
    pub primary_target: UnitId,
    pub cast_id: CastId,
    pub mode: SkillCtxMode,
    pending: &'a mut VecDeque<Intent>,
}

impl<'a> SkillCtx<'a> {
    pub fn new(
        caster: UnitId,
        primary_target: UnitId,
        cast_id: CastId,
        mode: SkillCtxMode,
        pending: &'a mut VecDeque<Intent>,
    ) -> Self {
        Self {
            caster,
            primary_target,
            cast_id,
            mode,
            pending,
        }
    }

    /// Enqueue an `Intent` for deferred execution by `intent_applier`.
    ///
    /// Call order within a single hook is preserved (FIFO).
    pub fn enqueue(&mut self, intent: Intent) {
        self.pending.push_back(intent);
    }

    pub fn cast_id(&self) -> CastId {
        self.cast_id
    }

    pub fn mode(&self) -> SkillCtxMode {
        self.mode
    }
}
