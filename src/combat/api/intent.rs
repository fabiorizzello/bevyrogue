use std::num::NonZeroU32;

use bevy::prelude::Resource;

use crate::combat::{
    status_effect::StatusEffectKind,
    types::{DamageTag, SkillId, UnitId},
};

/// Unique identifier for a single cast pipeline invocation.
///
/// `ROOT` is reserved for root-level actions that are not part of a cast chain.
/// All other values are allocated by `pipeline::step_app` (S02+).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct CastId(pub NonZeroU32);

impl CastId {
    // SAFETY: 1 is non-zero.
    pub const ROOT: Self = Self(unsafe { NonZeroU32::new_unchecked(1) });
}

/// Monotonic counter that allocates unique `CastId` values for each cast pipeline invocation.
///
/// `ROOT` (value 1) is reserved; the first allocated id is 2.
/// Register with `.init_resource::<CastIdGen>()` in any app that runs the combat pipeline.
#[derive(Debug, Resource)]
pub struct CastIdGen(u32);

impl Default for CastIdGen {
    fn default() -> Self {
        Self(1) // next() adds 1 → first issued id is 2, leaving 1 for ROOT
    }
}

impl CastIdGen {
    /// Allocate the next unique `CastId`. Never returns ROOT.
    pub fn next(&mut self) -> CastId {
        self.0 = self.0.saturating_add(1);
        // If we somehow hit 1 again (impossible with saturating_add from 1), skip it.
        if self.0 == 1 {
            self.0 = 2;
        }
        CastId(NonZeroU32::new(self.0).expect("CastIdGen: counter saturated at 0"))
    }
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
    /// Payload is a `SignalPayload` (S04).
    BlueprintSignal {
        source: UnitId,
        owner: &'static str,
        name: &'static str,
        payload: crate::combat::api::SignalPayload,
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
