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
    let back: Effect = ron::from_str("Damage(amount: 18, target: Single)")
        .expect("parse with default per_hop");
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
            legacy_ops: [Damage(amount: 1, target: Single)]
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
            legacy_ops: [Damage(amount: 1, target: Single)]
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
        legacy_ops: vec![Effect::Damage {
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
        legacy_ops: vec![Effect::Revive(25)],
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
        legacy_ops: vec![Effect::Damage {
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
        legacy_ops: vec![
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
    assert_eq!(book.0.len(), 65, "unexpected skill catalog size");

    let ids: HashSet<_> = book.0.iter().map(|skill| skill.id.clone()).collect();
    assert_eq!(ids.len(), book.0.len(), "duplicate skill ids in skills.ron");
    assert!(book.0.iter().all(|skill| !skill.legacy_ops.is_empty()));
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

    for legacy in ["tentomon_ult"] {
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
