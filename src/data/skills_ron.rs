use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

use super::skill_timeline::SkillTimeline;
use crate::combat::status_effect::StatusEffectKind;
use crate::combat::types::{DamageTag, SkillId};

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
    EnergyCapReached,
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
pub enum Effect {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct SkillDef {
    pub id: SkillId,
    pub name: String,
    pub damage_tag: DamageTag,
    pub sp_cost: i32,
    pub targeting: SkillTargeting,
    pub implementation: SkillImplementation,
    pub effects: Vec<Effect>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillBookValidationCategory {
    #[allow(dead_code)] // kept for: structural-error category (vocabulary anchor; only Semantic constructed today)
    Structural,
    Semantic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillBookValidationError {
    pub skill_id: SkillId,
    pub category: SkillBookValidationCategory,
    pub reason: LegalityReasonCode,
    pub detail: String,
}

impl fmt::Display for SkillBookValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "skill_id={} category={:?} reason={:?} detail={}",
            self.skill_id.0, self.category, self.reason, self.detail
        )
    }
}

impl std::error::Error for SkillBookValidationError {}

pub fn validate_skill_book(book: &SkillBook) -> Result<(), SkillBookValidationError> {
    for skill in &book.0 {
        validate_skill_def(skill)?;
    }
    Ok(())
}

fn validation_error(
    skill: &SkillDef,
    category: SkillBookValidationCategory,
    reason: LegalityReasonCode,
    detail: impl Into<String>,
) -> SkillBookValidationError {
    SkillBookValidationError {
        skill_id: skill.id.clone(),
        category,
        reason,
        detail: detail.into(),
    }
}

fn skill_has_effect(skill: &SkillDef, predicate: impl Fn(&Effect) -> bool) -> bool {
    skill.effects.iter().any(predicate)
}

const CANON_STATUS_IDS: &[&str] = &["heated", "chilled", "paralyzed", "slowed", "blessed"];

