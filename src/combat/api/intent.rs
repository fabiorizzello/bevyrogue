use std::num::NonZeroU32;

use crate::combat::{
    status_effect::StatusEffectKind,
    types::{DamageTag, SkillId, UnitId},
};

/// Unique identifier for a single cast pipeline invocation.
///
/// `ROOT` is reserved for root-level actions that are not part of a cast chain.
/// All other values are allocated by `pipeline::step_app` (S02+).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CastId(pub NonZeroU32);

impl CastId {
    // SAFETY: 1 is non-zero.
    pub const ROOT: Self = Self(unsafe { NonZeroU32::new_unchecked(1) });
}

/// Closed enum of every mutation the kernel may apply to combat state.
///
/// Skills produce `Intent` values via `SkillCtx::enqueue`; the `intent_applier`
/// system drains the queue and routes each variant to the appropriate subsystem.
/// Nothing outside `intent_applier` may mutate combat state directly.
///
/// # Rule (P001)
/// No Digimon-specific names or logic appear here. Blueprint-specific mutations
/// travel via `BlueprintSignal` and `SetBlueprintState`.
#[derive(Debug, Clone)]
pub enum Intent {
    DealDamage {
        source: UnitId,
        target: UnitId,
        amount: i32,
        tag: DamageTag,
        cast_id: CastId,
    },
    HealHp {
        target: UnitId,
        amount: i32,
        cast_id: CastId,
    },
    ApplyStatus {
        source: UnitId,
        target: UnitId,
        kind: StatusEffectKind,
        duration_turns: u32,
        cast_id: CastId,
    },
    RemoveStatus {
        target: UnitId,
        kind: StatusEffectKind,
        cast_id: CastId,
    },
    ApplyBuff {
        target: UnitId,
        kind: StatusEffectKind,
        duration_turns: u32,
        cast_id: CastId,
    },
    RemoveBuff {
        target: UnitId,
        kind: StatusEffectKind,
        cast_id: CastId,
    },
    AdvanceTurn {
        target: UnitId,
        amount_pct: u32,
        cast_id: CastId,
    },
    DelayTurn {
        target: UnitId,
        amount_pct: u32,
        cast_id: CastId,
    },
    EnqueueFollowUp {
        source: UnitId,
        skill_id: SkillId,
        cast_id: CastId,
    },
    BreakToughness {
        source: UnitId,
        target: UnitId,
        amount: i32,
        tag: DamageTag,
        cast_id: CastId,
    },
    ChargeUltimate {
        target: UnitId,
        amount: i32,
        cast_id: CastId,
    },
    ModifySp {
        delta: i32,
        cast_id: CastId,
    },
    AddEnergy {
        target: UnitId,
        amount: i32,
        cast_id: CastId,
    },
    RemoveEnergy {
        target: UnitId,
        amount: i32,
        cast_id: CastId,
    },
    KoUnit {
        target: UnitId,
        cast_id: CastId,
    },
    /// Cross-blueprint signal dispatched to the owning unit's blueprint handler.
    ///
    /// Payload is a `u64` tag for S01; S04 replaces this with the closed-enum
    /// `Signal` type (D028: signal taxonomy registered at `App::finish()`).
    BlueprintSignal {
        owner: UnitId,
        payload: u64,
        cast_id: CastId,
    },
    /// Write a per-unit/per-key state entry (D034 canonical blueprint write-path).
    ///
    /// Key format: `"<blueprint>/<field>"`, e.g. `"kitsune_grace/stacks"`.
    SetBlueprintState {
        actor: UnitId,
        key: String,
        value: i64,
        cast_id: CastId,
    },
    /// Explicitly discard a pending action (e.g. predicate-gate failure).
    Reject {
        cast_id: CastId,
        reason: String,
    },
}
