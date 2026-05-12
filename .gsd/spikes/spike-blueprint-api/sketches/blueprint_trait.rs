// THROWAWAY SKETCH — NON-COMPILED, ILLUSTRATIVE ONLY.
// Spike SP2 — Option B trait definition.
// Real implementation lands in M017 Slice A under src/combat/blueprint_registry.rs.
//
// This file exists to prove the trait shape is expressive enough to cover:
//   1. The 6 existing blueprints (commit_signals override + snapshot override).
//   2. The 2 new round-3 passives (on_event override only — listener-only).
//   3. ValidationSnapshot integration (snapshot method drives observability).

use std::collections::HashMap;

// --- Identity ---

/// Stable, string-keyed blueprint identifier. Matches `SkillCustomSignal.owner`
/// for commit-time routing, and the blueprint id() field for listener routing.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlueprintId(pub &'static str);

// --- Snapshot enum (extends per blueprint addition) ---

/// Observability payload returned by Blueprint::snapshot.
/// Listener-only blueprints (kitsune_grace, holy_aegis) return Empty.
pub enum BlueprintSnapshot {
    Empty,
    TwinCore(super::TwinCoreSnapshot),            // crate::combat::twin_core::TwinCoreSnapshot
    BatteryLoop(super::BatteryLoopSnapshot),       // crate::combat::observability::BatteryLoopSnapshot
    PredatorLoop(super::PredatorLoopSnapshot),     // crate::combat::predator_loop::PredatorLoopSnapshot
    HolySupport(super::HolySupportSnapshot),       // crate::combat::holy_support::HolySupportSnapshot
    PrecisionMindGame(super::PrecisionMindGameSnapshot),
    // Future additions live here. One line per new stateful blueprint.
}

// --- Ctx passed to event listeners ---

/// Read-only handle that listeners use to consult world state without
/// taking a `&mut World`. The kernel constructs this once per event drain
/// and passes it to every Blueprint::on_event call.
///
/// Why a ctx struct instead of `&World`: we want to keep blueprint
/// listeners stateless and trivially-testable. Ctx exposes a closed
/// surface (lookups by UnitId, team membership, status registry).
/// Adding a new lookup requires a deliberate edit here.
pub struct BlueprintCtx<'a> {
    pub world: &'a bevy::prelude::World,
    pub turn_order: &'a super::TurnOrder,
    pub teams: &'a HashMap<super::UnitId, super::Team>,
}

impl<'a> BlueprintCtx<'a> {
    pub fn team_of(&self, unit: super::UnitId) -> Option<super::Team> { unimplemented!() }
    pub fn find_unit_id_for_blueprint(&self, bp: BlueprintId) -> Option<super::UnitId> { unimplemented!() }
    pub fn is_alive(&self, unit: super::UnitId) -> bool { unimplemented!() }
}

// --- The trait ---

/// The single seam between the combat kernel and per-blueprint behaviour.
///
/// Three responsibilities, three methods, each with a sensible default so a
/// blueprint only overrides what it actually needs.
///
///   * `commit_signals`: existing 6 blueprints. Translates a `SkillCustomSignal`
///     (commit-time RON-declared signal) into typed `CombatKernelTransition`s
///     that the existing `apply_*_transitions_system`s consume. Default: empty.
///
///   * `on_event`: new 2 blueprints (kitsune_grace, holy_aegis), and future
///     reactive passives. Reads a `CombatEvent` and emits zero-or-more `Effect`
///     cascades. The listener filter (caster identity, team check, etc.) lives
///     here in Rust — exactly the SP3 hybrid partition. Default: empty.
///
///   * `snapshot`: observability. Reads the blueprint's backing state resource
///     from `World` and returns a `BlueprintSnapshot`. Listener-only blueprints
///     return `Empty`. Default: empty.
///
/// Contract:
///   * All three methods must be pure functions of their inputs (no
///     world mutation, no RNG without seed). This is enforced by structure
///     (`&self`, `&CombatEvent`, `&BlueprintCtx`, return-by-value) and
///     required by SP1 cascade-suspend safety.
pub trait Blueprint: Send + Sync + 'static {
    fn id(&self) -> BlueprintId;

    fn commit_signals(
        &self,
        _signal: &super::SkillCustomSignal,
        _action: &super::ResolvedAction,
    ) -> Result<Vec<super::CombatKernelTransition>, super::CustomSignalDispatchError> {
        Ok(Vec::new())
    }

    fn on_event(
        &self,
        _event: &super::CombatEvent,
        _ctx: &BlueprintCtx,
    ) -> Vec<super::Effect> {
        Vec::new()
    }

    fn snapshot(&self, _world: &bevy::prelude::World) -> BlueprintSnapshot {
        BlueprintSnapshot::Empty
    }
}