fn validate_skill_def(skill: &SkillDef) -> Result<(), SkillBookValidationError> {
    use crate::combat::status_effect::StatusEffectKind;

    for effect in &skill.effects {
        if let Effect::ApplyStatus { kind, .. } = effect {
            if matches!(kind, StatusEffectKind::Burn | StatusEffectKind::Shock) {
                return Err(validation_error(
                    skill,
                    SkillBookValidationCategory::Semantic,
                    LegalityReasonCode::UnimplementedEffect,
                    format!(
                        "ApplyStatus uses reserved status kind {:?}; valid ids are: {}",
                        kind,
                        CANON_STATUS_IDS.join(", ")
                    ),
                ));
            }
        }
    }

    let has_damage = skill_has_effect(skill, |effect| matches!(effect, Effect::Damage { .. }));
    let has_revive = skill_has_effect(skill, |effect| matches!(effect, Effect::Revive(_)));

    // Reject Bounce with hops == 0 (always, regardless of implementation status).
    if let TargetShape::Bounce { hops, .. } = skill.targeting.shape {
        if hops == 0 {
            return Err(validation_error(
                skill,
                SkillBookValidationCategory::Semantic,
                LegalityReasonCode::UnimplementedTargetShape,
                "Bounce hops must be >= 1; found hops=0",
            ));
        }
    }

    fn shape_is_executable(shape: TargetShape) -> bool {
        matches!(
            shape,
            TargetShape::Single
                | TargetShape::Blast
                | TargetShape::AllEnemies
                | TargetShape::SelfOnly
                | TargetShape::AllAllies
                | TargetShape::Bounce { .. }
        )
    }

    if matches!(skill.implementation, SkillImplementation::Implemented)
        && !shape_is_executable(skill.targeting.shape)
    {
        return Err(validation_error(
            skill,
            SkillBookValidationCategory::Semantic,
            LegalityReasonCode::UnimplementedTargetShape,
            format!(
                "implemented skills support Single, Blast, AllEnemies, SelfOnly, AllAllies, or Bounce{{hops>=1}}; found {:?}",
                skill.targeting.shape
            ),
        ));
    }

    if has_damage {
        for (target, per_hop) in skill.effects.iter().filter_map(|effect| match effect {
            Effect::Damage { target, per_hop, .. } => Some((*target, per_hop)),
            _ => None,
        }) {
            if target != skill.targeting.shape {
                return Err(validation_error(
                    skill,
                    SkillBookValidationCategory::Semantic,
                    LegalityReasonCode::UnimplementedTargetShape,
                    format!(
                        "damage effect target {:?} contradicts targeting.shape {:?}",
                        target, skill.targeting.shape
                    ),
                ));
            }

            // Validate DamageCurve constraints for Bounce shapes.
            if let TargetShape::Bounce { hops, .. } = skill.targeting.shape {
                match per_hop {
                    DamageCurve::Constant => {}
                    DamageCurve::Falloff { pct } => {
                        if *pct > 100 {
                            return Err(validation_error(
                                skill,
                                SkillBookValidationCategory::Semantic,
                                LegalityReasonCode::UnimplementedEffect,
                                format!(
                                    "DamageCurve::Falloff pct must be <= 100; found pct={}",
                                    pct
                                ),
                            ));
                        }
                    }
                    DamageCurve::PerHop(v) => {
                        if v.len() != hops as usize {
                            return Err(validation_error(
                                skill,
                                SkillBookValidationCategory::Semantic,
                                LegalityReasonCode::UnimplementedEffect,
                                format!(
                                    "DamageCurve::PerHop length {} must equal hops {}",
                                    v.len(),
                                    hops
                                ),
                            ));
                        }
                    }
                }
            }
        }
    }

    if has_revive && !has_damage {
        if skill.targeting.side != TargetSide::Ally {
            return Err(validation_error(
                skill,
                SkillBookValidationCategory::Semantic,
                LegalityReasonCode::WrongSide,
                format!(
                    "revive skills must target allies, found {:?}",
                    skill.targeting.side
                ),
            ));
        }

        if skill.targeting.life != TargetLife::Ko {
            return Err(validation_error(
                skill,
                SkillBookValidationCategory::Semantic,
                LegalityReasonCode::TargetNotKo,
                format!(
                    "revive skills must target KO units, found {:?}",
                    skill.targeting.life
                ),
            ));
        }

        if skill.targeting.shape != TargetShape::Single {
            return Err(validation_error(
                skill,
                SkillBookValidationCategory::Semantic,
                LegalityReasonCode::UnimplementedTargetShape,
                format!(
                    "revive skills currently support only TargetShape::Single, found {:?}",
                    skill.targeting.shape
                ),
            ));
        }
    }

    if has_damage && has_revive {
        match &skill.implementation {
            SkillImplementation::Implemented => {
                return Err(validation_error(
                    skill,
                    SkillBookValidationCategory::Semantic,
                    LegalityReasonCode::UnimplementedEffect,
                    "mixed damage+revive semantics are unresolved",
                ));
            }
            SkillImplementation::Deferred { reason } | SkillImplementation::Hidden { reason } => {
                if *reason != LegalityReasonCode::UnimplementedEffect {
                    return Err(validation_error(
                        skill,
                        SkillBookValidationCategory::Semantic,
                        LegalityReasonCode::UnimplementedEffect,
                        format!(
                            "mixed damage+revive semantics require Deferred/Hidden reason UnimplementedEffect, found {:?}",
                            reason
                        ),
                    ));
                }
            }
        }
    }

    if let SkillImplementation::Deferred { reason } | SkillImplementation::Hidden { reason } =
        &skill.implementation
    {
        if *reason == LegalityReasonCode::UnimplementedTargetShape
            && skill.targeting.shape == TargetShape::Single
            && !has_revive
            && !has_damage
        {
            return Err(validation_error(
                skill,
                SkillBookValidationCategory::Semantic,
                LegalityReasonCode::UnimplementedTargetShape,
                "target-shape deferrals must not claim single-target execution",
            ));
        }
    }

    for effect in &skill.effects {
        if let Effect::Heal { target, .. } = effect {
            match target {
                TargetShape::Bounce { .. }
                | TargetShape::AllEnemies
                | TargetShape::Blast => {
                    return Err(validation_error(
                        skill,
                        SkillBookValidationCategory::Semantic,
                        LegalityReasonCode::WrongSide,
                        format!(
                            "Heal effect may not target enemy-side shapes; found {:?}",
                            target
                        ),
                    ));
                }
                _ => {}
            }
        }
    }

    for effect in &skill.effects {
        if let Effect::Cleanse { target, .. } = effect {
            match target {
                TargetShape::Bounce { .. }
                | TargetShape::AllEnemies
                | TargetShape::Blast => {
                    return Err(validation_error(
                        skill,
                        SkillBookValidationCategory::Semantic,
                        LegalityReasonCode::WrongSide,
                        format!(
                            "Cleanse effect may not target enemy-side shapes; found {:?}",
                            target
                        ),
                    ));
                }
                _ => {}
            }
        }
    }

    let has_heal = skill.effects.iter().any(|e| matches!(e, Effect::Heal { .. }));
    let has_cleanse = skill.effects.iter().any(|e| matches!(e, Effect::Cleanse { .. }));
    if has_heal && has_cleanse {
        return Err(validation_error(
            skill,
            SkillBookValidationCategory::Semantic,
            LegalityReasonCode::MixedEffectKinds,
            "Heal and Cleanse may not coexist in the same skill (deferred to M021)".to_string(),
        ));
    }

    Ok(())
}

