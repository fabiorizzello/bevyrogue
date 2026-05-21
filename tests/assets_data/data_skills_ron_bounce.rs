use bevyrogue::combat::types::{DamageTag, SkillId};
use bevyrogue::data::skills_ron::{
    BounceSelector, DamageCurve, Effect, LegalityReasonCode, RepeatPolicy, SelfTargetRule,
    SkillBook, SkillDef, SkillImplementation, SkillTargeting, TargetLife, TargetShape, TargetSide,
    validate_skill_book,
};

fn offensive_targeting(shape: TargetShape) -> SkillTargeting {
    SkillTargeting {
        shape,
        side: TargetSide::Enemy,
        life: TargetLife::Alive,
        self_rule: SelfTargetRule::Forbid,
        ..Default::default()
    }
}

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
        legacy_ops: vec![Effect::Damage {
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
        legacy_ops: vec![Effect::Damage {
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
        legacy_ops: vec![Effect::Damage {
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
        legacy_ops: vec![Effect::Damage {
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
        legacy_ops: vec![Effect::Damage {
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
