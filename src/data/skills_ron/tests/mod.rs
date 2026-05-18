use super::*;
use crate::combat::types::{DamageTag, SkillId};

mod bounce;
mod roundtrip;
mod validation;

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
        legacy_ops: vec![
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
    crate::data::aggregate_skill_book()
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
        legacy_ops: vec![
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
