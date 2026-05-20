use bevyrogue::combat::status_effect::StatusEffectKind;
use bevyrogue::combat::types::{DamageTag, SkillId};
use bevyrogue::data::skills_ron::{
    DamageCurve, Effect, LegalityReasonCode, SelfTargetRule, SkillDef, SkillImplementation,
    SkillTargeting, TargetLife, TargetShape, TargetSide,
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
