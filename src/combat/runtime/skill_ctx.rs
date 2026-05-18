use std::collections::{HashSet, VecDeque};

use bevy::prelude::World;

use crate::combat::{
    runtime::{
        intent::{CastId, Intent},
        registry::ExtRegistries,
        timeline::BeatPayload,
    },
    team::Team,
    types::UnitId,
    unit::Unit,
};

/// Governs how the skill pipeline processes enqueued `Intent` values.
///
/// `Execute` is the normal in-game path. `DryRun` and `Preview` are reserved
/// for AI simulation and UI affordance display respectively; full support arrives
/// in S05+. For S01 all paths behave identically.
// DryRun/Preview consumed by tests/timeline_mode_parity.rs.
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
    // `mode` is passed to the context constructor; accessed via mode() accessor in runner_common.
    pub mode: SkillCtxMode,
    /// All extension registries for this cast (hooks, selectors, predicates, cues…).
    // `registries` is part of the public context API surface for extension hooks.
    pub registries: &'a ExtRegistries,
    /// Read-only borrow of the Bevy world — replaces the spike's thread-local F7 pattern.
    pub world: &'a World,
    /// Tracks units already hit this cast (NoRepeat selector / chain-bolt pattern).
    pub cast_hit_set: &'a mut HashSet<UnitId>,
    /// Typed payload for the currently executing beat, if the timeline supplied one.
    beat_payload: Option<&'a BeatPayload>,
    pending: &'a mut VecDeque<Intent>,
}

impl<'a> SkillCtx<'a> {
    pub fn new(
        caster: UnitId,
        primary_target: UnitId,
        cast_id: CastId,
        mode: SkillCtxMode,
        registries: &'a ExtRegistries,
        world: &'a World,
        cast_hit_set: &'a mut HashSet<UnitId>,
        pending: &'a mut VecDeque<Intent>,
        beat_payload: Option<&'a BeatPayload>,
    ) -> Self {
        Self {
            caster,
            primary_target,
            cast_id,
            mode,
            registries,
            world,
            cast_hit_set,
            beat_payload,
            pending,
        }
    }

    /// Enqueue an `Intent` for deferred execution by `intent_applier`.
    ///
    /// Call order within a single hook is preserved (FIFO).
    pub fn enqueue(&mut self, intent: Intent) {
        self.pending.push_back(intent);
    }

    // Accessor used in runner_common.rs and potentially by blueprint hook fns.
    pub fn cast_id(&self) -> CastId {
        self.cast_id
    }

    // Accessor for mode; used in runner_common.rs.
    pub fn mode(&self) -> SkillCtxMode {
        self.mode
    }

    pub fn beat_payload(&self) -> Option<&'a BeatPayload> {
        self.beat_payload
    }

    /// Return the caster's team using an immutable world query.
    // Available for blueprint hook fns to use in filtering logic.
    pub fn caster_team(&self) -> Option<Team> {
        let mut q = self.world.try_query::<(&Unit, &Team)>()?;
        q.iter(self.world)
            .find_map(|(unit, team)| (unit.id == self.caster).then_some(*team))
    }
}
