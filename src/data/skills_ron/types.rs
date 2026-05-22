use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::combat::status_effect::StatusEffectKind;
use crate::combat::types::{DamageTag, SkillId};
use crate::data::skill_timeline::SkillTimeline;

/// How the next bounce hop target is selected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BounceSelector {
    /// Select the alive enemy with the lowest HP percentage.
    LowestHpPctAlive,
    /// Select the next alive enemy in slot order (wrapping).
    NextSlotAlive,
    /// Select the alive enemy in the adjacent slot with the lowest HP.
    AdjLowest,
}

/// Whether the bounce chain is allowed to revisit already-hit targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepeatPolicy {
    /// Each target can only be hit once per cast.
    NoRepeat,
    /// Targets may be re-selected on subsequent hops.
    AllowRepeat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetShape {
    Single,
    /// Primary target + adjacent slot_index ±1 on the same team, alive, slot_index asc.
    Blast,
    Row,
    AllEnemies,
    SelfOnly,
    /// All alive units on the caster's own team (ally side), slot_index ascending.
    AllAllies,
    /// Chaining bounce: hits up to `hops` targets in sequence, re-resolving the selector
    /// each hop. Chain stops early if no valid target is found.
    Bounce {
        hops: u8,
        selector: BounceSelector,
        repeat: RepeatPolicy,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetSide {
    Ally,
    Enemy,
    Any,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetLife {
    Alive,
    Ko,
    Any,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelfTargetRule {
    Forbid,
    Allow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TargetHpRule {
    #[default]
    Any,
    Damaged,
}

// S03 declares side/life/self targeting metadata here; later slices make it queryable and enforce it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillTargeting {
    pub shape: TargetShape,
    pub side: TargetSide,
    pub life: TargetLife,
    pub self_rule: SelfTargetRule,
    #[serde(default)]
    pub target_hp_rule: TargetHpRule,
}

impl Default for SkillTargeting {
    fn default() -> Self {
        Self {
            shape: TargetShape::Single,
            side: TargetSide::Enemy,
            life: TargetLife::Alive,
            self_rule: SelfTargetRule::Forbid,
            target_hp_rule: TargetHpRule::Any,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LegalityReasonCode {
    UnimplementedTargetShape,
    UnimplementedEffect,
    WrongSide,
    TargetKo,
    TargetNotKo,
    TargetFullHp,
    TargetNotDamaged,
    TargetIsSelf,
    TargetIsCommander,
    NoValidTargets,
    ToughnessEnemyOnly,
    NotActiveUnit,
    WrongPhase,
    AttackerKo,
    AttackerStunned,
    MissingSkill,
    SpShortfall,
    UltimateNotReady,
    TargetNotFound,
    TamerGaugeDeferred,
    TamerCommandDeferred,
    ChargedTelegraphDeferred,
    EnemyTraitDeferred,
    /// A skill carries two effect kinds that are mutually exclusive in v0 (e.g. Heal + Cleanse).
    MixedEffectKinds,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SkillImplementation {
    #[default]
    Implemented,
    Deferred {
        reason: LegalityReasonCode,
    },
    Hidden {
        reason: LegalityReasonCode,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum CustomSignalPayload {
    Empty,
    Amount { amount: i32 },
}

impl Default for CustomSignalPayload {
    fn default() -> Self {
        Self::Empty
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillCustomSignal {
    pub owner: String,
    pub signal: String,
    #[serde(default)]
    pub payload: CustomSignalPayload,
}

impl SkillCustomSignal {
    pub fn blueprint(
        owner: impl Into<String>,
        signal: impl Into<String>,
        payload: CustomSignalPayload,
    ) -> Self {
        Self {
            owner: owner.into(),
            signal: signal.into(),
            payload,
        }
    }

    pub fn owner(&self) -> &str {
        self.owner.as_str()
    }

    pub fn signal(&self) -> &str {
        self.signal.as_str()
    }

    pub fn payload(&self) -> CustomSignalPayload {
        self.payload
    }
}

/// Per-hop damage scaling for Bounce chains.
///
/// - `Constant`: every hop deals `base_damage` (default).
/// - `Falloff { pct }`: each subsequent hop deals `pct/100` of `base_damage` less than the previous
///   (i.e. hop N deals `base_damage * (pct/100)^N`). `pct` must be <= 100.
/// - `PerHop(Vec<i32>)`: explicit override per hop; vec length must equal `hops`.
///   Overrides `base_damage` for each index; `base_damage` is ignored.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DamageCurve {
    #[default]
    Constant,
    Falloff {
        /// Percentage retained per hop (1–100). E.g. 80 means each hop deals 80% of the previous.
        pct: u16,
    },
    PerHop(Vec<i32>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum LegacyEffect {
    Damage {
        amount: i32,
        target: TargetShape,
        #[serde(default)]
        per_hop: DamageCurve,
    },
    ToughnessHit(i32),
    GainSP(i32),
    UltGain(i32),
    Stun,
    Revive(i32),
    GrantFreeSkill {
        count: usize,
    },
    ApplyStatus {
        kind: StatusEffectKind,
        duration: u32,
    },
    AdvanceTurn(u32),
    DelayTurn(u32),
    /// Grant the attacker N energy (once-per-round gated by RoundFlags.form_identity_used).
    GrantEnergy(i32),
    /// Advance the attacker's own AV by N percent (self-tempo boost).
    SelfAdvance(i32),
    /// Restore HP to one or more allies. `amount_pct_max_hp` is a percentage of the target's
    /// hp_max (1–100). `target` must be an ally-side shape (Single, SelfOnly, AllAllies).
    /// Capped at hp_max; no-ops silently on KO targets.
    Heal {
        amount_pct_max_hp: u32,
        target: TargetShape,
    },
    /// Remove up to `count` non-immune debuffs from an ally's StatusBag (None = remove all).
    /// `target` must be an ally-side shape (Single, SelfOnly, AllAllies).
    /// Cannot coexist with Effect::Heal in the same skill (deferred to M021).
    Cleanse {
        count: Option<u8>,
        target: TargetShape,
    },
}

pub use LegacyEffect as Effect;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct SkillDef {
    pub id: SkillId,
    pub name: String,
    pub damage_tag: DamageTag,
    pub sp_cost: i32,
    pub targeting: SkillTargeting,
    pub implementation: SkillImplementation,
    pub legacy_ops: Vec<LegacyEffect>,
    #[serde(default)]
    pub custom_signals: Vec<SkillCustomSignal>,
    /// Optional sequence of animation steps for visual polish.
    pub animation_sequence: Option<Vec<String>>,
    /// Optional QTE mechanic description.
    pub qte: Option<String>,
    /// Optional compiled timeline schema for the kernel timeline path.
    #[serde(default)]
    pub timeline: Option<SkillTimeline>,
}

#[derive(Asset, TypePath, Debug, Clone, Deserialize)]
#[serde(transparent)]
pub struct SkillBook(pub Vec<SkillDef>);