#[derive(Asset, TypePath, Debug, Clone, Deserialize)]
#[serde(transparent)]
pub struct SkillBook(pub Vec<SkillDef>);

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn offensive_targeting(shape: TargetShape) -> SkillTargeting {
        SkillTargeting {
            shape,
            side: TargetSide::Enemy,
            life: TargetLife::Alive,
            self_rule: SelfTargetRule::Forbid,
            ..Default::default()
        }
    }

    fn revive_targeting() -> SkillTargeting {
        SkillTargeting {
            shape: TargetShape::Single,
            side: TargetSide::Ally,
            life: TargetLife::Ko,
            self_rule: SelfTargetRule::Forbid,
            ..Default::default()
        }
    }

    fn sample_skill() -> SkillDef {
        SkillDef {
            id: SkillId("baby_flame".into()),
            name: "Baby Flame".into(),
            damage_tag: DamageTag::Fire,
            sp_cost: 4,
            targeting: offensive_targeting(TargetShape::Single),
            implementation: SkillImplementation::Implemented,
            effects: vec![
                Effect::Damage {
                    amount: 18,
                    target: TargetShape::Single,
                    per_hop: DamageCurve::Constant,
                },
                Effect::ToughnessHit(10),
            ],
            ..Default::default()
        }
}

    fn canonical_skill_book() -> SkillBook {
        ron::from_str(include_str!("../../assets/data/skills.ron")).expect("parse skills.ron")
    }

    #[test]
    fn round_trip_skill_def() {
        let def = sample_skill();
        let s = ron::to_string(&def).expect("serialize");
        let back: SkillDef = ron::from_str(&s).expect("deserialize");
        assert_eq!(def, back);
    }

    #[test]
    fn effect_roundtrip_damage_struct_variant() {
        let effect = Effect::Damage {
            amount: 18,
            target: TargetShape::Single,
            per_hop: DamageCurve::Constant,
        };
        let s = ron::to_string(&effect).expect("serialize");
        // per_hop is always serialized (serde default only skips on deserialize side)
        assert!(
            s.contains("amount:18") && s.contains("target:Single"),
            "unexpected serialized form: {s}"
        );
        let back: Effect = ron::from_str("Damage(amount: 18, target: Single)").expect("parse with default per_hop");
        assert_eq!(back, effect);
    }

    #[test]
    fn effect_roundtrip_toughness_tuple() {
        let effect = Effect::ToughnessHit(10);
        let s = ron::to_string(&effect).expect("serialize");
        assert_eq!(s, "ToughnessHit(10)");
        let back: Effect = ron::from_str("ToughnessHit(10)").expect("parse");
        assert_eq!(back, effect);
    }

    #[test]
    fn effect_roundtrip_stun_unit() {
        let s = ron::to_string(&Effect::Stun).expect("serialize");
        assert_eq!(s, "Stun");
        let back: Effect = ron::from_str("Stun").expect("parse");
        assert_eq!(back, Effect::Stun);
    }

    #[test]
    fn effect_roundtrip_revive() {
        let effect = Effect::Revive(25);
        let s = ron::to_string(&effect).expect("serialize");
        assert_eq!(s, "Revive(25)");
        let back: Effect = ron::from_str("Revive(25)").expect("parse");
        assert_eq!(back, effect);
    }

    #[test]
    fn effect_roundtrip_apply_status_heated() {
        let effect = Effect::ApplyStatus {
            kind: StatusEffectKind::Heated,
            duration: 3,
        };
        let s = ron::to_string(&effect).expect("serialize");
        let back: Effect = ron::from_str(&s).expect("deserialize");
        assert_eq!(effect, back);
    }

    #[test]
    fn effect_roundtrip_apply_status_chilled() {
        let effect = Effect::ApplyStatus {
            kind: StatusEffectKind::Chilled,
            duration: 2,
        };
        let s = ron::to_string(&effect).expect("serialize");
        let back: Effect = ron::from_str(&s).expect("deserialize");
        assert_eq!(effect, back);
    }

    #[test]
    fn effect_roundtrip_apply_status_paralyzed() {
        let effect = Effect::ApplyStatus {
            kind: StatusEffectKind::Paralyzed,
            duration: 1,
        };
        let s = ron::to_string(&effect).expect("serialize");
        let back: Effect = ron::from_str(&s).expect("deserialize");
        assert_eq!(effect, back);
    }

    #[test]
    fn effect_roundtrip_advance_turn() {
        let effect = Effect::AdvanceTurn(25);
        let s = ron::to_string(&effect).expect("serialize");
        assert_eq!(s, "AdvanceTurn(25)");
        let back: Effect = ron::from_str("AdvanceTurn(25)").expect("parse");
        assert_eq!(back, effect);
    }

    #[test]
    fn effect_roundtrip_delay_turn() {
        let effect = Effect::DelayTurn(30);
        let s = ron::to_string(&effect).expect("serialize");
        assert_eq!(s, "DelayTurn(30)");
        let back: Effect = ron::from_str("DelayTurn(30)").expect("parse");
        assert_eq!(back, effect);
    }

    // duration is u32 so negative durations are structurally impossible at parse time.
    #[test]
    fn apply_status_negative_duration_rejected_at_parse_time() {
        let err = ron::from_str::<Effect>("ApplyStatus(kind:Heated,duration:-1)")
            .expect_err("negative u32 must fail");
        let msg = err.to_string();
        assert!(
            msg.contains("Expected integer")
                || msg.contains("integer overflow")
                || msg.contains("trailing characters")
                || msg.contains("Invalid value")
                || msg.contains("expected u32")
                || msg.contains("Err"),
            "unexpected parse error: {msg}"
        );
    }

    #[test]
    fn effect_parse_error_bad_type() {
        let err = ron::from_str::<Effect>("Damage(amount: \"not_int\", target: Single)")
            .expect_err("invalid int should fail");
        assert!(
            err.to_string().contains("Expected integer")
                || err.to_string().contains("Expected integer type")
                || err.to_string().contains("Invalid value"),
            "unexpected parse error: {err}"
        );
    }

    #[test]
    fn effect_roundtrip_grant_energy() {
        let effect = Effect::GrantEnergy(5);
        let s = ron::to_string(&effect).expect("serialize");
        assert_eq!(s, "GrantEnergy(5)");
        let back: Effect = ron::from_str("GrantEnergy(5)").expect("parse");
        assert_eq!(back, effect);
    }

    #[test]
    fn effect_roundtrip_self_advance() {
        let effect = Effect::SelfAdvance(20);
        let s = ron::to_string(&effect).expect("serialize");
        assert_eq!(s, "SelfAdvance(20)");
        let back: Effect = ron::from_str("SelfAdvance(20)").expect("parse");
        assert_eq!(back, effect);
    }

    #[test]
    fn targeting_roundtrip_and_reason_codes() {
        let targeting = revive_targeting();
        let s = ron::to_string(&targeting).expect("serialize");
        let back: SkillTargeting = ron::from_str(&s).expect("deserialize");
        assert_eq!(targeting, back);

        let implemented = ron::to_string(&SkillImplementation::Implemented).expect("serialize");
        let back_impl: SkillImplementation = ron::from_str(&implemented).expect("deserialize");
        assert_eq!(back_impl, SkillImplementation::Implemented);

        let deferred = SkillImplementation::Deferred {
            reason: LegalityReasonCode::UnimplementedTargetShape,
        };
        let s = ron::to_string(&deferred).expect("serialize");
        let back_deferred: SkillImplementation = ron::from_str(&s).expect("deserialize");
        assert_eq!(deferred, back_deferred);
    }

    #[test]
    fn missing_targeting_metadata_fails_parse() {
        let err = ron::from_str::<SkillDef>(
            r#"(
                id: SkillId("bad_skill"),
                name: "Bad Skill",
                damage_tag: Fire,
                sp_cost: 0,
                implementation: Implemented,
                effects: [Damage(amount: 1, target: Single)]
            )"#,
        )
        .expect_err("missing targeting must fail parse");

        let msg = err.to_string();
        assert!(
            msg.contains("missing") && msg.contains("targeting"),
            "unexpected parse error: {msg}"
        );
    }

    #[test]
    fn unknown_targeting_field_fails_parse() {
        let err = ron::from_str::<SkillDef>(
            r#"(
                id: SkillId("bad_skill"),
                name: "Bad Skill",
                damage_tag: Fire,
                sp_cost: 0,
                targeting: (shape: Single, side: Enemy, life: Alive, self_rule: Forbid),
                implementation: Implemented,
                bogus_field: true,
                effects: [Damage(amount: 1, target: Single)]
            )"#,
        )
        .expect_err("unknown field must fail parse");

        let msg = err.to_string();
        assert!(
            (msg.contains("Unexpected field named") || msg.contains("unknown"))
                && msg.contains("bogus_field"),
            "unexpected parse error: {msg}"
        );
    }

    #[test]
    fn validate_rejects_row_damage_against_single_targeting() {
        let book = SkillBook(vec![SkillDef {
            id: SkillId("row_mismatch".into()),
            name: "Row Mismatch".into(),
            damage_tag: DamageTag::Fire,
            sp_cost: 0,
            targeting: SkillTargeting {
                shape: TargetShape::Single,
                side: TargetSide::Enemy,
                life: TargetLife::Alive,
                self_rule: SelfTargetRule::Forbid,
                ..Default::default()
            },
            implementation: SkillImplementation::Implemented,
            effects: vec![Effect::Damage {
                amount: 10,
                target: TargetShape::Row,
                per_hop: DamageCurve::Constant,
            }],
            ..Default::default()
        }]);

        let err =
            validate_skill_book(&book).expect_err("row damage with single targeting must fail");
        assert_eq!(err.skill_id, SkillId("row_mismatch".into()));
        assert_eq!(err.category, SkillBookValidationCategory::Semantic);
        assert_eq!(err.reason, LegalityReasonCode::UnimplementedTargetShape);
        assert!(err.detail.contains("contradicts targeting.shape"));
    }

    #[test]
    fn validate_rejects_revive_with_non_ko_targeting() {
        let book = SkillBook(vec![SkillDef {
            id: SkillId("bad_revive".into()),
            name: "Bad Revive".into(),
            damage_tag: DamageTag::Light,
            sp_cost: 0,
            targeting: SkillTargeting {
                shape: TargetShape::Single,
                side: TargetSide::Ally,
                life: TargetLife::Alive,
                self_rule: SelfTargetRule::Forbid,
                ..Default::default()
            },
            implementation: SkillImplementation::Implemented,
            effects: vec![Effect::Revive(25)],
            ..Default::default()
        }]);

        let err = validate_skill_book(&book).expect_err("revive with alive targeting must fail");
        assert_eq!(err.skill_id, SkillId("bad_revive".into()));
        assert_eq!(err.category, SkillBookValidationCategory::Semantic);
        assert_eq!(err.reason, LegalityReasonCode::TargetNotKo);
        assert!(err.detail.contains("KO units"));
    }

    #[test]
    fn validate_rejects_implemented_non_single_shape() {
        let book = SkillBook(vec![SkillDef {
            id: SkillId("wide_impl".into()),
            name: "Wide Impl".into(),
            damage_tag: DamageTag::Fire,
            sp_cost: 0,
            targeting: SkillTargeting {
                shape: TargetShape::Row,
                side: TargetSide::Enemy,
                life: TargetLife::Alive,
                self_rule: SelfTargetRule::Forbid,
                ..Default::default()
            },
            implementation: SkillImplementation::Implemented,
            effects: vec![Effect::Damage {
                amount: 10,
                target: TargetShape::Row,
                per_hop: DamageCurve::Constant,
            }],
            ..Default::default()
        }]);

        let err = validate_skill_book(&book).expect_err("implemented Row shape must fail");
        assert_eq!(err.skill_id, SkillId("wide_impl".into()));
        assert_eq!(err.category, SkillBookValidationCategory::Semantic);
        assert_eq!(err.reason, LegalityReasonCode::UnimplementedTargetShape);
        assert!(err.detail.contains("Row"));
    }

    #[test]
    fn validate_allows_canonical_mixed_effect_deferral() {
        let book = SkillBook(vec![SkillDef {
            id: SkillId("angemon_ult".into()),
            name: "God Typhoon".into(),
            damage_tag: DamageTag::Light,
            sp_cost: 0,
            targeting: SkillTargeting {
                shape: TargetShape::Row,
                side: TargetSide::Enemy,
                life: TargetLife::Alive,
                self_rule: SelfTargetRule::Forbid,
                ..Default::default()
            },
            implementation: SkillImplementation::Deferred {
                reason: LegalityReasonCode::UnimplementedEffect,
            },
            effects: vec![
                Effect::Damage {
                    amount: 48,
                    target: TargetShape::Row,
                    per_hop: DamageCurve::Constant,
                },
                Effect::ToughnessHit(28),
                Effect::Revive(20),
            ],
            ..Default::default()
        }]);

        validate_skill_book(&book).expect("mixed-effect deferral should validate");
    }

    #[test]
    fn parse_canonical_skills_ron() {
        let book = canonical_skill_book();
        assert_eq!(book.0.len(), 74, "unexpected skill catalog size");

        let ids: HashSet<_> = book.0.iter().map(|skill| skill.id.clone()).collect();
        assert_eq!(ids.len(), book.0.len(), "duplicate skill ids in skills.ron");
        assert!(book.0.iter().all(|skill| !skill.effects.is_empty()));
        validate_skill_book(&book).expect("canonical skills.ron must validate");

        for required in [
            // Surviving Child kits (post-cleanup D039)
            "baby_flame",
            "bubble_blast",
            "draconic_edge",
            "diamond_storm",
            "holy_breeze",
            "agumon_ult",
            "gabumon_ult",
            "dorumon_ult",
            "renamon_ult",
            "patamon_ult",
            "agumon_follow_up",
            "renamon_follow_up",
            "patamon_revive",
        ] {
            assert!(
                ids.contains(&SkillId(required.into())),
                "missing Child skill asset {required}"
            );
        }

        for legacy in ["heat_viper", "tentomon_ult", "biyomon_ult", "flame_bite"] {
            assert!(
                ids.contains(&SkillId(legacy.into())),
                "missing compatibility skill asset {legacy}"
            );
        }

        // MVP v5.3 skill assets (D039)
        for mvp in [
            "tentomon_basic",
            "petit_thunder",
            "tentomon_follow_up",
            "greymon_basic",
            "mega_flame",
            "horn_impulse",
            "greymon_ult",
            "greymon_follow_up",
            "garurumon_basic",
            "foxfire",
            "freeze_fang",
            "garurumon_ult",
            "garurumon_follow_up",
            "kabuterimon_basic",
            "mega_blaster",
            "mega_blaster_aoe",
            "kabuterimon_ult",
            "kabuterimon_follow_up",
            "kyubimon_basic",
            "onibidama",
            "koenryu",
            "kyubimon_ult",
            "kyubimon_follow_up",
            "dorugamon_basic",
            "power_metal",
            "cannonball",
            "dorugamon_ult",
            "dorugamon_follow_up",
            "angemon_basic",
            "heavens_knuckle",
            "holy_rod",
            "angemon_ult",
            "angemon_follow_up",
            // Form Identity skills (T02+)
            "greymon_form_identity",
            "garurumon_form_identity",
            "kabuterimon_form_identity",
            "kyubimon_form_identity",
            "dorugamon_form_identity",
            "angemon_form_identity",
        ] {
            assert!(
                ids.contains(&SkillId(mvp.into())),
                "missing MVP v5.3 skill asset {mvp}"
            );
        }
    }

    // ── chain_bolt inline fixture ──────────────────────────────────────────────

    /// Returns the canonical chain_bolt fixture: 3-hop Bounce with LowestHpPctAlive,
    /// NoRepeat, and a Falloff curve (80% per hop).
    fn chain_bolt_skill() -> SkillDef {
        SkillDef {
            id: SkillId("chain_bolt".into()),
            name: "Chain Bolt".into(),
            damage_tag: DamageTag::Electric,
            sp_cost: 3,
            targeting: offensive_targeting(TargetShape::Bounce {
                hops: 3,
                selector: BounceSelector::LowestHpPctAlive,
                repeat: RepeatPolicy::NoRepeat,
            }),
            implementation: SkillImplementation::Implemented,
            effects: vec![
                Effect::Damage {
                    amount: 20,
                    target: TargetShape::Bounce {
                        hops: 3,
                        selector: BounceSelector::LowestHpPctAlive,
                        repeat: RepeatPolicy::NoRepeat,
                    },
                    per_hop: DamageCurve::Falloff { pct: 80 },
                },
                Effect::ToughnessHit(8),
            ],
            ..Default::default()
        }
    }

    #[test]
    fn chain_bolt_fixture_validates() {
        let book = SkillBook(vec![chain_bolt_skill()]);
        validate_skill_book(&book).expect("chain_bolt must validate");
    }

    #[test]
    fn bounce_target_shape_ron_roundtrip() {
        let shape = TargetShape::Bounce {
            hops: 3,
            selector: BounceSelector::LowestHpPctAlive,
            repeat: RepeatPolicy::NoRepeat,
        };
        let s = ron::to_string(&shape).expect("serialize");
        let back: TargetShape = ron::from_str(&s).expect("deserialize");
        assert_eq!(shape, back);
    }

    #[test]
    fn damage_curve_constant_roundtrip() {
        let curve = DamageCurve::Constant;
        let s = ron::to_string(&curve).expect("serialize");
        let back: DamageCurve = ron::from_str(&s).expect("deserialize");
        assert_eq!(curve, back);
    }

    #[test]
    fn damage_curve_falloff_roundtrip() {
        let curve = DamageCurve::Falloff { pct: 75 };
        let s = ron::to_string(&curve).expect("serialize");
        let back: DamageCurve = ron::from_str(&s).expect("deserialize");
        assert_eq!(curve, back);
    }

    #[test]
    fn damage_curve_per_hop_roundtrip() {
        let curve = DamageCurve::PerHop(vec![30, 25, 20]);
        let s = ron::to_string(&curve).expect("serialize");
        let back: DamageCurve = ron::from_str(&s).expect("deserialize");
        assert_eq!(curve, back);
    }

    #[test]
    fn effect_damage_with_bounce_shape_roundtrip() {
        let effect = Effect::Damage {
            amount: 20,
            target: TargetShape::Bounce {
                hops: 3,
                selector: BounceSelector::LowestHpPctAlive,
                repeat: RepeatPolicy::NoRepeat,
            },
            per_hop: DamageCurve::Falloff { pct: 80 },
        };
        let s = ron::to_string(&effect).expect("serialize");
        let back: Effect = ron::from_str(&s).expect("deserialize");
        assert_eq!(effect, back);
    }

    #[test]
    fn validator_accepts_bounce_with_per_hop_curve_matching_hops() {
        let book = SkillBook(vec![SkillDef {
            id: SkillId("per_hop_test".into()),
            name: "PerHop Test".into(),
            damage_tag: DamageTag::Electric,
            sp_cost: 3,
            targeting: offensive_targeting(TargetShape::Bounce {
                hops: 3,
                selector: BounceSelector::NextSlotAlive,
                repeat: RepeatPolicy::NoRepeat,
            }),
            implementation: SkillImplementation::Implemented,
            effects: vec![Effect::Damage {
                amount: 25,
                target: TargetShape::Bounce {
                    hops: 3,
                    selector: BounceSelector::NextSlotAlive,
                    repeat: RepeatPolicy::NoRepeat,
                },
                per_hop: DamageCurve::PerHop(vec![30, 25, 20]),
            }],
            ..Default::default()
        }]);
        validate_skill_book(&book).expect("PerHop with matching length must validate");
    }

    #[test]
    fn validator_accepts_bounce_with_falloff_curve() {
        let book = SkillBook(vec![SkillDef {
            id: SkillId("falloff_test".into()),
            name: "Falloff Test".into(),
            damage_tag: DamageTag::Electric,
            sp_cost: 3,
            targeting: offensive_targeting(TargetShape::Bounce {
                hops: 2,
                selector: BounceSelector::AdjLowest,
                repeat: RepeatPolicy::AllowRepeat,
            }),
            implementation: SkillImplementation::Implemented,
            effects: vec![Effect::Damage {
                amount: 20,
                target: TargetShape::Bounce {
                    hops: 2,
                    selector: BounceSelector::AdjLowest,
                    repeat: RepeatPolicy::AllowRepeat,
                },
                per_hop: DamageCurve::Falloff { pct: 100 },
            }],
            ..Default::default()
        }]);
        validate_skill_book(&book).expect("Falloff pct=100 must validate");
    }

    #[test]
    fn validator_rejects_per_hop_length_mismatch() {
        let book = SkillBook(vec![SkillDef {
            id: SkillId("per_hop_mismatch".into()),
            name: "PerHop Mismatch".into(),
            damage_tag: DamageTag::Electric,
            sp_cost: 3,
            targeting: offensive_targeting(TargetShape::Bounce {
                hops: 3,
                selector: BounceSelector::LowestHpPctAlive,
                repeat: RepeatPolicy::NoRepeat,
            }),
            implementation: SkillImplementation::Implemented,
            effects: vec![Effect::Damage {
                amount: 25,
                target: TargetShape::Bounce {
                    hops: 3,
                    selector: BounceSelector::LowestHpPctAlive,
                    repeat: RepeatPolicy::NoRepeat,
                },
                // length 2, but hops = 3 -> mismatch
                per_hop: DamageCurve::PerHop(vec![30, 20]),
            }],
            ..Default::default()
        }]);
        let err = validate_skill_book(&book).expect_err("PerHop length mismatch must fail");
        assert_eq!(err.skill_id, SkillId("per_hop_mismatch".into()));
        assert_eq!(err.reason, LegalityReasonCode::UnimplementedEffect);
        assert!(err.detail.contains("PerHop length"));
    }

    #[test]
    fn validator_rejects_bounce_hops_zero() {
        let book = SkillBook(vec![SkillDef {
            id: SkillId("zero_hops".into()),
            name: "Zero Hops".into(),
            damage_tag: DamageTag::Electric,
            sp_cost: 3,
            targeting: offensive_targeting(TargetShape::Bounce {
                hops: 0,
                selector: BounceSelector::LowestHpPctAlive,
                repeat: RepeatPolicy::NoRepeat,
            }),
            implementation: SkillImplementation::Deferred {
                reason: LegalityReasonCode::UnimplementedTargetShape,
            },
            effects: vec![Effect::Damage {
                amount: 10,
                target: TargetShape::Bounce {
                    hops: 0,
                    selector: BounceSelector::LowestHpPctAlive,
                    repeat: RepeatPolicy::NoRepeat,
                },
                per_hop: DamageCurve::Constant,
            }],
            ..Default::default()
        }]);
        let err = validate_skill_book(&book).expect_err("Bounce hops=0 must fail");
        assert_eq!(err.reason, LegalityReasonCode::UnimplementedTargetShape);
        assert!(err.detail.contains("hops"));
    }

    #[test]
    fn validator_rejects_falloff_pct_over_100() {
        let book = SkillBook(vec![SkillDef {
            id: SkillId("bad_falloff".into()),
            name: "Bad Falloff".into(),
            damage_tag: DamageTag::Electric,
            sp_cost: 3,
            targeting: offensive_targeting(TargetShape::Bounce {
                hops: 2,
                selector: BounceSelector::NextSlotAlive,
                repeat: RepeatPolicy::NoRepeat,
            }),
            implementation: SkillImplementation::Implemented,
            effects: vec![Effect::Damage {
                amount: 20,
                target: TargetShape::Bounce {
                    hops: 2,
                    selector: BounceSelector::NextSlotAlive,
                    repeat: RepeatPolicy::NoRepeat,
                },
                per_hop: DamageCurve::Falloff { pct: 150 },
            }],
            ..Default::default()
        }]);
        let err = validate_skill_book(&book).expect_err("Falloff pct > 100 must fail");
        assert_eq!(err.reason, LegalityReasonCode::UnimplementedEffect);
        assert!(err.detail.contains("pct"));
    }
}
