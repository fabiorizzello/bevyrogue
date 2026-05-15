use bevy::prelude::Message;

use crate::combat::{
    StatusEffectKind,
    api::intent::CastId,
    toughness::DamageKind,
    types::{DamageTag, SkillId, UnitId},
};

// Re-export kernel types used in CombatEventKind variants so callers can import from events.
pub use crate::combat::kernel::{
    BatteryLoopTransition, CombatBeatId, CombatKernelTransition, PredatorLoopTransition,
};

/// Coarse intent classification carried by `OnActionDeclared`.
/// Distinct from `ActionIntent` (which carries full payload) — this is
/// serializable metadata only, with no heap-allocated fields.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ActionIntentKind {
    Basic,
    Skill,
    Ultimate,
}

/// Machine-readable combat bus consumed by follow-up listeners.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CombatEventKind {
    /// Emitted only for Skill/Ultimate intents, never for Basic.
    OnSkillCast {
        skill_id: SkillId,
    },
    OnDamageDealt {
        amount: i32,
        kind: DamageKind,
        tag_mod_pct: i32,
        triangle_mod_pct: i32,
        damage_tag: DamageTag,
    },
    OnBreak {
        damage_tag: DamageTag,
    },
    UnitDied {
        status_remaining: Vec<StatusEffectKind>,
        heated_remaining: u32,
    },
    OnRevive {
        hp_after: i32,
    },
    OnActionFailed {
        reason: String,
    },
    /// Emitted after a status effect ticks on the unit's own turn.
    OnStatusTick {
        kind: StatusEffectKind,
        turns_left: u32,
    },
    /// Emitted when a status effect expires and its component is removed.
    OnStatusExpired {
        kind: StatusEffectKind,
    },
    /// Emitted only on the threshold crossing from >30% HP to <=30% HP.
    /// `source` and `target` both point at the endangered unit so downstream
    /// listeners can resolve ally/enemy context via `Team`.
    OnAllyLowHp,
    /// Emitted when the attacker KOs a unit on the opposing team.
    OnEnemyKill,
    /// Emitted once after a successful bootstrap, documenting the resolved party composition.
    PartySelected {
        ally_ids: Vec<UnitId>,
        tamer_id: UnitId,
    },
    /// Emitted once after turn order is seeded, listing all unit IDs in queue order.
    TurnOrderSeeded {
        unit_ids: Vec<UnitId>,
    },
    /// Emitted when a unit's ultimate charge actually increases; amount is the delta added.
    UltGain {
        unit_id: UnitId,
        amount: i32,
    },
    /// Emitted once per cast when an ultimate is spent (UltEffect::Reset). Distinct from
    /// UltGain — enables downstream listeners to observe spend without reconstructing state.
    UltimateUsed {
        unit_id: UnitId,
    },
    /// Emitted for the defender on every non-revive hit (companion of OnDamageDealt).
    OnHitTaken {
        amount: i32,
    },
    /// Emitted once per successful status-effect application (after apply_effects succeeds
    /// and the defender survived). Re-applying the same kind replaces the component (Bevy
    /// insert-overwrites by design).
    OnStatusApplied {
        kind: StatusEffectKind,
    },
    /// Emitted when a status application is blocked by the accuracy roll (triangle penalty).
    /// Source = attacker, target = intended defender. No StatusEffect component is inserted.
    OnStatusResisted {
        kind: StatusEffectKind,
    },
    /// Pull a unit's turn forward by amount_pct% of MAX_AV.
    AdvanceTurn {
        target: UnitId,
        amount_pct: u32,
    },
    /// Push a unit's turn back by amount_pct% of MAX_AV (TempoResistance applies).
    DelayTurn {
        target: UnitId,
        amount_pct: u32,
    },
    /// Emitted at the start of action processing, before any effects are applied.
    OnActionDeclared {
        intent_kind: ActionIntentKind,
    },
    /// Emitted after declaration but before `apply_deferred` / `ApplyDeferred` flush.
    OnActionPreApp,
    /// Canonical beat marker corresponding to a lifecycle seam.
    OnCombatBeat {
        beat: CombatBeatId,
    },
    /// Typed kernel transition for Tactical Cycle / Strain / Flow / Fatigue / tags.
    OnKernelTransition {
        transition: CombatKernelTransition,
    },
    /// Typed BatteryLoop outcome after the raw kernel transition has been resolved against
    /// the current combat state and energy targets.
    BatteryLoopResolved {
        transition: BatteryLoopTransition,
    },
    /// Typed PredatorLoop outcome after the raw kernel transition has been resolved against
    /// the current combat state and prey-lock targets.
    PredatorLoopResolved {
        transition: PredatorLoopTransition,
    },
    /// Emitted immediately after effects are applied (damage/status committed to world).
    OnActionApplied,
    /// Emitted at the end of the full action pipeline, after all reactive events settle.
    OnActionResolved,
    /// Emitted whenever Form Identity grants energy to a unit.
    EnergyGained {
        unit_id: UnitId,
        amount: i32,
    },
    /// Emitted once per successful Heal application; amount is the actual HP restored
    /// (capped at hp_max). Silently suppressed on KO targets (no event emitted).
    OnHealed {
        amount: i32,
        hp_after: i32,
    },
    /// Emitted once per Cleanse application per target (including no-op cleanses where
    /// kinds is empty, mirroring OnHealed amount=0).
    OnCleansed {
        kinds: Vec<StatusEffectKind>,
    },
}

#[derive(Message, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CombatEvent {
    pub kind: CombatEventKind,
    pub source: UnitId,
    pub target: UnitId,
    /// 0 for root actions, incremented by 1 for each follow-up hop.
    /// D046: chain bounding lives in the data (stack/cooldown/once-per-round flags),
    /// not in the engine. The listener does not suppress based on this value.
    pub follow_up_depth: u8,
    /// Unique identifier for the cast pipeline invocation that produced this event.
    /// `CastId::ROOT` (value 1) for lifecycle events outside a cast (status ticks,
    /// victory checks, pre-cast declaration/preapp). All events emitted within a
    /// single `step_app` call share the same non-ROOT `cast_id`.
    pub cast_id: CastId,
}
