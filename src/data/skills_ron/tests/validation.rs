use super::*;
use std::collections::HashSet;

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
